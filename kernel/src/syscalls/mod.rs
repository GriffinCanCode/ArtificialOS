/*!
 * Syscalls Module
 * Modular system call implementation
 */

mod executor;
mod fs;
mod ipc;
mod memory;
mod process;
mod scheduler;
mod signal;
mod system;
mod time;
mod types;

// Re-export public API
pub use executor::SyscallExecutor;
pub use types::{ProcessMemoryStats, ProcessOutput, Syscall, SyscallResult, SystemInfo};
