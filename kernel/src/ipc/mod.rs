/*!
 * IPC Module
 * Inter-process communication: messages, pipes, queues, and shared memory
 */

pub mod core;
pub mod pipe;
pub mod queue;
pub mod shm;

// Re-export for convenience
pub use core::*;
pub use pipe::{PipeError, PipeManager, PipeStats};
pub use queue::{QueueManager, QueueMessage, QueueStats};
pub use shm::{ShmError, ShmManager, ShmPermission, ShmStats};
