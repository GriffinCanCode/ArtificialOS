/*!
 * Syscall Error Types
 * Defines error types for syscall operations
 */

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Syscall operation errors with rich context
#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "error_type", content = "details")]
#[non_exhaustive]
pub enum SyscallError {
    /// Permission denied for the requested operation
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Operation failed with an error message
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Invalid argument provided to syscall
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Resource not found (file, process, etc.)
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Resource temporarily unavailable
    #[error("Resource unavailable: {0}")]
    Unavailable(String),

    /// I/O error occurred
    #[error("I/O error: {0}")]
    IoError(String),

    /// Feature not yet implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Required manager/subsystem not available
    #[error("Manager not available: {0}")]
    ManagerNotAvailable(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl SyscallError {
    /// Create a permission denied error
    #[inline]
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create an operation failed error
    #[inline]
    pub fn operation_failed(msg: impl Into<String>) -> Self {
        Self::OperationFailed(msg.into())
    }

    /// Create an invalid argument error
    #[inline]
    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Self::InvalidArgument(msg.into())
    }

    /// Create a not found error
    #[inline]
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a manager not available error
    #[inline]
    pub fn manager_not_available(subsystem: impl Into<String>) -> Self {
        Self::ManagerNotAvailable(subsystem.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_error_helpers() {
        let err = SyscallError::permission_denied("test");
        assert!(matches!(err, SyscallError::PermissionDenied(_)));

        let err = SyscallError::not_found("missing");
        assert!(matches!(err, SyscallError::NotFound(_)));
    }
}
