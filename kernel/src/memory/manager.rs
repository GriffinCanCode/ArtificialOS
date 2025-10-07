/*!
 * Memory Management
 * Handles memory allocation and deallocation with graceful OOM handling
 */

use super::traits::{Allocator, GarbageCollector, MemoryInfo, ProcessMemoryCleanup};
use super::types::{MemoryBlock, MemoryError, MemoryPressure, MemoryResult, MemoryStats};
use crate::core::types::{Address, Pid, Size};
use dashmap::DashMap;
use log::{error, info, warn};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Per-process memory tracking
#[derive(Debug, Clone)]
struct ProcessMemoryTracking {
    current_bytes: Size,
    peak_bytes: Size,
    allocation_count: usize,
}

pub struct MemoryManager {
    blocks: Arc<DashMap<Address, MemoryBlock>>,
    next_address: Arc<AtomicU64>,
    total_memory: Size,
    used_memory: Arc<AtomicU64>,
    // Memory pressure thresholds (percentage)
    warning_threshold: f64,  // 80%
    critical_threshold: f64, // 95%
    // Garbage collection threshold - run GC when this many deallocated blocks accumulate
    gc_threshold: Size, // 1000 blocks
    deallocated_count: Arc<AtomicU64>,
    // Per-process memory tracking (for peak_bytes and allocation_count)
    process_tracking: Arc<DashMap<Pid, ProcessMemoryTracking>>,
    // Memory storage - maps addresses to their byte contents
    memory_storage: Arc<DashMap<Address, Vec<u8>>>,
}

impl MemoryManager {
    pub fn new() -> Self {
        let total = 1024 * 1024 * 1024; // 1GB simulated memory
        info!("Memory manager initialized with {} bytes", total);
        Self {
            blocks: Arc::new(DashMap::new()),
            next_address: Arc::new(AtomicU64::new(0)),
            total_memory: total,
            used_memory: Arc::new(AtomicU64::new(0)),
            warning_threshold: 0.80,
            critical_threshold: 0.95,
            gc_threshold: 1000,
            deallocated_count: Arc::new(AtomicU64::new(0)),
            process_tracking: Arc::new(DashMap::new()),
            memory_storage: Arc::new(DashMap::new()),
        }
    }

