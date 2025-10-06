/*!
 * AgentOS Kernel Library
 * Core kernel functionality exposed as a library
 */

pub mod errors;
pub mod executor;
pub mod grpc_server;
pub mod ipc;
pub mod limits;
pub mod memory;
pub mod pipe;
pub mod process;
pub mod sandbox;
pub mod scheduler;
pub mod shm;
pub mod syscall;

// Re-exports
pub use errors::*;
pub use executor::{ExecutionConfig, ProcessExecutor};
pub use grpc_server::start_grpc_server;
pub use ipc::IPCManager;
pub use limits::{LimitManager, Limits};
pub use memory::{MemoryManager, MemoryStats};
pub use pipe::{PipeError, PipeManager, PipeStats};
pub use process::{Process, ProcessManager, ProcessManagerBuilder, ProcessState};
pub use sandbox::{Capability, ResourceLimits, SandboxConfig, SandboxManager};
pub use scheduler::{Policy, ProcessStats, Scheduler, Stats as SchedulerStats};
pub use shm::{ShmError, ShmManager, ShmPermission, ShmStats};
pub use syscall::{Syscall, SyscallExecutor, SyscallResult};
