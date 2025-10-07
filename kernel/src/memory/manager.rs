/*!
 * Memory Management
 * Handles memory allocation and deallocation with graceful OOM handling
 */

use super::traits::{Allocator, GarbageCollector, MemoryInfo, ProcessMemoryCleanup};
use super::types::{MemoryBlock, MemoryError, MemoryPressure, MemoryResult, MemoryStats};
use crate::core::types::{Address, Pid, Size};
use log::{error, info, warn};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Per-process memory tracking
#[derive(Debug, Clone)]
struct ProcessMemoryTracking {
    current_bytes: Size,
    peak_bytes: Size,
    allocation_count: usize,
}

pub struct MemoryManager {
    blocks: Arc<RwLock<HashMap<Address, MemoryBlock>>>,
    next_address: Arc<RwLock<Address>>,
    total_memory: Size,
    used_memory: Arc<RwLock<Size>>,
    // Memory pressure thresholds (percentage)
    warning_threshold: f64,  // 80%
    critical_threshold: f64, // 95%
    // Garbage collection threshold - run GC when this many deallocated blocks accumulate
    gc_threshold: Size, // 1000 blocks
    deallocated_count: Arc<RwLock<Size>>,
    // Per-process memory tracking (for peak_bytes and allocation_count)
    process_tracking: Arc<RwLock<HashMap<Pid, ProcessMemoryTracking>>>,
}

