/*!
 * Execution Module
 * Async, batch, and streaming syscall execution
 */

pub mod async_task;
pub mod batch;
pub mod streaming;

pub use async_task::{AsyncTaskManager, TaskStats, TaskStatus};
pub use batch::BatchExecutor;
pub use streaming::StreamingManager;

// Re-export io_uring types for execution layer
pub use crate::syscalls::{
    IoUringExecutor, IoUringManager, SyscallCompletionEntry,
    SyscallSubmissionEntry, SyscallOpType,
};
