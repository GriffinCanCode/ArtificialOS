/**
 * AI-OS Kernel Library
 * Core kernel functionality exposed as a library
 */

pub mod process;
pub mod memory;
pub mod ipc;
pub mod sandbox;
pub mod syscall;

pub use process::ProcessManager;
pub use memory::MemoryManager;
pub use ipc::IPCManager;
pub use sandbox::{SandboxManager, SandboxConfig, Capability, ResourceLimits};
pub use syscall::{SyscallExecutor, Syscall, SyscallResult};

