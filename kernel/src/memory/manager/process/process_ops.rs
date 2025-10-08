/*!
 * Process Memory Operations
 * Process-specific memory management and statistics
 */

use super::super::core::{FreeBlock, MemoryBlock, MemoryStats, SegregatedFreeList};
use super::super::MemoryManager;
use crate::core::types::{Pid, Size};
use log::info;
use std::sync::atomic::Ordering;

impl MemoryManager {
    /// Coalesce adjacent free blocks to reduce fragmentation
    /// Works with segregated free lists by temporarily extracting all blocks
    pub(in crate::memory::manager) fn coalesce_free_blocks(free_list: &mut SegregatedFreeList) {
        if free_list.len() < 2 {
            return;
        }

        // Extract all blocks and sort by address
        let mut all_blocks = free_list.get_all_sorted();

        // Coalesce adjacent blocks
        let mut i = 0;
        let mut coalesced_count = 0;
        while i < all_blocks.len() - 1 {
            let current_end = all_blocks[i].address + all_blocks[i].size;
            let next_start = all_blocks[i + 1].address;

            // If blocks are adjacent, merge them
            if current_end == next_start {
                let next_size = all_blocks[i + 1].size;
                all_blocks[i].size += next_size;
                all_blocks.remove(i + 1);
                coalesced_count += 1;
            } else {
                i += 1;
            }
        }

        if coalesced_count > 0 {
            info!(
                "Coalesced {} pairs of adjacent free blocks, reduced from {} to {} blocks",
                coalesced_count,
                free_list.len() + coalesced_count,
                all_blocks.len()
            );
        }

        // Reinsert all blocks into segregated lists
        free_list.reinsert_all(all_blocks);
    }

    /// Free all memory allocated to a specific process (called on process termination)
    pub fn free_process_memory(&self, pid: Pid) -> Size {
        let mut freed_bytes = 0;
        let mut freed_count = 0;
        let mut freed_blocks = Vec::new();

        for mut entry in self.blocks.iter_mut() {
            let block = entry.value_mut();
            if block.allocated && block.owner_pid == Some(pid) {
                block.allocated = false;
                freed_bytes += block.size;
                freed_count += 1;
                freed_blocks.push(FreeBlock {
                    address: block.address,
                    size: block.size,
                });
            }
        }

        if freed_bytes > 0 {
            self.used_memory
                .fetch_sub(freed_bytes as u64, Ordering::SeqCst);

            // Remove process tracking entry
            self.process_tracking.remove(&pid);

            // Add freed blocks to segregated free list for recycling
            {
                match self.free_list.lock() {
                    Ok(mut free_list) => {
                        for block in freed_blocks {
                            free_list.insert(block);
                        }
                        // Always coalesce after large batch frees
                        Self::coalesce_free_blocks(&mut free_list);
                    }
                    Err(poisoned) => {
                        // Mutex poisoned: thread panicked while holding lock
                        // Attempt recovery by acquiring poisoned guard
                        log::error!("Free list mutex poisoned during process {} cleanup - attempting recovery", pid);
                        let mut free_list = poisoned.into_inner();
                        for block in freed_blocks {
                            free_list.insert(block);
                        }
                        Self::coalesce_free_blocks(&mut free_list);
                    }
                }
            }

            // Track deallocated blocks for GC
            let dealloc_count = self
                .deallocated_count
                .fetch_add(freed_count, Ordering::SeqCst)
                + freed_count;

            let used = self.used_memory.load(Ordering::SeqCst);
            info!(
                "Cleaned up {} bytes ({} blocks) from terminated PID {}, added to segregated free list ({} bytes now available, {} deallocated blocks)",
                freed_bytes,
                freed_count,
                pid,
                self.total_memory - used as usize,
                dealloc_count
            );

            // Trigger GC if threshold reached
            let should_gc = dealloc_count >= self.gc_threshold as u64;

            if should_gc {
                info!("GC threshold reached after process cleanup, running garbage collection...");
                self.collect();
            } else if freed_count > 100 {
                // For large cleanups, shrink maps even without full GC
                self.blocks.shrink_to_fit();
                self.process_tracking.shrink_to_fit();
                info!(
                    "Shrunk memory maps after large process cleanup ({} blocks freed)",
                    freed_count
                );
            }
        }

        freed_bytes
    }

    /// Get memory statistics for a specific process
    pub fn process_memory(&self, pid: Pid) -> Size {
        self.blocks
            .iter()
            .filter(|entry| {
                let b = entry.value();
                b.allocated && b.owner_pid == Some(pid)
            })
            .map(|entry| entry.value().size)
            .sum()
    }

    /// Get detailed process memory stats including peak and allocation count
    pub fn get_process_memory_details(&self, pid: Pid) -> (Size, Size, usize) {
        if let Some(track) = self.process_tracking.get(&pid) {
            (
                track.current_bytes,
                track.peak_bytes,
                track.allocation_count,
            )
        } else {
            (0, 0, 0)
        }
    }

    /// Get overall memory info: (total, used, available)
    pub fn info(&self) -> (Size, Size, Size) {
        let used = self.used_memory.load(Ordering::SeqCst) as usize;
        (self.total_memory, used, self.total_memory - used)
    }

    /// Get detailed memory statistics
    pub fn stats(&self) -> MemoryStats {
        let used = self.used_memory.load(Ordering::SeqCst) as usize;

        let allocated_blocks = self
            .blocks
            .iter()
            .filter(|entry| entry.value().allocated)
            .count();
        let fragmented_blocks = self
            .blocks
            .iter()
            .filter(|entry| !entry.value().allocated)
            .count();

        MemoryStats {
            total_memory: self.total_memory,
            used_memory: used,
            available_memory: self.total_memory - used,
            usage_percentage: (used as f64 / self.total_memory as f64) * 100.0,
            allocated_blocks,
            fragmented_blocks,
        }
    }

    /// Get all memory blocks allocated to a process
    pub fn process_allocations(&self, pid: Pid) -> Vec<MemoryBlock> {
        self.blocks
            .iter()
            .filter(|entry| {
                let b = entry.value();
                b.allocated && b.owner_pid == Some(pid)
            })
            .map(|entry| entry.value().clone())
            .collect()
    }
}
