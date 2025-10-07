/*!
 * Memory Management
 *
 * High-performance memory allocator with graceful OOM handling and address recycling.
 *
 * ## Allocation Performance
 *
 * Uses a **segregated free list** allocator for optimal performance:
 * - **Small blocks** (<4KB): O(1) lookup via power-of-2 bucketing
 *   - 12 buckets: 64B, 128B, 256B, 512B, 1KB, 2KB, 4KB
 *   - Most allocations fall into this category
 *
 * - **Medium blocks** (4KB-64KB): O(1) lookup via 4KB increment bucketing
 *   - 15 buckets: 8KB, 12KB, 16KB, ..., 64KB
 *   - Common for process stacks and buffers
 *
 * - **Large blocks** (>64KB): O(log n) lookup via BTreeMap
 *   - Efficient range queries for best-fit allocation
 *
 * **Old implementation**: O(n) linear scan of entire free list
 * **New implementation**: O(1) for small/medium, O(log n) for large
 *
 * ## Features
 *
 * - **Address recycling**: Deallocated memory is immediately available for reuse
 * - **Block splitting**: Larger blocks are split when smaller allocations are requested
 * - **Coalescing**: Adjacent free blocks are merged to reduce fragmentation
 * - **Memory pressure tracking**: Warns at 80%, critical at 95%
 * - **Garbage collection**: Automatic cleanup of deallocated block metadata
 * - **Per-process tracking**: Monitor peak usage and allocation counts
 */

use super::traits::{Allocator, GarbageCollector, MemoryInfo, ProcessMemoryCleanup};
use super::types::{MemoryBlock, MemoryError, MemoryPressure, MemoryResult, MemoryStats};
use crate::core::types::{Address, Pid, Size};
use ahash::RandomState;
use dashmap::DashMap;
use log::{error, info, warn};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Per-process memory tracking
#[derive(Debug, Clone)]
struct ProcessMemoryTracking {
    current_bytes: Size,
    peak_bytes: Size,
    allocation_count: usize,
}

/// Free block for address recycling
#[derive(Debug, Clone)]
struct FreeBlock {
    address: Address,
    size: Size,
}

/// Size classes for segregated free lists
/// Modern memory allocators use segregated lists for O(1) lookup
const SMALL_BLOCK_MAX: Size = 4 * 1024; // 4KB
const MEDIUM_BLOCK_MAX: Size = 64 * 1024; // 64KB

/// Segregated free list for efficient memory allocation
/// - Small blocks (<4KB): O(1) best-fit within size class
/// - Medium blocks (4KB-64KB): O(1) best-fit within size class
/// - Large blocks (>64KB): O(log n) using BTreeMap
#[derive(Debug)]
struct SegregatedFreeList {
    /// Small allocations: <4KB - most common case
    /// Bucketed by powers of 2 for cache-friendly access
    small_blocks: Vec<Vec<FreeBlock>>, // 12 buckets: 64, 128, 256, ..., 2048, 4096 bytes

    /// Medium allocations: 4KB-64KB
    /// Bucketed by 4KB increments
    medium_blocks: Vec<Vec<FreeBlock>>, // 15 buckets: 8KB, 12KB, 16KB, ..., 64KB

    /// Large allocations: >64KB
    /// BTreeMap for O(log n) lookup by size
    large_blocks: BTreeMap<Size, Vec<FreeBlock>>,
}

impl SegregatedFreeList {
    fn new() -> Self {
        Self {
            small_blocks: vec![Vec::new(); 12],  // 64B to 4KB (powers of 2)
            medium_blocks: vec![Vec::new(); 15], // 8KB to 64KB (4KB increments)
            large_blocks: BTreeMap::new(),
        }
    }

    /// Get the bucket index for small blocks (powers of 2)
    fn small_bucket_index(size: Size) -> Option<usize> {
        if size > SMALL_BLOCK_MAX {
            return None;
        }
        // Find the smallest power of 2 >= size, starting from 64
        let log2 = (size.max(64) - 1).next_power_of_two().trailing_zeros();
        let idx = (log2 as usize).saturating_sub(6); // 64 = 2^6
        if idx < 12 {
            Some(idx)
        } else {
            None
        }
    }

