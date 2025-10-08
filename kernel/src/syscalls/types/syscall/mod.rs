/*!
 * Syscall Enum
 * Unified flat system call type with organized module structure
 */

use crate::core::serialization::serde::skip_serializing_none;
use crate::core::types::{Fd, Pid, Priority, Size, SockFd};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Organized syscall definitions by category (internal modules)
pub mod fs;
pub mod ipc;
pub mod network;
pub mod process;
pub mod scheduler;
pub mod system;

/// System call types with modern serde patterns
///
/// This enum provides a flat interface for backward compatibility while
/// keeping the implementation organized in category modules.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "syscall")]
#[non_exhaustive]
pub enum Syscall {
    // ========================================================================
    // File System Operations (from fs module)
    // ========================================================================
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
    Open {
        path: PathBuf,
        flags: u32,
        #[serde(default)]
        mode: u32,
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
        whence: u32,
    },
    Fcntl {
        fd: Fd,
        cmd: u32,
        #[serde(default)]
        arg: u32,
    },

    // ========================================================================
    // Process Operations (from process module)
    // ========================================================================
    SpawnProcess {
        command: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
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

    // ========================================================================
    // IPC Operations (from ipc module)
    // ========================================================================
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

    CreateShm {
        size: Size,
    },
    AttachShm {
        segment_id: Pid,
        #[serde(default)]
        read_only: bool,
    },
    DetachShm {
        segment_id: Pid,
    },
    WriteShm {
        segment_id: Pid,
        #[serde(default)]
        offset: usize,
        data: Vec<u8>,
    },
    ReadShm {
        segment_id: Pid,
        #[serde(default)]
        offset: usize,
        size: Size,
    },
    DestroyShm {
        segment_id: Pid,
    },
    ShmStats {
        segment_id: Pid,
    },

    Mmap {
        path: String,
        #[serde(default)]
        offset: usize,
        length: Size,
        prot: u8,
        #[serde(default)]
        shared: bool,
    },
    MmapRead {
        mmap_id: u32,
        #[serde(default)]
        offset: usize,
        length: Size,
    },
    MmapWrite {
        mmap_id: u32,
        #[serde(default)]
        offset: usize,
        data: Vec<u8>,
    },
    Msync {
        mmap_id: u32,
    },
    Munmap {
        mmap_id: u32,
    },
    MmapStats {
        mmap_id: u32,
    },

    CreateQueue {
        queue_type: String,
        capacity: Option<Size>,
    },
    SendQueue {
        queue_id: Pid,
        data: Vec<u8>,
        priority: Option<u8>,
    },
    ReceiveQueue {
        queue_id: Pid,
    },
    SubscribeQueue {
        queue_id: Pid,
    },
    UnsubscribeQueue {
        queue_id: Pid,
    },
    CloseQueue {
        queue_id: Pid,
    },
    DestroyQueue {
        queue_id: Pid,
    },
    QueueStats {
        queue_id: Pid,
    },

    // ========================================================================
    // Network Operations (from network module)
    // ========================================================================
    NetworkRequest {
        url: String,
    },
    Socket {
        domain: u32,
        socket_type: u32,
        #[serde(default)]
        protocol: u32,
    },
    Bind {
        sockfd: SockFd,
        address: String,
    },
    Listen {
        sockfd: SockFd,
        #[serde(default)]
        backlog: u32,
    },
    Accept {
        sockfd: SockFd,
    },
    Connect {
        sockfd: SockFd,
        address: String,
    },
    Send {
        sockfd: SockFd,
        data: Vec<u8>,
        #[serde(default)]
        flags: u32,
    },
    Recv {
        sockfd: SockFd,
        size: Size,
        #[serde(default)]
        flags: u32,
    },
    SendTo {
        sockfd: SockFd,
        data: Vec<u8>,
        address: String,
        #[serde(default)]
        flags: u32,
    },
    RecvFrom {
        sockfd: SockFd,
        size: Size,
        #[serde(default)]
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

    // ========================================================================
    // Scheduler Operations (from scheduler module)
    // ========================================================================
    ScheduleNext,
    YieldProcess,
    GetCurrentScheduled,
    GetSchedulerStats,
    SetSchedulingPolicy {
        policy: String,
    },
    GetSchedulingPolicy,
    SetTimeQuantum {
        quantum_micros: u64,
    },
    GetTimeQuantum,
    GetProcessSchedulerStats {
        target_pid: Pid,
    },
    GetAllProcessSchedulerStats,
    BoostPriority {
        target_pid: Pid,
    },
    LowerPriority {
        target_pid: Pid,
    },

    // ========================================================================
    // System Operations (from system module)
    // ========================================================================
    GetSystemInfo,
    GetCurrentTime,
    GetEnvironmentVar {
        key: String,
    },
    SetEnvironmentVar {
        key: String,
        value: String,
    },
    Sleep {
        duration_ms: u64,
    },
    GetUptime,
    GetMemoryStats,
    GetProcessMemoryStats {
        target_pid: Pid,
    },
    TriggerGC {
        target_pid: Option<u32>,
    },
    SendSignal {
        target_pid: Pid,
        signal: u32,
    },
    RegisterSignalHandler {
        signal: u32,
        handler_id: u64,
    },
    BlockSignal {
        signal: u32,
    },
    UnblockSignal {
        signal: u32,
    },
    GetPendingSignals,
    GetSignalStats,
    WaitForSignal {
        signals: Vec<u32>,
        timeout_ms: Option<u64>,
    },
    GetSignalState {
        target_pid: Option<Pid>,
    },

    // ========================================================================
    // Clipboard Operations
    // ========================================================================
    ClipboardCopy {
        data: Vec<u8>,
        format: String,
        #[serde(default)]
        global: bool,
    },
    ClipboardPaste {
        #[serde(default)]
        global: bool,
    },
    ClipboardHistory {
        #[serde(default)]
        global: bool,
        limit: Option<usize>,
    },
    ClipboardGetEntry {
        entry_id: u64,
    },
    ClipboardClear {
        #[serde(default)]
        global: bool,
    },
    ClipboardSubscribe {
        formats: Vec<String>,
    },
    ClipboardUnsubscribe,
    ClipboardStats,
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
