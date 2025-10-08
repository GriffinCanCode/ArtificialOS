/*!
 * Core Memory Management
 * Types, traits, and allocation logic
 */

pub mod allocator;
pub mod free_list;
pub mod traits;
pub mod types;

// Re-export public types and traits
pub use free_list::{FreeBlock, SegregatedFreeList};
pub use traits::{Allocator, GarbageCollector, MemoryInfo, ProcessMemoryCleanup};
pub use types::{
    AllocationRequest, MemoryBlock, MemoryError, MemoryPressure, MemoryResult, MemoryStats,
    ProcessMemoryStats,
};

