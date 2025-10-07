/*!
 * IPC Module
 * Inter-process communication: messages, pipes, queues, and shared memory
 */

pub mod core;
pub mod mmap;
pub mod pipe;
pub mod queue;
pub mod shm;
pub mod zerocopy; // Zero-copy IPC with io_uring-inspired design

// Re-export for convenience
pub use core::*;
pub use mmap::{MapFlags, MmapEntry, MmapId, MmapManager, ProtFlags};
pub use pipe::{PipeError, PipeManager, PipeStats};
pub use queue::{QueueManager, QueueMessage, QueueStats};
pub use shm::{ShmError, ShmManager, ShmPermission, ShmStats};
pub use zerocopy::{ZeroCopyIpc, ZeroCopyRing, ZeroCopyStats};