    /// Get the bucket index for medium blocks (4KB increments)
    fn medium_bucket_index(size: Size) -> Option<usize> {
        if size <= SMALL_BLOCK_MAX || size > MEDIUM_BLOCK_MAX {
            return None;
        }
        // Round up to nearest 4KB increment
        let idx = ((size + 4095) / 4096).saturating_sub(2);
        if idx < 15 {
            Some(idx)
        } else {
            None
        }
    }

    /// Insert a free block into the appropriate list
    fn insert(&mut self, block: FreeBlock) {
        if let Some(idx) = Self::small_bucket_index(block.size) {
            self.small_blocks[idx].push(block);
        } else if let Some(idx) = Self::medium_bucket_index(block.size) {
            self.medium_blocks[idx].push(block);
        } else {
            // Large block - use BTreeMap for O(log n) access
            self.large_blocks
                .entry(block.size)
                .or_insert_with(Vec::new)
                .push(block);
        }
    }

    /// Find and remove a suitable block for the requested size
    /// Returns the block and whether it was split
    fn find_best_fit(&mut self, size: Size) -> Option<FreeBlock> {
        // Try small blocks first - O(1) lookup
        if size <= SMALL_BLOCK_MAX {
            if let Some(start_idx) = Self::small_bucket_index(size) {
                // Search from the appropriate bucket upwards
                for idx in start_idx..self.small_blocks.len() {
                    if !self.small_blocks[idx].is_empty() {
                        // Find smallest block >= size in this bucket
                        let best_idx = self.small_blocks[idx]
                            .iter()
                            .enumerate()
                            .filter(|(_, b)| b.size >= size)
                            .min_by_key(|(_, b)| b.size)
                            .map(|(i, _)| i);

                        if let Some(i) = best_idx {
                            return Some(self.small_blocks[idx].remove(i));
                        }
                    }
                }
            }
        }

        // Try medium blocks - O(1) lookup
        if size <= MEDIUM_BLOCK_MAX {
            if let Some(start_idx) = Self::medium_bucket_index(size) {
                for idx in start_idx..self.medium_blocks.len() {
                    if !self.medium_blocks[idx].is_empty() {
                        let best_idx = self.medium_blocks[idx]
                            .iter()
                            .enumerate()
                            .filter(|(_, b)| b.size >= size)
                            .min_by_key(|(_, b)| b.size)
                            .map(|(i, _)| i);

                        if let Some(i) = best_idx {
                            return Some(self.medium_blocks[idx].remove(i));
                        }
                    }
                }
            }
        }

        // Try large blocks - O(log n) lookup using BTreeMap range query
        // First find the size, then mutate to avoid borrow conflicts
        let found_size = self.large_blocks
            .range(size..)
            .find(|(_, blocks)| !blocks.is_empty())
            .map(|(k, _)| *k);

        if let Some(found_size) = found_size {
            let blocks = self.large_blocks.get_mut(&found_size).unwrap();
            let block = blocks.remove(0);
            // Clean up empty size classes
            if blocks.is_empty() {
                self.large_blocks.remove(&found_size);
            }
            return Some(block);
        }

        None
    }

    /// Get the total number of free blocks across all lists
    fn len(&self) -> usize {
        self.small_blocks.iter().map(|v| v.len()).sum::<usize>()
            + self.medium_blocks.iter().map(|v| v.len()).sum::<usize>()
            + self.large_blocks.values().map(|v| v.len()).sum::<usize>()
    }

    /// Get all blocks as a sorted vector (for coalescing)
    fn get_all_sorted(&mut self) -> Vec<FreeBlock> {
        let mut all_blocks = Vec::new();

        for bucket in self.small_blocks.iter_mut() {
            all_blocks.append(bucket);
        }
        for bucket in self.medium_blocks.iter_mut() {
            all_blocks.append(bucket);
        }
        for blocks in self.large_blocks.values_mut() {
            all_blocks.append(blocks);
        }

        self.large_blocks.clear();

        all_blocks.sort_by_key(|block| block.address);
        all_blocks
    }

    /// Reinsert blocks after coalescing
    fn reinsert_all(&mut self, blocks: Vec<FreeBlock>) {
        for block in blocks {
            self.insert(block);
        }
    }
}

pub struct MemoryManager {
    blocks: Arc<DashMap<Address, MemoryBlock, RandomState>>,
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
    process_tracking: Arc<DashMap<Pid, ProcessMemoryTracking, RandomState>>,
    // Memory storage - maps addresses to their byte contents
    memory_storage: Arc<DashMap<Address, Vec<u8>, RandomState>>,
    // Segregated free list for O(1) small/medium and O(log n) large block allocation
    free_list: Arc<Mutex<SegregatedFreeList>>,
}

