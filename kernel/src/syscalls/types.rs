/*!
 * Syscall Types
 * Defines syscall enum and result types
 */

use crate::core::types::{Fd, Pid, Priority, Size, SockFd};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Syscall operation errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SyscallError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource unavailable: {0}")]
    Unavailable(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Manager not available: {0}")]
    ManagerNotAvailable(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// System call result (keeping for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyscallResult {
    Success { data: Option<Vec<u8>> },
    Error { message: String },
    PermissionDenied { reason: String },
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

/// System call types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Syscall {
    // File system operations
    ReadFile {
        path: PathBuf,
    },
    WriteFile {
        path: PathBuf,
        data: Vec<u8>,
    },
    CreateFile {
        path: PathBuf,
    },
    DeleteFile {
        path: PathBuf,
    },
    ListDirectory {
        path: PathBuf,
    },
    FileExists {
        path: PathBuf,
    },
    FileStat {
        path: PathBuf,
    },
    MoveFile {
        source: PathBuf,
        destination: PathBuf,
    },
    CopyFile {
        source: PathBuf,
        destination: PathBuf,
    },
    CreateDirectory {
        path: PathBuf,
    },
    RemoveDirectory {
        path: PathBuf,
    },
    GetWorkingDirectory,
    SetWorkingDirectory {
        path: PathBuf,
    },
    TruncateFile {
        path: PathBuf,
        size: u64,
    },

    // Process operations
    SpawnProcess {
        command: String,
        args: Vec<String>,
    },
    KillProcess {
        target_pid: Pid,
    },
    GetProcessInfo {
        target_pid: Pid,
    },
    GetProcessList,
    SetProcessPriority {
        target_pid: Pid,
        priority: Priority,
    },
    GetProcessState {
        target_pid: Pid,
    },
    GetProcessStats {
        target_pid: Pid,
    },
    WaitProcess {
        target_pid: Pid,
        timeout_ms: Option<u64>,
    },

    // System info
    GetSystemInfo,
    GetCurrentTime,
    GetEnvironmentVar {
        key: String,
    },
    SetEnvironmentVar {
        key: String,
        value: String,
    },

    // Network (placeholder for future)
    NetworkRequest {
        url: String,
    },

    // IPC - Pipes
    CreatePipe {
        reader_pid: Pid,
        writer_pid: Pid,
        capacity: Option<Size>,
    },
    WritePipe {
        pipe_id: Pid,
        data: Vec<u8>,
    },
    ReadPipe {
        pipe_id: Pid,
        size: Size,
    },
    ClosePipe {
        pipe_id: Pid,
    },
    DestroyPipe {
        pipe_id: Pid,
    },
    PipeStats {
        pipe_id: Pid,
    },

    // IPC - Shared Memory
    CreateShm {
        size: Size,
    },
    AttachShm {
        segment_id: Pid,
        read_only: bool,
    },
    DetachShm {
        segment_id: Pid,
    },
    WriteShm {
        segment_id: Pid,
        offset: usize,
        data: Vec<u8>,
    },
    ReadShm {
        segment_id: Pid,
        offset: usize,
        size: Size,
    },
    DestroyShm {
        segment_id: Pid,
    },
    ShmStats {
        segment_id: Pid,
    },

    // Scheduler operations
    ScheduleNext,
    YieldProcess,
    GetCurrentScheduled,
    GetSchedulerStats,

    // Time operations
    Sleep {
        duration_ms: u64,
    },
    GetUptime,

    // Memory operations
    GetMemoryStats,
    GetProcessMemoryStats {
        target_pid: Pid,
    },
    TriggerGC {
        target_pid: Option<u32>,
    },

    // Signal operations
    SendSignal {
        target_pid: Pid,
        signal: u32,
    },

    // Network operations - Sockets
    Socket {
        domain: u32,  // AF_INET, AF_INET6, etc.
        socket_type: u32,  // SOCK_STREAM, SOCK_DGRAM, etc.
        protocol: u32,  // IPPROTO_TCP, IPPROTO_UDP, etc.
    },
    Bind {
        sockfd: SockFd,
        address: String,  // IP:port format
    },
    Listen {
        sockfd: SockFd,
        backlog: u32,
    },
    Accept {
        sockfd: SockFd,
    },
    Connect {
        sockfd: SockFd,
        address: String,  // IP:port format
    },
    Send {
        sockfd: SockFd,
        data: Vec<u8>,
        flags: u32,
    },
    Recv {
        sockfd: SockFd,
        size: Size,
        flags: u32,
    },
    SendTo {
        sockfd: SockFd,
        data: Vec<u8>,
        address: String,
        flags: u32,
    },
    RecvFrom {
        sockfd: SockFd,
        size: Size,
        flags: u32,
    },
    CloseSocket {
        sockfd: SockFd,
    },
    SetSockOpt {
        sockfd: SockFd,
        level: u32,
        optname: u32,
        optval: Vec<u8>,
    },
    GetSockOpt {
        sockfd: SockFd,
        level: u32,
        optname: u32,
    },

    // File Descriptor operations
    Open {
        path: PathBuf,
        flags: u32,  // O_RDONLY, O_WRONLY, O_RDWR, O_CREAT, O_APPEND, etc.
        mode: u32,   // File permissions (0644, etc.)
    },
    Close {
        fd: Fd,
    },
    Dup {
        fd: Fd,
    },
    Dup2 {
        oldfd: Fd,
        newfd: Fd,
    },
    Lseek {
        fd: Fd,
        offset: i64,
        whence: u32,  // SEEK_SET, SEEK_CUR, SEEK_END
    },
    Fcntl {
        fd: Fd,
        cmd: u32,
        arg: u32,
    },
}

/// Helper structs for serialization
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub family: String,
}

