/*!
 * IPC Types
 * Common types for inter-process communication
 */

use crate::core::serde::{is_zero_u64, is_zero_usize};
use crate::core::types::{Pid, Timestamp};
use serde::{Deserialize, Serialize};

/// IPC operation result
pub type IpcResult<T> = Result<T, IpcError>;

/// Unified IPC error type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error", content = "details")]
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

/// Queue identifier
pub type QueueId = u32;

/// Message timestamp
pub type MessageTimestamp = Timestamp;

/// Queue type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueType {
    /// First-in-first-out queue
    Fifo,
    /// Priority-based queue (higher priority first)
    Priority,
    /// Publish-subscribe (broadcast to all subscribers)
    PubSub,
}

// Implement BincodeSerializable for QueueType
impl crate::core::traits::BincodeSerializable for QueueType {}

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

    /// Serialize using bincode for internal IPC (much faster for binary data)
    ///
    /// This provides 5-10x better performance than JSON for messages with binary payloads.
    /// Use this for kernel-to-kernel IPC where the data doesn't need to be human-readable.
    pub fn to_bincode_bytes(&self) -> Result<Vec<u8>, String> {
        crate::core::bincode::to_vec(self)
            .map_err(|e| format!("Failed to serialize message with bincode: {}", e))
    }

    /// Deserialize from bincode format
    pub fn from_bincode_bytes(bytes: &[u8]) -> Result<Self, String> {
        crate::core::bincode::from_slice(bytes)
            .map_err(|e| format!("Failed to deserialize message with bincode: {}", e))
    }
}

// Implement BincodeSerializable trait for Message
impl crate::core::traits::BincodeSerializable for Message {}

/// IPC statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcStats {
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub messages_sent: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub messages_received: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub pipes_created: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub shm_segments_created: u64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub global_memory_usage: usize,
}

// Implement BincodeSerializable for efficient internal transfers
impl crate::core::traits::BincodeSerializable for IpcStats {}

/// Permission for shared resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

// Implement BincodeSerializable for efficient internal transfers
impl crate::core::traits::BincodeSerializable for Permission {}
