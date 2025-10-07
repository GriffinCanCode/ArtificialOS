/*!
 * Syscall Enum
 * Defines all system call variants
 */

use crate::core::serde::skip_serializing_none;
use crate::core::types::{Fd, Pid, Priority, Size, SockFd};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// System call types with modern serde patterns
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "syscall")]
#[non_exhaustive]
pub enum Syscall {
    // ========================================================================
    // File System Operations
    // ========================================================================

    /// Read file contents
    ReadFile {
        /// Path to file
        path: PathBuf,
    },

    /// Write data to file
    WriteFile {
        /// Path to file
        path: PathBuf,
        /// Data to write
        data: Vec<u8>,
    },

    /// Create empty file
    CreateFile {
        /// Path to file
        path: PathBuf,
    },

    /// Delete file
    DeleteFile {
        /// Path to file
        path: PathBuf,
    },

    /// List directory contents
    ListDirectory {
        /// Path to directory
        path: PathBuf,
    },

    /// Check if file exists
    FileExists {
        /// Path to check
        path: PathBuf,
    },

    /// Get file metadata
    FileStat {
        /// Path to file
        path: PathBuf,
    },

    /// Move/rename file
    MoveFile {
        /// Source path
        source: PathBuf,
        /// Destination path
        destination: PathBuf,
    },

    /// Copy file
    CopyFile {
        /// Source path
        source: PathBuf,
        /// Destination path
        destination: PathBuf,
    },

    /// Create directory
    CreateDirectory {
        /// Path to directory
        path: PathBuf,
    },

    /// Remove directory
    RemoveDirectory {
        /// Path to directory
        path: PathBuf,
    },

    /// Get current working directory
    GetWorkingDirectory,

    /// Set current working directory
    SetWorkingDirectory {
        /// Path to directory
        path: PathBuf,
    },

    /// Truncate file to size
    TruncateFile {
        /// Path to file
        path: PathBuf,
        /// New size in bytes
        size: u64,
    },

    // ========================================================================
    // Process Operations
    // ========================================================================

    /// Spawn a new process
    SpawnProcess {
        /// Command to execute
        command: String,
        /// Command arguments
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
    },

    /// Kill/terminate process
    KillProcess {
        /// Process ID to kill
        target_pid: Pid,
    },

    /// Get process information
    GetProcessInfo {
        /// Process ID to query
        target_pid: Pid,
    },

    /// Get list of all processes
    GetProcessList,

    /// Set process priority
    SetProcessPriority {
        /// Process ID to modify
        target_pid: Pid,
        /// New priority level
        priority: Priority,
    },

    /// Get process state
    GetProcessState {
        /// Process ID to query
        target_pid: Pid,
    },

    /// Get process statistics
    GetProcessStats {
        /// Process ID to query
        target_pid: Pid,
    },

    /// Wait for process to complete
    WaitProcess {
        /// Process ID to wait for
        target_pid: Pid,
        /// Optional timeout in milliseconds
        timeout_ms: Option<u64>,
    },

    // ========================================================================
    // System Information
    // ========================================================================

    /// Get system information
    GetSystemInfo,

    /// Get current system time
    GetCurrentTime,

    /// Get environment variable
    GetEnvironmentVar {
        /// Variable name
        key: String,
    },

    /// Set environment variable
    SetEnvironmentVar {
        /// Variable name
        key: String,
        /// Variable value
        value: String,
    },

    /// Make HTTP network request
    NetworkRequest {
        /// URL to fetch
        url: String,
    },

    // ========================================================================
    // IPC - Pipes
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
    // IPC - Shared Memory
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
    // IPC - Memory-Mapped Files
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
    // IPC - Async Queues
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

    // ========================================================================
    // Scheduler Operations
    // ========================================================================

    /// Schedule next process
    ScheduleNext,

    /// Yield current process
    YieldProcess,

    /// Get currently scheduled process
    GetCurrentScheduled,

    /// Get global scheduler statistics
    GetSchedulerStats,

    /// Set scheduling policy
    SetSchedulingPolicy {
        /// Policy: "round_robin", "priority", or "fair"
        policy: String,
    },

    /// Get current scheduling policy
    GetSchedulingPolicy,

    /// Set time quantum for scheduler
    SetTimeQuantum {
        /// Time quantum in microseconds
        quantum_micros: u64,
    },

    /// Get current time quantum
    GetTimeQuantum,

    /// Get scheduler stats for a process
    GetProcessSchedulerStats {
        /// Process ID to query
        target_pid: Pid,
    },

    /// Get scheduler stats for all processes
    GetAllProcessSchedulerStats,

