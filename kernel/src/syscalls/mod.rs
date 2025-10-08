/*!
 * Syscalls Module
 * Modular system call implementation
 */

mod executor;
pub mod fd;
mod fs;
mod handler;
mod handlers;
pub mod iouring; // io_uring-style async syscall completion
mod ipc;
pub mod jit; // JIT compilation for hot syscall paths
mod memory;
mod mmap;
mod network;
mod process;
mod scheduler;
mod signals;
mod system;
mod time;
pub mod traits;
pub mod types;
mod types_ext; // Syscall type extensions for tracing
mod vfs_adapter;

// Re-export public API
pub use executor::SyscallExecutor;
pub use fd::FdManager;
pub use handler::{SyscallHandler, SyscallHandlerRegistry};
pub use iouring::{
    IoUringExecutor, IoUringManager, SyscallCompletionEntry, SyscallCompletionRing,
    SyscallCompletionStatus, SyscallOpType, SyscallSubmissionEntry,
};
pub use jit::{JitManager, JitStats, SyscallPattern};
pub use network::{SocketManager, SocketStats};
pub use traits::*;
pub use types::{ProcessOutput, Syscall, SyscallError, SyscallResult, SystemInfo};

// Re-export ProcessMemoryStats from memory module
pub use crate::memory::ProcessMemoryStats;

// Re-export scheduler types for convenience
pub use crate::scheduler::{
    apply_priority_op, validate_priority, Policy as SchedulerPolicy, PriorityOp, TimeQuantum,
    DEFAULT_PRIORITY, MAX_PRIORITY, MIN_PRIORITY,
};
