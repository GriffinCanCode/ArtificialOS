/*!
 * Syscall Types
 * Defines syscall enum and result types
 */

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// System call result
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
        target_pid: u32,
    },
    GetProcessInfo {
        target_pid: u32,
    },
    GetProcessList,
    SetProcessPriority {
        target_pid: u32,
        priority: u8,
    },
    GetProcessState {
        target_pid: u32,
    },
    GetProcessStats {
        target_pid: u32,
    },
    WaitProcess {
        target_pid: u32,
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
        reader_pid: u32,
        writer_pid: u32,
        capacity: Option<usize>,
    },
    WritePipe {
        pipe_id: u32,
        data: Vec<u8>,
    },
    ReadPipe {
        pipe_id: u32,
        size: usize,
    },
    ClosePipe {
        pipe_id: u32,
    },
    DestroyPipe {
        pipe_id: u32,
    },
    PipeStats {
        pipe_id: u32,
    },

    // IPC - Shared Memory
    CreateShm {
        size: usize,
    },
    AttachShm {
        segment_id: u32,
        read_only: bool,
    },
    DetachShm {
        segment_id: u32,
    },
    WriteShm {
        segment_id: u32,
        offset: usize,
        data: Vec<u8>,
    },
    ReadShm {
        segment_id: u32,
        offset: usize,
        size: usize,
    },
    DestroyShm {
        segment_id: u32,
    },
    ShmStats {
        segment_id: u32,
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
        target_pid: u32,
    },
    TriggerGC {
        target_pid: Option<u32>,
    },

    // Signal operations
    SendSignal {
        target_pid: u32,
        signal: u32,
    },

    // Network operations - Sockets
    Socket {
        domain: u32,  // AF_INET, AF_INET6, etc.
        socket_type: u32,  // SOCK_STREAM, SOCK_DGRAM, etc.
        protocol: u32,  // IPPROTO_TCP, IPPROTO_UDP, etc.
    },
    Bind {
        sockfd: u32,
        address: String,  // IP:port format
    },
    Listen {
        sockfd: u32,
        backlog: u32,
    },
    Accept {
        sockfd: u32,
    },
    Connect {
        sockfd: u32,
        address: String,  // IP:port format
    },
    Send {
        sockfd: u32,
        data: Vec<u8>,
        flags: u32,
    },
    Recv {
        sockfd: u32,
        size: usize,
        flags: u32,
    },
    SendTo {
        sockfd: u32,
        data: Vec<u8>,
        address: String,
        flags: u32,
    },
    RecvFrom {
        sockfd: u32,
        size: usize,
        flags: u32,
    },
    CloseSocket {
        sockfd: u32,
    },
    SetSockOpt {
        sockfd: u32,
        level: u32,
        optname: u32,
        optval: Vec<u8>,
    },
    GetSockOpt {
        sockfd: u32,
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
        fd: u32,
    },
    Dup {
        fd: u32,
    },
    Dup2 {
        oldfd: u32,
        newfd: u32,
    },
    Lseek {
        fd: u32,
        offset: i64,
        whence: u32,  // SEEK_SET, SEEK_CUR, SEEK_END
    },
    Fcntl {
        fd: u32,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessMemoryStats {
    pub pid: u32,
    pub bytes_allocated: usize,
}
