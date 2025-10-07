/*!
 * Syscall Result Types
 * Defines result types for syscall operations
 */

use super::errors::SyscallError;
use crate::core::serde::skip_serializing_none;
use serde::{Deserialize, Serialize};

/// System call result with modern serde patterns
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum SyscallResult {
    /// Successful operation
    Success {
        /// Optional data payload (binary)
        data: Option<Vec<u8>>,
    },
    /// Operation failed with error message
    Error {
        /// Human-readable error message
        message: String,
    },
    /// Permission denied with reason
    PermissionDenied {
        /// Reason for permission denial
        reason: String,
    },
}

impl SyscallResult {
    #[inline]
    #[must_use]
    pub fn success() -> Self {
        Self::Success { data: None }
    }

    #[inline]
    #[must_use]
    pub fn success_with_data(data: Vec<u8>) -> Self {
        Self::Success { data: Some(data) }
    }

    #[inline]
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    #[inline]
    #[must_use]
    pub fn permission_denied(reason: impl Into<String>) -> Self {
        Self::PermissionDenied {
            reason: reason.into(),
        }
    }

    /// Check if result is successful
    #[inline]
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if result is error
    #[inline]
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Check if result is permission denied
    #[inline]
    #[must_use]
    pub const fn is_permission_denied(&self) -> bool {
        matches!(self, Self::PermissionDenied { .. })
    }

    /// Extract data if successful
    #[inline]
    #[must_use]
    pub const fn data(&self) -> Option<&Vec<u8>> {
        match self {
            Self::Success { data } => data.as_ref(),
            _ => None,
        }
    }
}

/// Convert from SyscallError to SyscallResult
impl From<SyscallError> for SyscallResult {
    fn from(err: SyscallError) -> Self {
        match err {
            SyscallError::PermissionDenied(msg) => Self::PermissionDenied { reason: msg },
            other => Self::Error {
                message: other.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_result_methods() {
        let success = SyscallResult::success();
        assert!(success.is_success());
        assert!(!success.is_error());

        let error = SyscallResult::error("failed");
        assert!(error.is_error());
        assert!(!error.is_success());
    }

    #[test]
    fn test_syscall_result_serialization() {
        let result = SyscallResult::success_with_data(vec![1, 2, 3]);
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SyscallResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, deserialized);
    }
}
