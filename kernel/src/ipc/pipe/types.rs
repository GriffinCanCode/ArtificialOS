/*!
 * Pipe Types
 * Common types, constants, and errors for pipes
 */

use super::super::types::{IpcError, PipeId};
use crate::core::serde::{is_false, is_zero_usize};
use crate::core::types::{Pid, Size};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Pipe limits to prevent resource exhaustion
pub const DEFAULT_PIPE_CAPACITY: usize = 65536; // 64KB (Linux default)
pub const MAX_PIPE_CAPACITY: usize = 1024 * 1024; // 1MB max
pub const MAX_PIPES_PER_PROCESS: usize = 100;
pub const GLOBAL_PIPE_MEMORY_LIMIT: usize = 50 * 1024 * 1024; // 50MB total

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
