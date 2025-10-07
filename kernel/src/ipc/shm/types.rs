/*!
 * Shared Memory Types
 * Common types, constants, and errors for shared memory
 */

use super::super::types::{IpcError, ShmId};
use crate::core::serde::is_empty_vec;
use crate::core::types::{Pid, Size};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Shared memory limits
pub const MAX_SEGMENT_SIZE: usize = 100 * 1024 * 1024; // 100MB per segment
pub const MAX_SEGMENTS_PER_PROCESS: usize = 10;
pub const GLOBAL_SHM_MEMORY_LIMIT: usize = 500 * 1024 * 1024; // 500MB total

/// Shared memory error types
#[derive(Debug, Error)]
pub enum ShmError {
    #[error("Segment not found: {0}")]
    NotFound(u32),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid size: {0}")]
    InvalidSize(String),

    #[error("Invalid offset or size: offset {offset}, size {size}, segment size {segment_size}")]
    InvalidRange {
        offset: usize,
        size: usize,
        segment_size: usize,
    },

    #[error("Segment size exceeds limit: requested {requested}, max {max}")]
    SizeExceeded { requested: usize, max: usize },

    #[error("Process segment limit exceeded: {0}/{1}")]
    ProcessLimitExceeded(usize, usize),

    #[error("Global shared memory limit exceeded: {0}/{1} bytes")]
    GlobalMemoryExceeded(usize, usize),

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
    pub size: Size,
    pub owner_pid: Pid,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub attached_pids: Vec<Pid>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub read_only_pids: Vec<Pid>,
}

/// Shared memory permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmPermission {
    ReadWrite,
    ReadOnly,
}
