/**
 * AI-OS Kernel Library
 * Core kernel functionality exposed as a library
 */

pub mod process;
pub mod memory;
pub mod ipc;

pub use process::ProcessManager;
pub use memory::MemoryManager;
pub use ipc::IPCManager;