    /// Check memory pressure level
    fn check_memory_pressure(&self, used: Size) -> Option<MemoryPressure> {
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

    /// Allocate memory with graceful OOM handling
    pub fn allocate(&self, size: Size, pid: Pid) -> MemoryResult<Address> {
        // Check if allocation would exceed total memory atomically
        let used = self.used_memory.fetch_add(size, Ordering::SeqCst);

        if used + size > self.total_memory {
            // Revert the increment
            self.used_memory.fetch_sub(size, Ordering::SeqCst);

            let available = self.total_memory - used;
            error!(
                "OOM: PID {} requested {} bytes, only {} bytes available ({} used / {} total)",
                pid, size, available, used, self.total_memory
            );

            return Err(MemoryError::OutOfMemory {
                requested: size,
                available,
                used,
                total: self.total_memory,
            });
        }

        // Allocate address atomically
        let address = self.next_address.fetch_add(size, Ordering::SeqCst);

        let block = MemoryBlock {
            address,
            size,
            allocated: true,
            owner_pid: Some(pid),
        };

        self.blocks.insert(address, block);

        // Update per-process tracking
        self.process_tracking
            .entry(pid)
            .and_modify(|track| {
                track.current_bytes += size;
                track.allocation_count += 1;
                if track.current_bytes > track.peak_bytes {
                    track.peak_bytes = track.current_bytes;
                }
            })
            .or_insert(ProcessMemoryTracking {
                current_bytes: size,
                peak_bytes: size,
                allocation_count: 1,
            });

        // Log allocation with memory pressure warnings
        let used_val = used + size;

        if let Some(level) = self.check_memory_pressure(used_val) {
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

    /// Deallocate memory
    pub fn deallocate(&self, address: Address) -> MemoryResult<()> {
        if let Some(mut entry) = self.blocks.get_mut(&address) {
            let block = entry.value_mut();
            if block.allocated {
                let size = block.size;
                let pid = block.owner_pid;
                block.allocated = false;

                self.used_memory.fetch_sub(size, Ordering::SeqCst);

                // Update per-process tracking
                if let Some(pid) = pid {
                    if let Some(mut track) = self.process_tracking.get_mut(&pid) {
                        track.current_bytes = track.current_bytes.saturating_sub(size);
                    }
                }

                // Track deallocated blocks for GC
                let dealloc_count = self.deallocated_count.fetch_add(1, Ordering::SeqCst) + 1;

                let used = self.used_memory.load(Ordering::SeqCst);
                info!(
                    "Deallocated {} bytes at 0x{:x} ({} bytes now available, {} deallocated blocks)",
                    size,
                    address,
                    self.total_memory - used,
                    dealloc_count
                );

                // Trigger GC if threshold reached
                let should_gc = dealloc_count >= self.gc_threshold;
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

    /// Free all memory allocated to a specific process (called on process termination)
    pub fn free_process_memory(&self, pid: Pid) -> Size {
        let mut freed_bytes = 0;
        let mut freed_count = 0;

        for mut entry in self.blocks.iter_mut() {
            let block = entry.value_mut();
            if block.allocated && block.owner_pid == Some(pid) {
                block.allocated = false;
                freed_bytes += block.size;
                freed_count += 1;
            }
        }

        if freed_bytes > 0 {
            self.used_memory.fetch_sub(freed_bytes, Ordering::SeqCst);

            // Remove process tracking entry
            self.process_tracking.remove(&pid);

            // Track deallocated blocks for GC
            let dealloc_count = self.deallocated_count.fetch_add(freed_count, Ordering::SeqCst) + freed_count;

            let used = self.used_memory.load(Ordering::SeqCst);
            info!(
                "Cleaned up {} bytes ({} blocks) from terminated PID {} ({} bytes now available, {} deallocated blocks)",
                freed_bytes,
                freed_count,
                pid,
                self.total_memory - used,
                dealloc_count
            );

            // Trigger GC if threshold reached
            let should_gc = dealloc_count >= self.gc_threshold;

            if should_gc {
                info!("GC threshold reached after process cleanup, running garbage collection...");
                self.collect();
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
            (track.current_bytes, track.peak_bytes, track.allocation_count)
        } else {
            (0, 0, 0)
        }
    }

    /// Get overall memory info: (total, used, available)
    pub fn info(&self) -> (Size, Size, Size) {
        let used = self.used_memory.load(Ordering::SeqCst);
        (self.total_memory, used, self.total_memory - used)
    }

    /// Get detailed memory statistics
    pub fn stats(&self) -> MemoryStats {
        let used = self.used_memory.load(Ordering::SeqCst);

        let allocated_blocks = self.blocks.iter().filter(|entry| entry.value().allocated).count();
        let fragmented_blocks = self.blocks.iter().filter(|entry| !entry.value().allocated).count();

        MemoryStats {
            total_memory: self.total_memory,
            used_memory: used,
            available_memory: self.total_memory - used,
            usage_percentage: (used as f64 / self.total_memory as f64) * 100.0,
            allocated_blocks,
            fragmented_blocks,
        }
    }

    /// Garbage collect deallocated memory blocks
    /// Removes deallocated blocks from the HashMap to prevent unbounded growth
    pub fn collect(&self) -> Size {
        let initial_count = self.blocks.len();

        // Collect addresses of deallocated blocks before removing them
        let deallocated_addrs: Vec<Address> = self.blocks
            .iter()
            .filter(|entry| !entry.value().allocated)
            .map(|entry| *entry.key())
            .collect();

        // Remove deallocated blocks
        for addr in &deallocated_addrs {
            self.blocks.remove(addr);
        }

        let removed_count = deallocated_addrs.len();

        // Clean up storage for deallocated blocks
        if !deallocated_addrs.is_empty() {
            for addr in &deallocated_addrs {
                self.memory_storage.remove(addr);
            }
        }

        // Reset deallocated counter
        self.deallocated_count.store(0, Ordering::SeqCst);

        if removed_count > 0 {
            info!(
                "Garbage collection complete: removed {} deallocated blocks and their storage, {} blocks remain",
                removed_count,
                initial_count - removed_count
            );
        }

        removed_count
    }

    /// Force garbage collection (for testing or manual cleanup)
    pub fn force_collect(&self) -> Size {
        info!("Forcing garbage collection...");
        self.collect()
    }

    /// Check if GC should run
    pub fn should_collect(&self) -> bool {
        let dealloc_count = self.deallocated_count.load(Ordering::SeqCst);
        dealloc_count >= self.gc_threshold
    }

    /// Set GC threshold
    pub fn set_threshold(&mut self, _threshold: Size) {
        // Note: MemoryManager uses Arc internally, so this would need refactoring
        // to support mutable threshold changes. For now, threshold is set at construction.
    }

    /// Get list of allocations for a process
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
    /// Check if an address is valid and allocated
    pub fn is_valid(&self, address: Address) -> bool {
        self.blocks.get(&address).map(|entry| entry.value().allocated).unwrap_or(false)
    }

    /// Get the size of an allocated block
    pub fn block_size(&self, address: Address) -> Option<Size> {
        self.blocks.get(&address).and_then(|entry| {
            let b = entry.value();
            if b.allocated {
                Some(b.size)
            } else {
                None
            }
        })
    }

    /// Write bytes to a memory address
    /// This simulates writing to physical memory for shared memory segments
    pub fn write_bytes(&self, address: Address, data: &[u8]) -> MemoryResult<()> {
        // Find the block containing this address
        let mut base_addr = None;
        let mut block_size = 0;
        for entry in self.blocks.iter() {
            let addr = *entry.key();
            let block = entry.value();
            if block.allocated && address >= addr && address < addr + block.size {
                // Check if write fits within block bounds
                if address + data.len() <= addr + block.size {
                    base_addr = Some(addr);
                    block_size = block.size;
                    break;
                } else {
                    return Err(MemoryError::InvalidAddress(address));
                }
            }
        }

        if let Some(base_addr) = base_addr {
            // Calculate offset within the block
            let offset = address - base_addr;

            // Get or create storage for this block
            self.memory_storage
                .entry(base_addr)
                .and_modify(|block_data| {
                    // Ensure block_data is large enough
                    if block_data.len() < block_size {
                        block_data.resize(block_size, 0u8);
                    }
                    // Write data at the offset
                    let end = offset + data.len();
                    block_data[offset..end].copy_from_slice(data);
                })
                .or_insert_with(|| {
                    let mut vec = vec![0u8; block_size];
                    let end = offset + data.len();
                    vec[offset..end].copy_from_slice(data);
                    vec
                });

            info!(
                "Wrote {} bytes to address 0x{:x} (offset {} in block at 0x{:x})",
                data.len(), address, offset, base_addr
            );
            Ok(())
        } else {
            Err(MemoryError::InvalidAddress(address))
        }
    }

    /// Read bytes from a memory address
    /// This simulates reading from physical memory for shared memory segments
    pub fn read_bytes(&self, address: Address, size: Size) -> MemoryResult<Vec<u8>> {
        // Find the block containing this address
        let mut base_addr = None;
        for entry in self.blocks.iter() {
            let addr = *entry.key();
            let block = entry.value();
            if block.allocated && address >= addr && address < addr + block.size {
                // Check if read fits within block bounds
                if address + size <= addr + block.size {
                    base_addr = Some(addr);
                    break;
                } else {
                    return Err(MemoryError::InvalidAddress(address));
                }
            }
        }

        if let Some(base_addr) = base_addr {
            // Calculate offset within the block
            let offset = address - base_addr;

            // Get storage for this block
            let data = if let Some(block_data) = self.memory_storage.get(&base_addr) {
                // Read data from the stored bytes
                let end = offset + size;
                block_data[offset..end].to_vec()
            } else {
                // Block has no data written yet, return zeros
                vec![0u8; size]
            };

            info!(
                "Read {} bytes from address 0x{:x} (offset {} in block at 0x{:x})",
                size, address, offset, base_addr
            );

            Ok(data)
        } else {
            Err(MemoryError::InvalidAddress(address))
        }
    }
}

// Implement trait interfaces
impl Allocator for MemoryManager {
    fn allocate(&self, size: Size, pid: Pid) -> MemoryResult<Address> {
        MemoryManager::allocate(self, size, pid)
    }

    fn deallocate(&self, address: Address) -> MemoryResult<()> {
        MemoryManager::deallocate(self, address)
    }

    fn is_valid(&self, address: Address) -> bool {
        MemoryManager::is_valid(self, address)
    }

    fn block_size(&self, address: Address) -> Option<Size> {
        MemoryManager::block_size(self, address)
    }
}

impl MemoryInfo for MemoryManager {
    fn stats(&self) -> MemoryStats {
        MemoryManager::stats(self)
    }

    fn info(&self) -> (Size, Size, Size) {
        MemoryManager::info(self)
    }

    fn process_memory(&self, pid: Pid) -> Size {
        MemoryManager::process_memory(self, pid)
    }
}

impl GarbageCollector for MemoryManager {
    fn collect(&self) -> Size {
        MemoryManager::collect(self)
    }

    fn force_collect(&self) -> Size {
        MemoryManager::force_collect(self)
    }

    fn should_collect(&self) -> bool {
        MemoryManager::should_collect(self)
    }

    fn set_threshold(&mut self, threshold: Size) {
        MemoryManager::set_threshold(self, threshold)
    }
}

impl ProcessMemoryCleanup for MemoryManager {
    fn free_process_memory(&self, pid: Pid) -> Size {
        MemoryManager::free_process_memory(self, pid)
    }

    fn process_allocations(&self, pid: Pid) -> Vec<MemoryBlock> {
        MemoryManager::process_allocations(self, pid)
    }
}

impl Clone for MemoryManager {
    fn clone(&self) -> Self {
        Self {
            blocks: Arc::clone(&self.blocks),
            next_address: Arc::clone(&self.next_address),
            total_memory: self.total_memory,
            used_memory: Arc::clone(&self.used_memory),
            warning_threshold: self.warning_threshold,
            critical_threshold: self.critical_threshold,
            gc_threshold: self.gc_threshold,
            deallocated_count: Arc::clone(&self.deallocated_count),
            process_tracking: Arc::clone(&self.process_tracking),
            memory_storage: Arc::clone(&self.memory_storage),
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
