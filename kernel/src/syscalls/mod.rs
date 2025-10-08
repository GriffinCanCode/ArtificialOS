/*!
 * Syscalls Module
 * Modular system call implementation with native async traits (Rust 1.75+)
 *
 * ## Module Organization
 *
 * - **core**: Core execution infrastructure (executor, handlers, registry)
 * - **async**: Async execution layer with intelligent dispatch
 * - **impls**: Syscall implementations by category (fs, network, process, etc.)
 * - **timeout**: Timeout management for blocking operations
 * - **types**: Type definitions and serialization
 * - **iouring**: io_uring-style async completion
 * - **ipc**: Inter-process communication implementations
 * - **jit**: JIT compilation for hot paths
 */

// Core modules (organized by domain)
pub mod r#async;
pub mod core;
pub mod impls;
pub mod iouring;
pub mod ipc;
pub mod jit;
pub mod timeout;
pub mod traits;
pub mod types;
mod types_ext;

// Re-export public API from core
pub use core::{SyscallExecutorWithIpc, SyscallHandler, SyscallHandlerRegistry, SYSTEM_START};

// Re-export public API from impls
pub use impls::{FdManager, FileHandle, Socket, SocketManager, SocketStats};

// Re-export public API from async
pub use r#async::{AsyncExecutorStats, AsyncSyscallExecutor, SyscallClass};

// Re-export public API from timeout
pub use timeout::{SyscallTimeoutConfig, TimeoutError, TimeoutExecutor, TimeoutPolicy};

// Re-export public API from iouring
pub use iouring::{
    IoUringExecutor, IoUringManager, SyscallCompletionEntry, SyscallCompletionRing,
    SyscallCompletionStatus, SyscallOpType, SyscallSubmissionEntry,
};

// Re-export public API from jit
pub use jit::{JitManager, JitStats, SyscallPattern};

// Re-export public API from traits
pub use traits::*;

// Re-export public API from types
pub use types::{ProcessOutput, Syscall, SyscallError, SyscallResult, SystemInfo};

// Re-export ProcessMemoryStats from memory module
pub use crate::memory::ProcessMemoryStats;

// Re-export scheduler types for convenience
pub use crate::scheduler::{
    apply_priority_op, validate_priority, Policy as SchedulerPolicy, PriorityOp, TimeQuantum,
    DEFAULT_PRIORITY, MAX_PRIORITY, MIN_PRIORITY,
};
