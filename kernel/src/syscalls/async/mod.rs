/*!
 * Async Syscall Execution Layer
 *
 * Provides async execution capabilities with intelligent dispatch:
 * - Classification: Compile-time syscall classification
 * - Executor: Dual-mode execution (fast-path sync, slow-path async)
 * - I/O: True async file operations (tokio::fs)
 * - IPC: Native async IPC operations (flume async)
 * - Dispatcher: Adaptive selection between tokio::fs and io_uring
 */

pub mod classification;
pub mod dispatcher;
pub mod executor;
pub mod io;
pub mod ipc;

// Re-export commonly used types
pub use classification::SyscallClass;
pub use dispatcher::AdaptiveDispatcher;
pub use executor::{AsyncExecutorStats, AsyncSyscallExecutor};
pub use io::AsyncFileOps;
pub use ipc::AsyncIpcOps;
