/*!
 * IPC Syscalls
 * Inter-process communication (pipes, shared memory, and queues)
 */

mod pipe;
mod queue;
mod shm;

// Re-export for backward compatibility
pub use pipe::*;
pub use queue::*;
pub use shm::*;
