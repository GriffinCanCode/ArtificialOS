/*!
 * Syscalls Module
 * Modular system call implementation
 */

mod executor;
mod fd;
mod fs;
mod ipc;
mod memory;
mod network;
mod process;
mod scheduler;
mod signal;
mod system;
mod time;
pub mod traits;
pub mod types;
mod vfs_adapter;

// Re-export public API
pub use executor::SyscallExecutor;
pub use traits::*;
pub use types::{ProcessOutput, Syscall, SyscallError, SyscallResult, SystemInfo};

// Re-export ProcessMemoryStats from memory module
pub use crate::memory::ProcessMemoryStats;
