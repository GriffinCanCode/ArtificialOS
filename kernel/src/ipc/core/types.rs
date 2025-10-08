/*!
 * IPC Types
 * Common types for inter-process communication
 */

use crate::core::serde::{is_zero_u64, is_zero_usize};
use crate::core::types::{Pid, Timestamp};
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// IPC operation result
///
/// # Must Use
/// IPC operations can fail and must be handled to prevent resource leaks
pub type IpcResult<T> = Result<T, IpcError>;

/// Unified IPC error type with miette diagnostics
#[derive(Error, Debug, Clone, Serialize, Deserialize, Diagnostic)]
#[serde(rename_all = "snake_case", tag = "error", content = "details")]
pub enum IpcError {
    /// Resource not found
    #[error("IPC resource not found: {0}")]
    #[diagnostic(
        code(ipc::not_found),
        help("The requested IPC resource (pipe, queue, or shared memory) does not exist. Verify the resource ID.")
    )]
    NotFound(String),

    /// Permission denied
    #[error("IPC permission denied: {0}")]
    #[diagnostic(
        code(ipc::permission_denied),
        help("Insufficient permissions to access this IPC resource. Check process capabilities.")
    )]
    PermissionDenied(String),

    /// Resource limit exceeded
    #[error("IPC resource limit exceeded: {0}")]
    #[diagnostic(
        code(ipc::limit_exceeded),
        help("IPC resource limit reached. Close unused resources or increase limits.")
    )]
    LimitExceeded(String),

    /// Operation would block
    #[error("IPC operation would block: {0}")]
    #[diagnostic(
        code(ipc::would_block),
        help("Operation cannot complete without blocking. Use non-blocking mode or wait for resource availability.")
    )]
    WouldBlock(String),

    /// Invalid operation or argument
    #[error("Invalid IPC operation: {0}")]
    #[diagnostic(
        code(ipc::invalid_operation),
        help("The requested operation is invalid for this IPC resource or in its current state.")
    )]
    InvalidOperation(String),

    /// Resource closed
    #[error("IPC resource closed: {0}")]
    #[diagnostic(
        code(ipc::closed),
        help("The IPC resource has been closed and can no longer be used.")
    )]
    Closed(String),

    /// Operation timed out
    #[error("IPC operation timed out after {elapsed_ms}ms (timeout: {}ms)", timeout_ms.map(|t| t.to_string()).unwrap_or_else(|| "none".to_string()))]
    #[diagnostic(
        code(ipc::timeout),
        help("The IPC operation did not complete within the specified timeout. Try increasing the timeout or check for deadlocks.")
    )]
    Timeout {
        elapsed_ms: u64,
        timeout_ms: Option<u64>,
    },
}

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
///
/// # Performance
/// - Cache-line aligned for fast message passing
/// - C-compatible layout for predictable memory layout
#[repr(C, align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: Pid,
    pub to: Pid,
    pub data: Vec<u8>,
    pub timestamp: MessageTimestamp,
    /// Memory address for tracking/deallocation (internal use)
    #[serde(skip)]
    pub(crate) mem_address: Option<crate::core::types::Address>,
}

impl Message {
    #[inline]
    #[must_use]
    pub fn new(from: Pid, to: Pid, data: Vec<u8>) -> Self {
        Self {
            from,
            to,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_micros() as Timestamp)
                .unwrap_or(0),
            mem_address: None,
        }
    }

    /// Get message size
    ///
    /// # Performance
    /// Hot path - called frequently for size calculations and limits
    #[inline(always)]
    #[must_use]
    pub fn size(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.len()
    }

    /// Serialize using bincode for internal IPC (much faster for binary data)
    ///
    /// This provides 5-10x better performance than JSON for messages with binary payloads.
    /// Use this for kernel-to-kernel IPC where the data doesn't need to be human-readable.
    #[must_use = "serialization can fail and must be handled"]
    pub fn to_bincode_bytes(&self) -> Result<Vec<u8>, String> {
        crate::core::bincode::to_vec(self)
            .map_err(|e| format!("Failed to serialize message with bincode: {}", e))
    }

    /// Deserialize from bincode format
    #[must_use = "deserialization can fail and must be handled"]
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
    /// Check if permission allows reading
    ///
    /// # Performance
    /// Hot path - frequently called in IPC operations
    #[inline(always)]
    #[must_use]
    pub const fn can_read(&self) -> bool {
        matches!(self, Permission::ReadOnly | Permission::ReadWrite)
    }

    /// Check if permission allows writing
    ///
    /// # Performance
    /// Hot path - frequently called in IPC operations
    #[inline(always)]
    #[must_use]
    pub const fn can_write(&self) -> bool {
        matches!(self, Permission::WriteOnly | Permission::ReadWrite)
    }
}

// Implement BincodeSerializable for efficient internal transfers
impl crate::core::traits::BincodeSerializable for Permission {}
