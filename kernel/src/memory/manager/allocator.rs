/*!
 * Memory Allocator Implementation
 * Allocation and deallocation logic
 */

use super::super::types::{MemoryBlock, MemoryError, MemoryPressure, MemoryResult};
use super::free_list::FreeBlock;
use super::MemoryManager;
use crate::core::types::{Address, Pid, Size};
use crate::monitoring::{Category, Event, Payload, Severity};
use log::{error, info, warn};
use std::sync::atomic::Ordering;

impl MemoryManager {
    /// Allocate memory with graceful OOM handling and address recycling
    /// Uses segregated free lists for O(1) small/medium and O(log n) large allocations
    pub fn allocate(&self, size: Size, pid: Pid) -> MemoryResult<Address> {
        // Check if allocation would exceed total memory atomically
        let size_u64 = size as u64;
        let used = self.used_memory.fetch_add(size_u64, Ordering::SeqCst);

        if used + size_u64 > self.total_memory as u64 {
            // Revert the increment
            self.used_memory.fetch_sub(size_u64, Ordering::SeqCst);

            let available = self.total_memory - used as usize;
            error!(
                "OOM: PID {} requested {} bytes, only {} bytes available ({} used / {} total)",
                pid, size, available, used, self.total_memory
            );

            return Err(MemoryError::OutOfMemory {
                requested: size,
                available,
                used: used as usize,
                total: self.total_memory,
            });
        }

        // Try to recycle an address from the segregated free list (O(1) or O(log n) lookup)
        let address = {
            let mut free_list = match self.free_list.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    log::error!("Free list mutex poisoned during allocation - recovering");
                    poisoned.into_inner()
                }
            };

            if let Some(free_block) = free_list.find_best_fit(size) {
                let address = free_block.address;

                info!(
                    "Recycled address 0x{:x} (block size: {}, requested: {}) for PID {} [segregated list: O(1)/O(log n)]",
                    address, free_block.size, size, pid
                );

                // If the free block is larger than needed, split it and return the remainder
                if free_block.size > size {
                    let remainder_size = free_block.size - size;
                    let remainder_addr = address + size;
                    free_list.insert(FreeBlock {
                        address: remainder_addr,
                        size: remainder_size,
                    });
                    info!(
                        "Split block: keeping {} bytes, returning {} bytes at 0x{:x} to free list",
                        size, remainder_size, remainder_addr
                    );
                }

                address
            } else {
                // No suitable free block, allocate new address
                self.next_address.fetch_add(size_u64, Ordering::SeqCst) as usize
            }
        };

        let block = MemoryBlock {
            address,
            size,
            allocated: true,
            owner_pid: Some(pid),
        };

        self.blocks.insert(address, block);

        // Update per-process tracking using entry() for atomic operation
        {
            let mut track = self.process_tracking.entry(pid).or_insert_with(|| {
                crate::memory::manager::tracking::ProcessMemoryTracking::new()
            });
            track.current_bytes += size;
            track.allocation_count += 1;
            if track.current_bytes > track.peak_bytes {
                track.peak_bytes = track.current_bytes;
            }
        }

        // Log allocation with memory pressure warnings
        let used_val = used as usize + size;

        // Emit memory allocation event
        if let Some(ref collector) = self.collector {
            collector.emit(
                Event::new(
                    Severity::Debug,
                    Category::Memory,
                    Payload::MemoryAllocated {
                        size,
                        region_id: address as u64,
                    },
                )
                .with_pid(pid),
            );
        }

        if let Some(level) = self.check_memory_pressure(used_val) {
            // Emit memory pressure event
            if let Some(ref collector) = self.collector {
                let usage_pct = ((used_val as f64 / self.total_memory as f64) * 100.0) as u8;
                let available_mb = ((self.total_memory - used_val) / 1024 / 1024) as u64;
                collector.memory_pressure(usage_pct, available_mb);
            }

            warn!(
                "Memory pressure {}: Allocated {} bytes at 0x{:x} for PID {} ({:.1}% used: {} / {})",
                level, size, address, pid,
                (used_val as f64 / self.total_memory as f64) * 100.0,
                used_val, self.total_memory
            );
        } else {
            info!(
                "Allocated {} bytes at 0x{:x} for PID {}",
                size, address, pid
            );
        }

        Ok(address)
    }

    /// Deallocate memory and add to segregated free list for address recycling
    pub fn deallocate(&self, address: Address) -> MemoryResult<()> {
        if let Some(mut entry) = self.blocks.get_mut(&address) {
            let block = entry.value_mut();
            if block.allocated {
                let size = block.size;
                let pid = block.owner_pid;
                block.allocated = false;

                self.used_memory.fetch_sub(size as u64, Ordering::SeqCst);

                // Update per-process tracking
                if let Some(pid) = pid {
                    if let Some(mut track) = self.process_tracking.get_mut(&pid) {
                        track.current_bytes = track.current_bytes.saturating_sub(size);
                    }
                }

                // Emit memory freed event
                if let Some(ref collector) = self.collector {
                    if let Some(pid) = pid {
                        collector.emit(
                            Event::new(
                                Severity::Debug,
                                Category::Memory,
                                Payload::MemoryFreed {
                                    size,
                                    region_id: address as u64,
                                },
                            )
                            .with_pid(pid),
                        );
                    }
                }

                // Add to segregated free list for address recycling
                {
                    let mut free_list = match self.free_list.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            log::error!(
                                "Free list mutex poisoned during deallocation - recovering"
                            );
                            poisoned.into_inner()
                        }
                    };
                    free_list.insert(FreeBlock { address, size });

                    // Periodically coalesce adjacent blocks to reduce fragmentation
                    // Only coalesce every 100 deallocations to amortize the O(n log n) cost
                    if self.deallocated_count.load(Ordering::SeqCst) % 100 == 0 {
                        Self::coalesce_free_blocks(&mut free_list);
                    }
                }

                // Track deallocated blocks for GC
                let dealloc_count = self.deallocated_count.fetch_add(1, Ordering::SeqCst) + 1;

                let used = self.used_memory.load(Ordering::SeqCst);
                info!(
                    "Deallocated {} bytes at 0x{:x}, added to segregated free list ({} bytes now available, {} deallocated blocks)",
                    size,
                    address,
                    self.total_memory - used as usize,
                    dealloc_count
                );

                // Trigger GC if threshold reached
                let should_gc = dealloc_count >= self.gc_threshold as u64;
                drop(entry);

                if should_gc {
                    info!("GC threshold reached, running garbage collection...");
                    self.collect();
                }

                return Ok(());
            }
        }

        warn!(
            "Attempted to deallocate invalid or already freed address: 0x{:x}",
            address
        );
        Err(MemoryError::InvalidAddress(address))
    }

    /// Check if an address is currently allocated
    pub fn is_valid(&self, address: Address) -> bool {
        self.blocks
            .get(&address)
            .map_or(false, |entry| entry.value().allocated)
    }

    /// Get the size of a memory block if it exists
    pub fn block_size(&self, address: Address) -> Option<Size> {
        self.blocks.get(&address).map(|entry| entry.value().size)
    }

    /// Check memory pressure level
    pub(super) fn check_memory_pressure(&self, used: Size) -> Option<MemoryPressure> {
        let usage_ratio = used as f64 / self.total_memory as f64;

        if usage_ratio >= self.critical_threshold {
            Some(MemoryPressure::Critical)
        } else if usage_ratio >= self.warning_threshold {
            Some(MemoryPressure::High)
        } else if usage_ratio >= 0.60 {
            Some(MemoryPressure::Medium)
        } else {
            None
        }
    }
}
