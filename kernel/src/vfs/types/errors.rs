/*!
 * VFS Error Types
 * Structured, type-safe error handling for filesystem operations
 */

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// VFS operation result
///
/// # Must Use
/// VFS operations can fail and must be handled to prevent data loss
#[must_use = "VFS operations can fail and must be handled"]
pub type VfsResult<T> = Result<T, VfsError>;

/// VFS errors with structured, type-safe error handling
///
/// All error variants include context strings that should be non-empty.
/// Serialization uses tagged enum pattern for type safety.
#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "error", content = "details")]
pub enum VfsError {
    #[error("Not found: {0}")]
    NotFound(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Already exists: {0}")]
    AlreadyExists(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Permission denied: {0}")]
    PermissionDenied(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Not a directory: {0}")]
    NotADirectory(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Is a directory: {0}")]
    IsADirectory(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Invalid path: {0}")]
    InvalidPath(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("I/O error: {0}")]
    IoError(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Not supported: {0}")]
    NotSupported(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Out of space")]
    OutOfSpace,

    #[error("Invalid argument: {0}")]
    InvalidArgument(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("File too large")]
    FileTooLarge,

    #[error("Read-only filesystem")]
    ReadOnly,

    #[error("Cross-device link")]
    CrossDevice,
}

/// Deserialize and validate non-empty string for error messages
pub(super) fn deserialize_nonempty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Err(serde::de::Error::custom("error message must not be empty"));
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_error_validation() {
        // Valid error with non-empty message
        let error = VfsError::NotFound("file.txt".to_string());
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: VfsError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);

        // Invalid error with empty message should fail deserialization
        let invalid_json = r#"{"error":"not_found","details":""}"#;
        let result: Result<VfsError, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }
}
