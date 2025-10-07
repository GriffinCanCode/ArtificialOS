/*!
 * Shared Memory Types
 * Common types, constants, and errors for shared memory
 */

use super::super::types::{IpcError, ShmId};
use crate::core::serde::{is_empty_vec, is_zero_usize};
use crate::core::types::{Pid, Size};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Shared memory limits
pub const MAX_SEGMENT_SIZE: usize = 100 * 1024 * 1024; // 100MB per segment
pub const MAX_SEGMENTS_PER_PROCESS: usize = 10;
pub const GLOBAL_SHM_MEMORY_LIMIT: usize = 500 * 1024 * 1024; // 500MB total

/// Shared memory error types
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error", content = "details")]
pub enum ShmError {
    /// Segment not found
    #[error("Segment not found: {0}")]
    NotFound(u32),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid size
    #[error("Invalid size: {0}")]
    InvalidSize(String),

    /// Invalid offset or size range
    #[error("Invalid offset or size: offset {offset}, size {size}, segment size {segment_size}")]
    InvalidRange {
        offset: usize,
        size: usize,
        segment_size: usize,
    },

    /// Segment size exceeds maximum allowed
    #[error("Segment size exceeds limit: requested {requested}, max {max}")]
    SizeExceeded { requested: usize, max: usize },

    /// Process has too many segments
    #[error("Process segment limit exceeded: {0}/{1}")]
    ProcessLimitExceeded(usize, usize),

    /// Global memory limit exceeded
    #[error("Global shared memory limit exceeded: {0}/{1} bytes")]
    GlobalMemoryExceeded(usize, usize),

    /// Memory allocation failed
    #[error("Memory allocation failed: {0}")]
    AllocationFailed(String),
}

// Convert ShmError to IpcError
impl From<ShmError> for IpcError {
    fn from(err: ShmError) -> Self {
        match err {
            ShmError::NotFound(id) => IpcError::NotFound(format!("Shared memory segment {}", id)),
            ShmError::PermissionDenied(msg) => IpcError::PermissionDenied(msg),
            ShmError::InvalidSize(msg) => IpcError::InvalidOperation(msg),
            ShmError::InvalidRange {
                offset,
                size,
                segment_size,
            } => IpcError::InvalidOperation(format!(
                "Invalid range: offset {}, size {}, segment size {}",
                offset, size, segment_size
            )),
            ShmError::SizeExceeded { requested, max } => IpcError::LimitExceeded(format!(
                "Segment size exceeds limit: requested {}, max {}",
                requested, max
            )),
            ShmError::ProcessLimitExceeded(current, max) => IpcError::LimitExceeded(format!(
                "Process segment limit exceeded: {}/{}",
                current, max
            )),
            ShmError::GlobalMemoryExceeded(current, max) => IpcError::LimitExceeded(format!(
                "Global shared memory limit exceeded: {}/{} bytes",
                current, max
            )),
            ShmError::AllocationFailed(msg) => {
                IpcError::InvalidOperation(format!("Memory allocation failed: {}", msg))
            }
        }
    }
}

/// Shared memory segment statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShmStats {
    pub id: ShmId,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub size: Size,
    pub owner_pid: Pid,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub attached_pids: Vec<Pid>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub read_only_pids: Vec<Pid>,
}

impl ShmStats {
    /// Serialize using bincode for internal shared memory operations
    ///
    /// This provides better performance than JSON for shared memory metadata.
    pub fn to_bincode_bytes(&self) -> Result<Vec<u8>, String> {
        crate::core::bincode::to_vec(self)
            .map_err(|e| format!("Failed to serialize shm stats with bincode: {}", e))
    }

    /// Deserialize from bincode format
    pub fn from_bincode_bytes(bytes: &[u8]) -> Result<Self, String> {
        crate::core::bincode::from_slice(bytes)
            .map_err(|e| format!("Failed to deserialize shm stats with bincode: {}", e))
    }
}

// Implement BincodeSerializable for ShmStats
impl crate::core::traits::BincodeSerializable for ShmStats {}

/// Shared memory permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShmPermission {
    /// Read and write access
    ReadWrite,
    /// Read-only access
    ReadOnly,
}

// Implement BincodeSerializable for ShmPermission
impl crate::core::traits::BincodeSerializable for ShmPermission {}

impl ShmPermission {
    /// Check if this permission allows reading
    pub fn can_read(&self) -> bool {
        true // Both permissions allow reading
    }

    /// Check if this permission allows writing
    pub fn can_write(&self) -> bool {
        matches!(self, ShmPermission::ReadWrite)
    }

    /// Check if this permission is at least as permissive as the required permission
    pub fn satisfies(&self, required: ShmPermission) -> bool {
        match (self, required) {
            (ShmPermission::ReadWrite, _) => true,
            (ShmPermission::ReadOnly, ShmPermission::ReadOnly) => true,
            _ => false,
        }
    }
}
