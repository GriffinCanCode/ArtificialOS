/*!
 * Memory Traits
 * Memory management abstractions
 */

use super::types::*;
use crate::core::types::{Pid, Address, Size};

/// Memory allocator interface
pub trait Allocator: Send + Sync {
    /// Allocate memory for a process
    fn allocate(&self, size: Size, pid: Pid) -> MemoryResult<Address>;

    /// Deallocate memory at an address
    fn deallocate(&self, address: Address) -> MemoryResult<()>;

    /// Reallocate memory with a new size
    fn reallocate(&self, address: Address, _new_size: Size) -> MemoryResult<Address> {
        // Default implementation: not supported - return error
        Err(MemoryError::InvalidAddress(address))
    }

    /// Check if an address is valid and allocated
    fn is_valid(&self, address: Address) -> bool;

    /// Get the size of an allocated block
    fn block_size(&self, address: Address) -> Option<Size>;
}

/// Memory statistics provider
pub trait MemoryInfo: Send + Sync {
    /// Get overall memory statistics
    fn stats(&self) -> MemoryStats;

    /// Get memory info as (total, used, available)
    fn info(&self) -> (Size, Size, Size);

    /// Get memory usage for a specific process
    fn process_memory(&self, pid: Pid) -> Size;

    /// Get memory pressure level
    fn pressure(&self) -> MemoryPressure {
        self.stats().memory_pressure()
    }
}

/// Garbage collection interface
pub trait GarbageCollector: Send + Sync {
    /// Run garbage collection
    fn collect(&self) -> Size;

    /// Force immediate garbage collection
    fn force_collect(&self) -> Size;

    /// Check if GC should run
    fn should_collect(&self) -> bool;

    /// Set GC threshold
    fn set_threshold(&mut self, threshold: Size);
}

/// Process memory cleanup
pub trait ProcessMemoryCleanup: Send + Sync {
    /// Free all memory allocated to a process
    fn free_process_memory(&self, pid: Pid) -> Size;

    /// Get list of allocations for a process
    fn process_allocations(&self, pid: Pid) -> Vec<MemoryBlock>;
}

/// Memory manager trait combining all interfaces
pub trait MemoryManager:
    Allocator + MemoryInfo + GarbageCollector + ProcessMemoryCleanup + Clone + Send + Sync
{
}

/// Implement MemoryManager for types that implement all required traits
impl<T> MemoryManager for T where
    T: Allocator + MemoryInfo + GarbageCollector + ProcessMemoryCleanup + Clone + Send + Sync
{
}