impl MemoryManager {
    pub fn new() -> Self {
        let total = 1024 * 1024 * 1024; // 1GB simulated memory
        info!("Memory manager initialized with {} bytes", total);
        Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
            next_address: Arc::new(RwLock::new(0)),
            total_memory: total,
            used_memory: Arc::new(RwLock::new(0)),
            warning_threshold: 0.80,
            critical_threshold: 0.95,
            gc_threshold: 1000,
            deallocated_count: Arc::new(RwLock::new(0)),
            process_tracking: Arc::new(RwLock::new(HashMap::new())),
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
        // Single atomic transaction - acquire all locks at once to prevent races
        let mut blocks = self.blocks.write();
        let mut next_addr = self.next_address.write();
        let mut used = self.used_memory.write();
        let mut tracking = self.process_tracking.write();

        // Check if allocation would exceed total memory
        if *used + size > self.total_memory {
            let available = self.total_memory - *used;
            error!(
                "OOM: PID {} requested {} bytes, only {} bytes available ({} used / {} total)",
                pid, size, available, *used, self.total_memory
            );

            return Err(MemoryError::OutOfMemory {
                requested: size,
                available,
                used: *used,
                total: self.total_memory,
            });
        }

        // Allocate memory - all state updates are atomic
        let address = *next_addr;
        *next_addr += size;
        *used += size;

        let block = MemoryBlock {
            address,
            size,
            allocated: true,
            owner_pid: Some(pid),
        };

        blocks.insert(address, block);

        // Update per-process tracking
        let process_track = tracking.entry(pid).or_insert(ProcessMemoryTracking {
            current_bytes: 0,
            peak_bytes: 0,
            allocation_count: 0,
        });
        process_track.current_bytes += size;
        process_track.allocation_count += 1;
        if process_track.current_bytes > process_track.peak_bytes {
            process_track.peak_bytes = process_track.current_bytes;
        }

        // Log allocation with memory pressure warnings
        let used_val = *used;
        drop(tracking);
        drop(blocks);
        drop(next_addr);
        drop(used);

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
        let mut blocks = self.blocks.write();

        if let Some(block) = blocks.get_mut(&address) {
            if block.allocated {
                let size = block.size;
                let pid = block.owner_pid;
                block.allocated = false;

                let mut used = self.used_memory.write();
                *used = used.saturating_sub(size);

                // Update per-process tracking
                if let Some(pid) = pid {
                    let mut tracking = self.process_tracking.write();
                    if let Some(process_track) = tracking.get_mut(&pid) {
                        process_track.current_bytes = process_track.current_bytes.saturating_sub(size);
                    }
                }

                // Track deallocated blocks for GC
                let mut dealloc_count = self.deallocated_count.write();
                *dealloc_count += 1;

                info!(
                    "Deallocated {} bytes at 0x{:x} ({} bytes now available, {} deallocated blocks)",
                    size,
                    address,
                    self.total_memory - *used,
                    *dealloc_count
                );

                // Trigger GC if threshold reached
                let should_gc = *dealloc_count >= self.gc_threshold;
                drop(dealloc_count);
                drop(used);
                drop(blocks);

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
        let mut blocks = self.blocks.write();
        let mut freed_bytes = 0;
        let mut freed_count = 0;

        for (_, block) in blocks.iter_mut() {
            if block.allocated && block.owner_pid == Some(pid) {
                block.allocated = false;
                freed_bytes += block.size;
                freed_count += 1;
            }
        }

        if freed_bytes > 0 {
            let mut used = self.used_memory.write();
            *used = used.saturating_sub(freed_bytes);

            // Remove process tracking entry
            let mut tracking = self.process_tracking.write();
            tracking.remove(&pid);

            // Track deallocated blocks for GC
            let mut dealloc_count = self.deallocated_count.write();
            *dealloc_count += freed_count;

            info!(
                "Cleaned up {} bytes ({} blocks) from terminated PID {} ({} bytes now available, {} deallocated blocks)",
                freed_bytes,
                freed_count,
                pid,
                self.total_memory - *used,
                *dealloc_count
            );

            // Trigger GC if threshold reached
            let should_gc = *dealloc_count >= self.gc_threshold;
            drop(dealloc_count);
            drop(tracking);
            drop(used);
            drop(blocks);

            if should_gc {
                info!("GC threshold reached after process cleanup, running garbage collection...");
                self.collect();
            }
        } else {
            drop(blocks);
        }

        freed_bytes
    }

    /// Get memory statistics for a specific process
    pub fn process_memory(&self, pid: Pid) -> Size {
        let blocks = self.blocks.read();
        blocks
            .values()
            .filter(|b| b.allocated && b.owner_pid == Some(pid))
            .map(|b| b.size)
            .sum()
    }

    /// Get detailed process memory stats including peak and allocation count
    pub fn get_process_memory_details(&self, pid: Pid) -> (Size, Size, usize) {
        let tracking = self.process_tracking.read();
        if let Some(track) = tracking.get(&pid) {
            (track.current_bytes, track.peak_bytes, track.allocation_count)
        } else {
            (0, 0, 0)
        }
    }

    /// Get overall memory info: (total, used, available)
    pub fn info(&self) -> (Size, Size, Size) {
        let used = *self.used_memory.read();
        (self.total_memory, used, self.total_memory - used)
    }

    /// Get detailed memory statistics
    pub fn stats(&self) -> MemoryStats {
        let blocks = self.blocks.read();
        let used = *self.used_memory.read();

        let allocated_blocks = blocks.values().filter(|b| b.allocated).count();
        let fragmented_blocks = blocks.values().filter(|b| !b.allocated).count();

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
        let mut blocks = self.blocks.write();
        let initial_count = blocks.len();

        // Retain only allocated blocks
        blocks.retain(|_, block| block.allocated);

        let removed_count = initial_count - blocks.len();

        // Reset deallocated counter
        let mut dealloc_count = self.deallocated_count.write();
        *dealloc_count = 0;

        if removed_count > 0 {
            info!(
                "Garbage collection complete: removed {} deallocated blocks, {} blocks remain",
                removed_count,
                blocks.len()
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
        let dealloc_count = *self.deallocated_count.read();
        dealloc_count >= self.gc_threshold
    }

    /// Set GC threshold
    pub fn set_threshold(&mut self, _threshold: Size) {
        // Note: MemoryManager uses Arc internally, so this would need refactoring
        // to support mutable threshold changes. For now, threshold is set at construction.
    }

    /// Get list of allocations for a process
    pub fn process_allocations(&self, pid: Pid) -> Vec<MemoryBlock> {
        let blocks = self.blocks.read();
        blocks
            .values()
            .filter(|b| b.allocated && b.owner_pid == Some(pid))
            .cloned()
            .collect()
    }
    /// Check if an address is valid and allocated
    pub fn is_valid(&self, address: Address) -> bool {
        let blocks = self.blocks.read();
        blocks.get(&address).map(|b| b.allocated).unwrap_or(false)
    }

    /// Get the size of an allocated block
    pub fn block_size(&self, address: Address) -> Option<Size> {
        let blocks = self.blocks.read();
        blocks.get(&address).filter(|b| b.allocated).map(|b| b.size)
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
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
