/*!
 * AI-OS Kernel Library
 * Core kernel functionality exposed as a library
 */

pub mod grpc_server;
pub mod ipc;
pub mod memory;
pub mod process;
pub mod sandbox;
pub mod syscall;

pub use grpc_server::start_grpc_server;
pub use ipc::IPCManager;
pub use memory::{MemoryError, MemoryManager, MemoryStats};
pub use process::ProcessManager;
pub use sandbox::{Capability, ResourceLimits, SandboxConfig, SandboxManager};
pub use syscall::{Syscall, SyscallExecutor, SyscallResult};