impl MemoryManager {
    pub fn new() -> Self {
        let total = 1024 * 1024 * 1024; // 1GB simulated memory
        info!(
            "Memory manager initialized with {} bytes and segregated free list allocator (O(1) small/medium, O(log n) large)",
            total
        );
        Self {
            // Use 128 shards for blocks - high contention data structure (default is 64)
            // More shards = better concurrent access performance. Using ahash hasher.
            blocks: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(0, RandomState::new(), 128)),
            next_address: Arc::new(AtomicU64::new(0)),
            total_memory: total,
            used_memory: Arc::new(AtomicU64::new(0)),
            warning_threshold: 0.80,
            critical_threshold: 0.95,
            gc_threshold: 1000,
            deallocated_count: Arc::new(AtomicU64::new(0)),
            // Use 64 shards for process tracking (moderate contention). Using ahash hasher.
            process_tracking: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(0, RandomState::new(), 64)),
            // Use 128 shards for memory storage - high I/O contention. Using ahash hasher.
            memory_storage: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(0, RandomState::new(), 128)),
            free_list: Arc::new(Mutex::new(SegregatedFreeList::new())),
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
            let mut free_list = self.free_list.lock().unwrap();

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

        // Update per-process tracking using alter() for atomic operation
        self.process_tracking.alter(&pid, |_, mut track| {
            track.current_bytes += size;
            track.allocation_count += 1;
            if track.current_bytes > track.peak_bytes {
                track.peak_bytes = track.current_bytes;
            }
            track
        });

        // Log allocation with memory pressure warnings
        let used_val = used as usize + size;

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

                // Add to segregated free list for address recycling
                {
                    let mut free_list = self.free_list.lock().unwrap();
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

    /// Coalesce adjacent free blocks to reduce fragmentation
    /// Works with segregated free lists by temporarily extracting all blocks
    fn coalesce_free_blocks(free_list: &mut SegregatedFreeList) {
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
                let mut free_list = self.free_list.lock().unwrap();
                for block in freed_blocks {
                    free_list.insert(block);
                }
                // Always coalesce after large batch frees
                Self::coalesce_free_blocks(&mut free_list);
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

    /// Garbage collect deallocated memory blocks
    /// Removes deallocated blocks from the HashMap to prevent unbounded growth
    /// Note: Free blocks remain in the segregated free list for address recycling
    pub fn collect(&self) -> Size {
        let initial_count = self.blocks.len();

        // Collect addresses of deallocated blocks before removing them
        let deallocated_addrs: Vec<Address> = self
            .blocks
            .iter()
            .filter(|entry| !entry.value().allocated)
            .map(|entry| *entry.key())
            .collect();

        // Remove deallocated blocks from the blocks map
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

        // Note: We intentionally keep blocks in the segregated free list for address recycling
        // The segregated free list allows these addresses to be reused in O(1) or O(log n) time

        if removed_count > 0 {
            // Shrink DashMap capacity after bulk deletion to reclaim memory
            self.blocks.shrink_to_fit();
            self.memory_storage.shrink_to_fit();

            let free_list_size = self.free_list.lock().unwrap().len();
            info!(
                "Garbage collection complete: removed {} deallocated blocks and their storage, {} blocks remain, {} blocks in segregated free list for O(1)/O(log n) recycling (maps shrunk to fit)",
                removed_count,
                initial_count - removed_count,
                free_list_size
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
        dealloc_count >= self.gc_threshold as u64
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
        self.blocks
            .get(&address)
            .map(|entry| entry.value().allocated)
            .unwrap_or(false)
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

            // Get or create storage for this block using alter() for atomic operation
            self.memory_storage.alter(&base_addr, |_, mut block_data| {
                // Ensure block_data is large enough
                if block_data.len() < block_size {
                    block_data.resize(block_size, 0u8);
                }
                // Write data at the offset
                let end = offset + data.len();
                block_data[offset..end].copy_from_slice(data);
                block_data
            });

            info!(
                "Wrote {} bytes to address 0x{:x} (offset {} in block at 0x{:x})",
                data.len(),
                address,
                offset,
                base_addr
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
            free_list: Arc::clone(&self.free_list),
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
