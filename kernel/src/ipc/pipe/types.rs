/*!
 * Pipe Types
 * Common types, constants, and errors for pipes
 */

use super::super::types::{IpcError, PipeId};
use crate::core::limits;
use crate::core::serialization::serde::{is_false, is_zero_usize};
use crate::core::types::{Pid, Size};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Pipe limits - centralized in core::limits
pub use limits::{
    DEFAULT_PIPE_CAPACITY, GLOBAL_PIPE_MEMORY_LIMIT, MAX_PIPES_PER_PROCESS, MAX_PIPE_CAPACITY,
};

/// Pipe error types
#[derive(Debug, Error)]
pub enum PipeError {
    #[error("Pipe not found: {0}")]
    NotFound(u32),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Pipe closed")]
    Closed,

    #[error("Would block: {0}")]
    WouldBlock(String),

    #[error("Capacity exceeded: requested {requested}, capacity {capacity}")]
    CapacityExceeded { requested: usize, capacity: usize },

    #[error("Process pipe limit exceeded: {0}/{1}")]
    ProcessLimitExceeded(usize, usize),

    #[error("Global pipe memory limit exceeded: {0}/{1} bytes")]
    GlobalMemoryExceeded(usize, usize),

    #[error("Memory allocation failed: {0}")]
    AllocationFailed(String),

    #[error("Operation timed out after {elapsed_ms}ms (timeout: {}ms)", timeout_ms.map(|t| t.to_string()).unwrap_or_else(|| "none".to_string()))]
    Timeout {
        elapsed_ms: u64,
        timeout_ms: Option<u64>,
    },

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

// Convert PipeError to IpcError
impl From<PipeError> for IpcError {
    fn from(err: PipeError) -> Self {
        match err {
            PipeError::NotFound(id) => IpcError::NotFound(format!("Pipe {}", id)),
            PipeError::PermissionDenied(msg) => IpcError::PermissionDenied(msg),
            PipeError::Closed => IpcError::Closed("Pipe closed".to_string()),
            PipeError::WouldBlock(msg) => IpcError::WouldBlock(msg),
            PipeError::CapacityExceeded {
                requested,
                capacity,
            } => IpcError::LimitExceeded(format!(
                "Capacity exceeded: requested {}, capacity {}",
                requested, capacity
            )),
            PipeError::ProcessLimitExceeded(current, max) => {
                IpcError::LimitExceeded(format!("Process pipe limit exceeded: {}/{}", current, max))
            }
            PipeError::GlobalMemoryExceeded(current, max) => IpcError::LimitExceeded(format!(
                "Global pipe memory limit exceeded: {}/{} bytes",
                current, max
            )),
            PipeError::AllocationFailed(msg) => {
                IpcError::InvalidOperation(format!("Memory allocation failed: {}", msg))
            }
            PipeError::Timeout {
                elapsed_ms,
                timeout_ms,
            } => IpcError::Timeout {
                elapsed_ms,
                timeout_ms,
            },
            PipeError::InvalidOperation(msg) => IpcError::InvalidOperation(msg),
        }
    }
}

/// Pipe statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PipeStats {
    pub id: PipeId,
    pub reader_pid: Pid,
    pub writer_pid: Pid,
    pub capacity: Size,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub buffered: Size,
    #[serde(skip_serializing_if = "is_false")]
    pub closed: bool,
}

impl PipeStats {
    /// Serialize using bincode for internal pipe operations
    ///
    /// This provides better performance than JSON for pipe metadata transfers.
    pub fn to_bincode_bytes(&self) -> Result<Vec<u8>, String> {
        crate::core::serialization::bincode::to_vec(self)
            .map_err(|e| format!("Failed to serialize pipe stats with bincode: {}", e))
    }

    /// Deserialize from bincode format
    pub fn from_bincode_bytes(bytes: &[u8]) -> Result<Self, String> {
        crate::core::serialization::bincode::from_slice(bytes)
            .map_err(|e| format!("Failed to deserialize pipe stats with bincode: {}", e))
    }
}

// Implement BincodeSerializable for PipeStats
impl crate::core::traits::BincodeSerializable for PipeStats {}
