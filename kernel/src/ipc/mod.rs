/*!
 * IPC Module
 * Inter-process communication: messages, pipes, and shared memory
 */

pub mod manager;
pub mod pipe;
pub mod shm;

// Re-export for convenience
pub use manager::IPCManager;
pub use pipe::{PipeError, PipeManager, PipeStats};
pub use shm::{ShmError, ShmManager, ShmPermission, ShmStats};
