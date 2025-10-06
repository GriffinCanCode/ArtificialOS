/*!
 * IPC Module
 * Inter-process communication: messages, pipes, and shared memory
 */

pub mod manager;
pub mod pipe;
pub mod shm;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use manager::IPCManager;
pub use pipe::{PipeError, PipeManager, PipeStats};
pub use shm::{ShmError, ShmManager, ShmPermission, ShmStats};
pub use traits::*;
pub use types::*;
