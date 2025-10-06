/*!
 * IPC Traits
 * Inter-process communication abstractions
 */

use super::types::*;
use crate::core::types::{Pid, Size};

/// Message passing interface
pub trait MessageQueue: Send + Sync {
    /// Send a message to a process
    fn send(&self, from: Pid, to: Pid, data: Vec<u8>) -> IpcResult<()>;

    /// Receive a message for a process (blocking)
    fn receive(&self, pid: Pid) -> IpcResult<Message>;

    /// Try to receive a message (non-blocking)
    fn try_receive(&self, pid: Pid) -> IpcResult<Option<Message>>;

    /// Check if messages are available
    fn has_messages(&self, pid: Pid) -> bool;

    /// Clear all messages for a process
    fn clear(&self, pid: Pid) -> Size;
}

/// Pipe communication interface
pub trait PipeChannel: Send + Sync {
    /// Create a new pipe
    fn create(&self, reader_pid: Pid, writer_pid: Pid, capacity: Option<Size>)
        -> IpcResult<PipeId>;

    /// Write data to a pipe
    fn write(&self, pipe_id: PipeId, pid: Pid, data: &[u8]) -> IpcResult<Size>;

    /// Read data from a pipe
    fn read(&self, pipe_id: PipeId, pid: Pid, size: Size) -> IpcResult<Vec<u8>>;

    /// Close a pipe endpoint
    fn close(&self, pipe_id: PipeId, pid: Pid) -> IpcResult<()>;

    /// Destroy a pipe
    fn destroy(&self, pipe_id: PipeId) -> IpcResult<()>;

    /// Get pipe statistics
    fn stats(&self, pipe_id: PipeId) -> IpcResult<crate::ipc::pipe::PipeStats>;
}

/// Shared memory interface
pub trait SharedMemory: Send + Sync {
    /// Create a shared memory segment
    fn create(&self, size: Size, owner_pid: Pid) -> IpcResult<ShmId>;

    /// Attach to a shared memory segment
    fn attach(&self, segment_id: ShmId, pid: Pid, read_only: bool) -> IpcResult<()>;

    /// Detach from a shared memory segment
    fn detach(&self, segment_id: ShmId, pid: Pid) -> IpcResult<()>;

    /// Write to shared memory
    fn write(&self, segment_id: ShmId, pid: Pid, offset: Size, data: &[u8]) -> IpcResult<()>;

    /// Read from shared memory
    fn read(&self, segment_id: ShmId, pid: Pid, offset: Size, size: Size) -> IpcResult<Vec<u8>>;

    /// Destroy a shared memory segment
    fn destroy(&self, segment_id: ShmId, pid: Pid) -> IpcResult<()>;

    /// Get shared memory statistics
    fn stats(&self, segment_id: ShmId) -> IpcResult<crate::ipc::shm::ShmStats>;
}

/// IPC cleanup interface
pub trait IpcCleanup: Send + Sync {
    /// Clean up all IPC resources for a process
    fn cleanup_process(&self, pid: Pid) -> Size;

    /// Get global memory usage
    fn global_memory_usage(&self) -> Size;
}

/// Async queue interface
pub trait AsyncQueue: Send + Sync {
    /// Create a new async queue
    fn create(&self, owner_pid: Pid, queue_type: super::types::QueueType, capacity: Option<Size>) -> IpcResult<super::types::QueueId>;

    /// Send message to queue
    fn send(&self, queue_id: super::types::QueueId, from_pid: Pid, data: Vec<u8>, priority: Option<u8>) -> IpcResult<()>;

    /// Receive message from queue (non-blocking)
    fn receive(&self, queue_id: super::types::QueueId, pid: Pid) -> IpcResult<Option<crate::ipc::queue::QueueMessage>>;

    /// Subscribe to PubSub queue
    fn subscribe(&self, queue_id: super::types::QueueId, pid: Pid) -> IpcResult<()>;

    /// Unsubscribe from PubSub queue
    fn unsubscribe(&self, queue_id: super::types::QueueId, pid: Pid) -> IpcResult<()>;

    /// Close queue
    fn close(&self, queue_id: super::types::QueueId, pid: Pid) -> IpcResult<()>;

    /// Destroy queue
    fn destroy(&self, queue_id: super::types::QueueId, pid: Pid) -> IpcResult<()>;

    /// Get queue statistics
    fn stats(&self, queue_id: super::types::QueueId) -> IpcResult<crate::ipc::queue::QueueStats>;
}

/// Combined IPC interface
pub trait IpcManager: MessageQueue + IpcCleanup + Send + Sync + Clone {
    /// Get pipe manager
    fn pipes(&self) -> &dyn PipeChannel;

    /// Get shared memory manager
    fn shm(&self) -> &dyn SharedMemory;

    /// Get async queue manager
    fn queues(&self) -> &dyn AsyncQueue;
}
