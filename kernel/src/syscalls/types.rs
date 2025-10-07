/*!
 * Syscall Types
 * Defines syscall enum and result types with modern serde patterns
 */

use crate::core::serde::skip_serializing_none;
use crate::core::types::{Fd, Pid, Priority, Size, SockFd};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::path::PathBuf;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Syscall operation errors with rich context
#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "error_type", content = "details")]
#[non_exhaustive]
pub enum SyscallError {
    /// Permission denied for the requested operation
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Operation failed with an error message
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Invalid argument provided to syscall
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Resource not found (file, process, etc.)
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Resource temporarily unavailable
    #[error("Resource unavailable: {0}")]
    Unavailable(String),

    /// I/O error occurred
    #[error("I/O error: {0}")]
    IoError(String),

    /// Feature not yet implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Required manager/subsystem not available
    #[error("Manager not available: {0}")]
    ManagerNotAvailable(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl SyscallError {
    /// Create a permission denied error
    #[inline]
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create an operation failed error
    #[inline]
    pub fn operation_failed(msg: impl Into<String>) -> Self {
        Self::OperationFailed(msg.into())
    }

    /// Create an invalid argument error
    #[inline]
    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Self::InvalidArgument(msg.into())
    }

    /// Create a not found error
    #[inline]
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a manager not available error
    #[inline]
    pub fn manager_not_available(subsystem: impl Into<String>) -> Self {
        Self::ManagerNotAvailable(subsystem.into())
    }
}

// ============================================================================
// Result Types
// ============================================================================

/// System call result with modern serde patterns
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum SyscallResult {
    /// Successful operation
    Success {
        /// Optional data payload (binary)
        data: Option<Vec<u8>>,
    },
    /// Operation failed with error message
    Error {
        /// Human-readable error message
        message: String,
    },
    /// Permission denied with reason
    PermissionDenied {
        /// Reason for permission denial
        reason: String,
    },
}

impl SyscallResult {
    pub fn success() -> Self {
        Self::Success { data: None }
    }

    pub fn success_with_data(data: Vec<u8>) -> Self {
        Self::Success { data: Some(data) }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    pub fn permission_denied(reason: impl Into<String>) -> Self {
        Self::PermissionDenied {
            reason: reason.into(),
        }
    }

    /// Check if result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if result is error
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Check if result is permission denied
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Self::PermissionDenied { .. })
    }

    /// Extract data if successful
    pub fn data(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Success { data } => data.as_ref(),
            _ => None,
        }
    }
}

/// Convert from SyscallError to SyscallResult
impl From<SyscallError> for SyscallResult {
    fn from(err: SyscallError) -> Self {
        match err {
            SyscallError::PermissionDenied(msg) => Self::PermissionDenied { reason: msg },
            other => Self::Error {
                message: other.to_string(),
            },
        }
    }
}

// ============================================================================
// Syscall Enum
// ============================================================================

/// System call types with modern serde patterns
#[serde_as]
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
        #[serde(skip_serializing_if = "Option::is_none")]
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
        #[serde(skip_serializing_if = "Option::is_none")]
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
        #[serde(skip_serializing_if = "Option::is_none")]
        capacity: Option<Size>,
    },

    /// Send message to queue
    SendQueue {
        /// Queue ID
        queue_id: Pid,
        /// Message data
        data: Vec<u8>,
        /// Optional priority (for priority queues)
        #[serde(skip_serializing_if = "Option::is_none")]
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
        #[serde(skip_serializing_if = "Option::is_none")]
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

// ============================================================================
// Helper Structures
// ============================================================================

/// Process execution output
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProcessOutput {
    /// Standard output content
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stdout: String,

    /// Standard error content
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stderr: String,

    /// Process exit code
    pub exit_code: i32,
}

impl ProcessOutput {
    /// Create new process output
    #[inline]
    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            stdout,
            stderr,
            exit_code,
        }
    }

    /// Check if process was successful (exit code 0)
    #[inline]
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// Check if output is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stdout.is_empty() && self.stderr.is_empty()
    }
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    /// Operating system name
    pub os: String,

    /// CPU architecture
    pub arch: String,

    /// OS family (unix, windows, etc.)
    pub family: String,
}

impl SystemInfo {
    /// Create new system info
    #[inline]
    pub fn new(os: String, arch: String, family: String) -> Self {
        Self { os, arch, family }
    }

    /// Get current system info
    #[inline]
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            family: std::env::consts::FAMILY.to_string(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_error_helpers() {
        let err = SyscallError::permission_denied("test");
        assert!(matches!(err, SyscallError::PermissionDenied(_)));

        let err = SyscallError::not_found("missing");
        assert!(matches!(err, SyscallError::NotFound(_)));
    }

    #[test]
    fn test_syscall_result_methods() {
        let success = SyscallResult::success();
        assert!(success.is_success());
        assert!(!success.is_error());

        let error = SyscallResult::error("failed");
        assert!(error.is_error());
        assert!(!error.is_success());
    }

    #[test]
    fn test_process_output() {
        let output = ProcessOutput::new("hello".to_string(), String::new(), 0);
        assert!(output.is_success());
        assert!(!output.is_empty());

        let output = ProcessOutput::default();
        assert!(output.is_success());
        assert!(output.is_empty());
    }

    #[test]
    fn test_system_info_serialization() {
        let info = SystemInfo::current();
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: SystemInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, deserialized);
    }

    #[test]
    fn test_syscall_serialization() {
        let syscall = Syscall::ReadFile {
            path: "/tmp/test".into(),
        };
        let json = serde_json::to_string(&syscall).unwrap();
        let deserialized: Syscall = serde_json::from_str(&json).unwrap();
        assert_eq!(syscall, deserialized);
    }

    #[test]
    fn test_syscall_result_serialization() {
        let result = SyscallResult::success_with_data(vec![1, 2, 3]);
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SyscallResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, deserialized);
    }
}