    /// Boost process priority
    BoostPriority {
        /// Process ID to boost
        target_pid: Pid,
    },

    /// Lower process priority
    LowerPriority {
        /// Process ID to lower
        target_pid: Pid,
    },

    // ========================================================================
    // Time Operations
    // ========================================================================

    /// Sleep for duration
    Sleep {
        /// Duration in milliseconds
        duration_ms: u64,
    },

    /// Get system uptime
    GetUptime,

    // ========================================================================
    // Memory Operations
    // ========================================================================

    /// Get global memory statistics
    GetMemoryStats,

    /// Get process memory statistics
    GetProcessMemoryStats {
        /// Process ID to query
        target_pid: Pid,
    },

    /// Trigger garbage collection
    TriggerGC {
        /// Optional target process ID (None = global GC)
        target_pid: Option<u32>,
    },

    // ========================================================================
    // Signal Operations
    // ========================================================================

    /// Send signal to process
    SendSignal {
        /// Target process ID
        target_pid: Pid,
        /// Signal number
        signal: u32,
    },

    // ========================================================================
    // Network Operations - Sockets
    // ========================================================================

    /// Create socket
    Socket {
        /// Address family (AF_INET, AF_INET6, etc.)
        domain: u32,
        /// Socket type (SOCK_STREAM, SOCK_DGRAM, etc.)
        socket_type: u32,
        /// Protocol (IPPROTO_TCP, IPPROTO_UDP, etc.)
        #[serde(default)]
        protocol: u32,
    },

    /// Bind socket to address
    Bind {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Address in IP:port format
        address: String,
    },

    /// Listen on socket
    Listen {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Connection backlog
        #[serde(default)]
        backlog: u32,
    },

    /// Accept connection on socket
    Accept {
        /// Socket file descriptor
        sockfd: SockFd,
    },

    /// Connect socket to address
    Connect {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Address in IP:port format
        address: String,
    },

    /// Send data on socket
    Send {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Data to send
        data: Vec<u8>,
        /// Send flags
        #[serde(default)]
        flags: u32,
    },

    /// Receive data from socket
    Recv {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Maximum bytes to receive
        size: Size,
        /// Receive flags
        #[serde(default)]
        flags: u32,
    },

    /// Send data to specific address (UDP)
    SendTo {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Data to send
        data: Vec<u8>,
        /// Destination address
        address: String,
        /// Send flags
        #[serde(default)]
        flags: u32,
    },

    /// Receive data with source address (UDP)
    RecvFrom {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Maximum bytes to receive
        size: Size,
        /// Receive flags
        #[serde(default)]
        flags: u32,
    },

    /// Close socket
    CloseSocket {
        /// Socket file descriptor
        sockfd: SockFd,
    },

    /// Set socket option
    SetSockOpt {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Protocol level
        level: u32,
        /// Option name
        optname: u32,
        /// Option value
        optval: Vec<u8>,
    },

    /// Get socket option
    GetSockOpt {
        /// Socket file descriptor
        sockfd: SockFd,
        /// Protocol level
        level: u32,
        /// Option name
        optname: u32,
    },

    // ========================================================================
    // File Descriptor Operations
    // ========================================================================

    /// Open file and return FD
    Open {
        /// Path to file
        path: PathBuf,
        /// Open flags (O_RDONLY, O_WRONLY, O_RDWR, O_CREAT, O_APPEND, etc.)
        flags: u32,
        /// File permissions (0644, etc.)
        #[serde(default)]
        mode: u32,
    },

    /// Close file descriptor
    Close {
        /// File descriptor
        fd: Fd,
    },

    /// Duplicate file descriptor
    Dup {
        /// File descriptor to duplicate
        fd: Fd,
    },

    /// Duplicate FD to specific number
    Dup2 {
        /// Source file descriptor
        oldfd: Fd,
        /// Target file descriptor
        newfd: Fd,
    },

    /// Seek within file
    Lseek {
        /// File descriptor
        fd: Fd,
        /// Offset to seek to
        offset: i64,
        /// Seek mode (SEEK_SET, SEEK_CUR, SEEK_END)
        whence: u32,
    },

    /// File control operations
    Fcntl {
        /// File descriptor
        fd: Fd,
        /// Control command
        cmd: u32,
        /// Command argument
        #[serde(default)]
        arg: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_serialization() {
        let syscall = Syscall::ReadFile {
            path: "/tmp/test".into(),
        };
        let json = serde_json::to_string(&syscall).unwrap();
        let deserialized: Syscall = serde_json::from_str(&json).unwrap();
        assert_eq!(syscall, deserialized);
    }
}
