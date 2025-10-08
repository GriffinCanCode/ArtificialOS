/*!
 * IPC Utilities Module
 * Specialized IPC utilities and advanced features
 */

pub mod lockfree_ring; // Lock-free SPSC ring buffers for IPC hot paths
pub mod mmap; // Memory-mapped files
pub mod timeout; // Timeout-aware IPC operations

// Re-export for convenience
pub use lockfree_ring::{LockFreeByteRing, LockFreeRing};
pub use mmap::{MapFlags, MmapEntry, MmapId, MmapManager, ProtFlags};
pub use timeout::{TimeoutPipeOps, TimeoutQueueOps};
