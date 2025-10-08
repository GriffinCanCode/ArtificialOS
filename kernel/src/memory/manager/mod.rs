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
 * ## Features
 *
 * - **Address recycling**: Deallocated memory is immediately available for reuse
 * - **Block splitting**: Larger blocks are split when smaller allocations are requested
 * - **Coalescing**: Adjacent free blocks are merged to reduce fragmentation
 * - **Memory pressure tracking**: Warns at 80%, critical at 95%
 * - **Garbage collection**: Automatic cleanup of deallocated block metadata
 * - **Per-process tracking**: Monitor peak usage and allocation counts
 */

// Organized submodules
mod core;
mod extensions;
mod gc;
mod process;
mod storage;

// Re-export public types, traits, and extensions
pub use core::{
    AllocationRequest, Allocator, GarbageCollector, MemoryBlock, MemoryError, MemoryInfo,
    MemoryPressure, MemoryResult, MemoryStats, ProcessMemoryCleanup, ProcessMemoryStats,
};
pub use extensions::MemoryGuardExt;

use crate::core::memory::CowMemory;
use crate::core::sync::lockfree::FlatCombiningCounter;
use crate::core::types::{Address, Pid, Size};
use crate::core::{ShardManager, WorkloadProfile};
use crate::monitoring::Collector;
use ahash::RandomState;
use core::SegregatedFreeList;
use dashmap::DashMap;
use log::info;
use process::ProcessMemoryTracking;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

/// Memory manager
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic counters
#[repr(C, align(64))]
pub struct MemoryManager {
    pub(super) blocks: Arc<DashMap<Address, MemoryBlock, RandomState>>,
    pub(super) next_address: Arc<AtomicU64>,
    pub(super) total_memory: Size,
    pub(super) used_memory: Arc<FlatCombiningCounter>,
    // Memory pressure thresholds (percentage)
    pub(super) warning_threshold: f64,  // 80%
    pub(super) critical_threshold: f64, // 95%
    // Garbage collection threshold - run GC when this many deallocated blocks accumulate
    pub(super) gc_threshold: Size, // 1000 blocks
    pub(super) deallocated_count: Arc<FlatCombiningCounter>,
    // Per-process memory tracking (for peak_bytes and allocation_count)
    pub(super) process_tracking: Arc<DashMap<Pid, ProcessMemoryTracking, RandomState>>,
    // Memory storage - maps addresses to CoW memory
    pub(super) memory_storage: Arc<DashMap<Address, CowMemory, RandomState>>,
    // Segregated free list for O(1) small/medium and O(log n) large block allocation
    pub(super) free_list: Arc<Mutex<SegregatedFreeList>>,
    // Observability collector for event streaming
    collector: Option<Arc<Collector>>,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self::with_capacity(crate::core::limits::DEFAULT_MEMORY_POOL)
    }

    /// Create memory manager with custom capacity (useful for testing)
    pub fn with_capacity(total: Size) -> Self {
        info!(
            "Memory manager initialized with {} bytes and segregated free list allocator (O(1) small/medium, O(log n) large)",
            total
        );
        Self {
            // CPU-topology-aware shard counts for optimal concurrent performance
            blocks: Arc::new(
                DashMap::with_capacity_and_hasher_and_shard_amount(
                    0,
                    RandomState::new(),
                    ShardManager::shards(WorkloadProfile::HighContention), // memory blocks: heavy concurrent access
                )
                .into(),
            ),
            next_address: Arc::new(AtomicU64::new(0).into()),
            total_memory: total,
            used_memory: Arc::new(FlatCombiningCounter::new(0).into()),
            warning_threshold: 0.80,
            critical_threshold: 0.95,
            gc_threshold: 1000,
            deallocated_count: Arc::new(FlatCombiningCounter::new(0).into()),
            process_tracking: Arc::new(
                DashMap::with_capacity_and_hasher_and_shard_amount(
                    0,
                    RandomState::new(),
                    ShardManager::shards(WorkloadProfile::MediumContention), // per-process tracking: moderate access
                )
                .into(),
            ),
            memory_storage: Arc::new(
                DashMap::with_capacity_and_hasher_and_shard_amount(
                    0,
                    RandomState::new(),
                    ShardManager::shards(WorkloadProfile::HighContention), // storage map: high I/O contention
                )
                .into(),
            ),
            free_list: Arc::new(Mutex::new(SegregatedFreeList::new().into())),
            collector: None,
        }
    }

    /// Add observability collector
    pub fn with_collector(mut self, collector: Arc<Collector>) -> Self {
        self.collector = Some(collector);
        self
    }

    /// Set collector after construction
    pub fn set_collector(&mut self, collector: Arc<Collector>) {
        self.collector = Some(collector);
    }

    /// Get collector reference
    pub fn collector(&self) -> Option<Arc<Collector>> {
        self.collector.clone()
    }

    /// Fork process memory using CoW semantics
    pub fn fork_memory(&self, parent_pid: Pid, child_pid: Pid) {
        let parent_blocks: Vec<_> = self
            .blocks
            .iter()
            .filter(|e| e.value().allocated && e.value().owner_pid == Some(parent_pid))
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect();

        for (addr, mut block) in parent_blocks {
            if let Some(parent_cow) = self.memory_storage.get(&addr) {
                let child_cow = parent_cow.clone_cow();

                block.owner_pid = Some(child_pid);
                let child_addr = self
                    .next_address
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    as Address;
                block.address = child_addr;

                self.blocks.insert(child_addr, block);
                self.memory_storage.insert(child_addr, child_cow);
            }
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
            collector: self.collector.as_ref().map(Arc::clone),
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
