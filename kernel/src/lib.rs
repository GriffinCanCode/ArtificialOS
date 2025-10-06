/*!
 * AgentOS Kernel Library
 * Core kernel functionality exposed as a library
 */

// Module organization
pub mod api;
pub mod core;
pub mod ipc;
pub mod memory;
pub mod process;
pub mod security;
pub mod syscalls;

// Re-exports for backwards compatibility
// API
pub use api::start_grpc_server;

// Core
pub use core::*;

// IPC
pub use ipc::{IPCManager, PipeError, PipeManager, PipeStats, ShmError, ShmManager, ShmPermission, ShmStats};

// Memory
pub use memory::{MemoryManager, MemoryStats};

// Process
pub use process::{ExecutionConfig, Policy, Process, ProcessExecutor, ProcessManager, ProcessManagerBuilder, ProcessState, ProcessStats, Scheduler, SchedulerStats};

// Security
pub use security::{Capability, LimitManager, Limits, ResourceLimits, SandboxConfig, SandboxManager};

// Syscalls
pub use syscalls::{Syscall, SyscallExecutor, SyscallResult};
