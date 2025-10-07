/*!
 * IPC Syscalls
 * Inter-process communication operations (pipes, shared memory, queues, mmap)
 */

use crate::core::types::{Pid, Size};
use serde::{Deserialize, Serialize};

/// IPC operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "syscall")]
#[non_exhaustive]
pub enum IpcSyscall {
    // ========================================================================
    // Pipes
    // ========================================================================

    /// Create pipe for IPC
    CreatePipe {
        /// Reader process ID
        reader_pid: Pid,
        /// Writer process ID
        writer_pid: Pid,
        /// Optional capacity (bytes)
        capacity: Option<Size>,
    },

    /// Write data to pipe
    WritePipe {
        /// Pipe ID
        pipe_id: Pid,
        /// Data to write
        data: Vec<u8>,
    },

    /// Read data from pipe
    ReadPipe {
        /// Pipe ID
        pipe_id: Pid,
        /// Number of bytes to read
        size: Size,
    },

    /// Close pipe end
    ClosePipe {
        /// Pipe ID
        pipe_id: Pid,
    },

    /// Destroy pipe completely
    DestroyPipe {
        /// Pipe ID
        pipe_id: Pid,
    },

    /// Get pipe statistics
    PipeStats {
        /// Pipe ID
        pipe_id: Pid,
    },

    // ========================================================================
    // Shared Memory
    // ========================================================================

    /// Create shared memory segment
    CreateShm {
        /// Size in bytes
        size: Size,
    },

    /// Attach to shared memory segment
    AttachShm {
        /// Segment ID
        segment_id: Pid,
        /// Read-only access
        #[serde(default)]
        read_only: bool,
    },

    /// Detach from shared memory segment
    DetachShm {
        /// Segment ID
        segment_id: Pid,
    },

    /// Write to shared memory
    WriteShm {
        /// Segment ID
        segment_id: Pid,
        /// Offset in bytes
        #[serde(default)]
        offset: usize,
        /// Data to write
        data: Vec<u8>,
    },

    /// Read from shared memory
    ReadShm {
        /// Segment ID
        segment_id: Pid,
        /// Offset in bytes
        #[serde(default)]
        offset: usize,
        /// Number of bytes to read
        size: Size,
    },

    /// Destroy shared memory segment
    DestroyShm {
        /// Segment ID
        segment_id: Pid,
    },

    /// Get shared memory statistics
    ShmStats {
        /// Segment ID
        segment_id: Pid,
    },

    // ========================================================================
    // Memory-Mapped Files
    // ========================================================================

    /// Memory-map a file
    Mmap {
        /// File path to map
        path: String,
        /// Offset in file
        #[serde(default)]
        offset: usize,
        /// Length to map
        length: Size,
        /// Protection flags (read, write, exec as bit flags)
        prot: u8,
        /// Shared (1) or Private (0) mapping
        #[serde(default)]
        shared: bool,
    },

    /// Read from memory-mapped region
    MmapRead {
        /// Mapping ID
        mmap_id: u32,
        /// Offset in mapping
        #[serde(default)]
        offset: usize,
        /// Length to read
        length: Size,
    },

    /// Write to memory-mapped region
    MmapWrite {
        /// Mapping ID
        mmap_id: u32,
        /// Offset in mapping
        #[serde(default)]
        offset: usize,
        /// Data to write
        data: Vec<u8>,
    },

    /// Synchronize mmap to file
    Msync {
        /// Mapping ID
        mmap_id: u32,
    },

    /// Unmap a memory-mapped region
    Munmap {
        /// Mapping ID
        mmap_id: u32,
    },

    /// Get mmap statistics
    MmapStats {
        /// Mapping ID
        mmap_id: u32,
    },

    // ========================================================================
    // Message Queues
    // ========================================================================

    /// Create message queue
    CreateQueue {
        /// Queue type: "fifo", "priority", or "pubsub"
        queue_type: String,
        /// Optional capacity (messages)
        capacity: Option<Size>,
    },

    /// Send message to queue
    SendQueue {
        /// Queue ID
        queue_id: Pid,
        /// Message data
        data: Vec<u8>,
        /// Optional priority (for priority queues)
        priority: Option<u8>,
    },

    /// Receive message from queue
    ReceiveQueue {
        /// Queue ID
        queue_id: Pid,
    },

    /// Subscribe to pubsub queue
    SubscribeQueue {
        /// Queue ID
        queue_id: Pid,
    },

    /// Unsubscribe from pubsub queue
    UnsubscribeQueue {
        /// Queue ID
        queue_id: Pid,
    },

    /// Close queue connection
    CloseQueue {
        /// Queue ID
        queue_id: Pid,
    },

    /// Destroy queue completely
    DestroyQueue {
        /// Queue ID
        queue_id: Pid,
    },

    /// Get queue statistics
    QueueStats {
        /// Queue ID
        queue_id: Pid,
    },
}
