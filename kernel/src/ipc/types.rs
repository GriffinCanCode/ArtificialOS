/*!
 * IPC Types
 * Common types for inter-process communication
 */

use crate::core::types::{Pid, Timestamp};
use serde::{Deserialize, Serialize};

/// IPC operation result
pub type IpcResult<T> = Result<T, IpcError>;

/// Unified IPC error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcError {
    /// Resource not found
    NotFound(String),

    /// Permission denied
    PermissionDenied(String),

    /// Resource limit exceeded
    LimitExceeded(String),

    /// Operation would block
    WouldBlock(String),

    /// Invalid operation or argument
    InvalidOperation(String),

    /// Resource closed
    Closed(String),
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IpcError::NotFound(msg) => write!(f, "Not found: {}", msg),
            IpcError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            IpcError::LimitExceeded(msg) => write!(f, "Limit exceeded: {}", msg),
            IpcError::WouldBlock(msg) => write!(f, "Would block: {}", msg),
            IpcError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            IpcError::Closed(msg) => write!(f, "Closed: {}", msg),
        }
    }
}

impl std::error::Error for IpcError {}

/// IPC channel identifier
pub type ChannelId = u32;

/// Pipe identifier
pub type PipeId = u32;

/// Shared memory segment identifier
pub type ShmId = u32;

/// Message timestamp
pub type MessageTimestamp = Timestamp;

/// IPC message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: Pid,
    pub to: Pid,
    pub data: Vec<u8>,
    pub timestamp: MessageTimestamp,
}

impl Message {
    pub fn new(from: Pid, to: Pid, data: Vec<u8>) -> Self {
        Self {
            from,
            to,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_micros() as Timestamp)
                .unwrap_or(0),
        }
    }

    pub fn size(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.len()
    }
}

/// IPC statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub pipes_created: u64,
    pub shm_segments_created: u64,
    pub global_memory_usage: usize,
}

/// Permission for shared resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl Permission {
    pub fn can_read(&self) -> bool {
        matches!(self, Permission::ReadOnly | Permission::ReadWrite)
    }

    pub fn can_write(&self) -> bool {
        matches!(self, Permission::WriteOnly | Permission::ReadWrite)
    }
}
