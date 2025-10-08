/*!
 * eBPF Types
 * Core data structures for eBPF-based syscall filtering and monitoring
 */

use crate::core::data_structures::InlineString;
use crate::core::types::Pid;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use thiserror::Error;

/// eBPF-specific errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error_type")]
pub enum EbpfError {
    #[error("Platform not supported: {platform}")]
    UnsupportedPlatform { platform: InlineString },

    #[error("Failed to load eBPF program: {reason}")]
    LoadFailed { reason: InlineString },

    #[error("Failed to attach eBPF program: {reason}")]
    AttachFailed { reason: InlineString },

    #[error("Failed to detach eBPF program: {reason}")]
    DetachFailed { reason: InlineString },

    #[error("Map operation failed: {reason}")]
    MapError { reason: InlineString },

    #[error("Program not found: {name}")]
    ProgramNotFound { name: InlineString },

    #[error("Filter rule invalid: {reason}")]
    InvalidFilter { reason: InlineString },

    #[error("Permission denied for operation")]
    PermissionDenied,

    #[error("eBPF feature not available")]
    NotAvailable,
}

/// Result type for eBPF operations
pub type EbpfResult<T> = Result<T, EbpfError>;

/// Platform types for eBPF implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EbpfPlatform {
    /// Linux with full eBPF support
    Linux,
    /// macOS with limited tracing
    MacOS,
    /// Simulation mode for testing
    #[default]
    Simulation,
}

/// Syscall event captured by eBPF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallEvent {
    /// Process ID that made the syscall
    pub pid: Pid,
    /// Syscall number
    pub syscall_nr: u64,
    /// Syscall name (if resolved)
    pub name: Option<InlineString>,
    /// Arguments (up to 6)
    pub args: [u64; 6],
    /// Return value (for exit events)
    pub ret: Option<i64>,
    /// Timestamp in nanoseconds
    pub timestamp_ns: u64,
    /// Event type
    pub event_type: SyscallEventType,
    /// Thread ID
    pub tid: u32,
    /// User ID
    pub uid: u32,
    /// Group ID
    pub gid: u32,
    /// CPU core
    pub cpu: u32,
}

/// Syscall event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyscallEventType {
    /// Syscall entry point
    Enter,
    /// Syscall exit point
    Exit,
}

/// Network event captured by eBPF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    /// Process ID
    pub pid: Pid,
    /// Source IP address
    pub src_addr: Option<IpAddr>,
    /// Destination IP address
    pub dst_addr: Option<IpAddr>,
    /// Source port
    pub src_port: Option<u16>,
    /// Destination port
    pub dst_port: Option<u16>,
    /// Protocol (TCP, UDP, etc.)
    pub protocol: u8,
    /// Bytes sent/received
    pub bytes: u64,
    /// Event type
    pub event_type: NetworkEventType,
    /// Timestamp
    pub timestamp_ns: u64,
}

/// Network event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkEventType {
    Connect,
    Accept,
    Send,
    Receive,
    Close,
}

/// File operation event captured by eBPF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    /// Process ID
    pub pid: Pid,
    /// File path
    pub path: PathBuf,
    /// Operation type
    pub operation: FileOperation,
    /// File descriptor
    pub fd: Option<i32>,
    /// Bytes read/written
    pub bytes: Option<u64>,
    /// Flags
    pub flags: u32,
    /// Mode
    pub mode: u32,
    /// Timestamp
    pub timestamp_ns: u64,
}

/// File operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOperation {
    Open,
    Read,
    Write,
    Close,
    Create,
    Delete,
    Rename,
    Chmod,
    Chown,
}

/// eBPF filter rule for syscalls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallFilter {
    /// Filter ID
    pub id: InlineString,
    /// Process ID to filter (None = all processes)
    pub pid: Option<Pid>,
    /// Syscall numbers to filter (None = all syscalls)
    pub syscall_nrs: Option<Vec<u64>>,
    /// Action to take
    pub action: FilterAction,
    /// Priority (higher = evaluated first)
    pub priority: u32,
}

/// Filter action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterAction {
    /// Allow the syscall
    Allow,
    /// Block the syscall
    Deny,
    /// Allow but log the syscall
    Log,
    /// Rate limit the syscall
    RateLimit { max_per_sec: u32 },
}

/// eBPF program configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
    /// Program name
    pub name: InlineString,
    /// Program type
    pub program_type: ProgramType,
    /// Auto-attach on load
    pub auto_attach: bool,
    /// Enabled
    pub enabled: bool,
}

/// eBPF program type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramType {
    /// Trace syscall entry
    SyscallEntry,
    /// Trace syscall exit
    SyscallExit,
    /// Network socket operations
    NetworkSocket,
    /// File operations
    FileOps,
    /// Process lifecycle
    ProcessLifecycle,
}

/// eBPF statistics (Copy for use with SeqlockStats)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct EbpfStats {
    /// Total programs loaded
    pub programs_loaded: usize,
    /// Total programs attached
    pub programs_attached: usize,
    /// Total syscall events captured
    pub syscall_events: u64,
    /// Total network events captured
    pub network_events: u64,
    /// Total file events captured
    pub file_events: u64,
    /// Active filters
    pub active_filters: usize,
    /// Events per second
    pub events_per_sec: f64,
    /// Platform
    pub platform: EbpfPlatform,
}

/// Program information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramInfo {
    /// Program name
    pub name: InlineString,
    /// Program type
    pub program_type: ProgramType,
    /// Is attached
    pub attached: bool,
    /// Events captured
    pub events_captured: u64,
    /// Created timestamp
    pub created_at: u64,
}

/// Event subscriber callback type
pub type EventCallback = Box<dyn Fn(EbpfEvent) + Send + Sync>;

/// Unified eBPF event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "event_kind")]
pub enum EbpfEvent {
    Syscall(SyscallEvent),
    Network(NetworkEvent),
    File(FileEvent),
}

impl EbpfEvent {
    /// Get the PID associated with this event
    pub fn pid(&self) -> Pid {
        match self {
            EbpfEvent::Syscall(e) => e.pid,
            EbpfEvent::Network(e) => e.pid,
            EbpfEvent::File(e) => e.pid,
        }
    }

    /// Get the timestamp
    pub fn timestamp_ns(&self) -> u64 {
        match self {
            EbpfEvent::Syscall(e) => e.timestamp_ns,
            EbpfEvent::Network(e) => e.timestamp_ns,
            EbpfEvent::File(e) => e.timestamp_ns,
        }
    }
}

/// Map of syscall numbers to names (architecture-specific)
pub fn syscall_name(nr: u64) -> Option<&'static str> {
    // x86_64 syscall numbers (common ones)
    match nr {
        0 => Some("read"),
        1 => Some("write"),
        2 => Some("open"),
        3 => Some("close"),
        4 => Some("stat"),
        5 => Some("fstat"),
        9 => Some("mmap"),
        10 => Some("mprotect"),
        11 => Some("munmap"),
        41 => Some("socket"),
        42 => Some("connect"),
        43 => Some("accept"),
        44 => Some("sendto"),
        45 => Some("recvfrom"),
        56 => Some("clone"),
        57 => Some("fork"),
        59 => Some("execve"),
        60 => Some("exit"),
        _ => None,
    }
}
