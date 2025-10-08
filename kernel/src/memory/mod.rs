/*!
 * Memory Module
 * Memory management and allocation
 *
 * Note: SIMD operations have been moved to `crate::core::simd`
 */

pub mod gc;
pub mod manager;

// Re-export for convenience
pub use gc::{GcStats, GcStrategy, GlobalGarbageCollector};
pub use manager::{
    AllocationRequest, Allocator, GarbageCollector, MemoryBlock, MemoryError, MemoryGuardExt,
    MemoryInfo, MemoryManager, MemoryPressure, MemoryResult, MemoryStats, ProcessMemoryCleanup,
    ProcessMemoryStats,
};
