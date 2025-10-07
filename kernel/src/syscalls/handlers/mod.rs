/*!
 * Syscall Handlers Module
 * Contains all syscall category handlers
 */

mod async_handler;
mod fd_handler;
mod fs_handler;
mod ipc_handler;
mod memory_handler;
mod mmap_handler;
mod network_handler;
mod process_handler;
mod scheduler_handler;
mod signal_handler;
mod system_handler;
mod time_handler;

pub use async_handler::{AsyncSyscallHandler, AsyncSyscallHandlerRegistry};
pub use fd_handler::FileDescriptorHandler;
pub use fs_handler::FileSystemHandler;
pub use ipc_handler::IpcHandler;
pub use memory_handler::MemoryHandler;
pub use mmap_handler::MmapHandler;
pub use network_handler::NetworkHandler;
pub use process_handler::ProcessHandler;
pub use scheduler_handler::SchedulerHandler;
pub use signal_handler::SignalHandler;
pub use system_handler::SystemHandler;
pub use time_handler::TimeHandler;
