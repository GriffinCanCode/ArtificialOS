/*!
 * VFS Directory Entry
 * Represents entries in a directory with validation
 */

use super::errors::VfsError;
use super::file_type::FileType;
use crate::core::data_structures::InlineString;
use crate::core::serialization::serde::is_default;
use serde::{Deserialize, Deserializer, Serialize};

/// Directory entry with type-safe construction and validation
///
/// Entry names must be non-empty and cannot contain null bytes or path separators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "deserialize_valid_filename")]
    pub name: InlineString,
    #[serde(skip_serializing_if = "is_default", default)]
    pub file_type: FileType,
}

impl Entry {
    /// Create a new directory entry with validation
    #[must_use = "validation result must be checked"]
    pub fn new(name: impl Into<InlineString>, file_type: FileType) -> Result<Self, VfsError> {
        let name = name.into();
        if name.is_empty() {
            return Err(VfsError::InvalidPath("entry name cannot be empty".into()));
        }
        if name.contains('\0') {
            return Err(VfsError::InvalidPath(
                "entry name cannot contain null bytes".into(),
            ));
        }
        if name.contains('/') || name.contains('\\') {
            return Err(VfsError::InvalidPath(
                "entry name cannot contain path separators".into(),
            ));
        }
        Ok(Self { name, file_type })
    }

    /// Create a new entry without validation (internal use)
    pub(crate) fn new_unchecked(name: impl Into<InlineString>, file_type: FileType) -> Self {
        Self {
            name: name.into(),
            file_type,
        }
    }

    /// Create a file entry
    #[inline]
    #[must_use = "validation result must be checked"]
    pub fn file(name: impl Into<InlineString>) -> Result<Self, VfsError> {
        Self::new(name, FileType::File)
    }

    /// Create a directory entry
    #[inline]
    #[must_use = "validation result must be checked"]
    pub fn directory(name: impl Into<InlineString>) -> Result<Self, VfsError> {
        Self::new(name, FileType::Directory)
    }

    /// Check if this is a directory entry
    #[inline]
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self.file_type, FileType::Directory)
    }

    /// Check if this is a file entry
    #[inline]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self.file_type, FileType::File)
    }

    /// Validate entry name
    #[must_use = "validation result must be checked"]
    pub fn validate_name(name: &str) -> Result<(), VfsError> {
        if name.is_empty() {
            return Err(VfsError::InvalidPath("entry name cannot be empty".into()));
        }
        if name.contains('\0') {
            return Err(VfsError::InvalidPath(
                "entry name cannot contain null bytes".into(),
            ));
        }
        if name.contains('/') || name.contains('\\') {
            return Err(VfsError::InvalidPath(
                "entry name cannot contain path separators".into(),
            ));
        }
        Ok(())
    }
}

/// Deserialize and validate filename
fn deserialize_valid_filename<'de, D>(deserializer: D) -> Result<InlineString, D::Error>
where
    D: Deserializer<'de>,
{
    let name = InlineString::deserialize(deserializer)?;
    if name.is_empty() {
        return Err(serde::de::Error::custom("entry name cannot be empty"));
    }
    if name.contains('\0') {
        return Err(serde::de::Error::custom(
            "entry name cannot contain null bytes",
        ));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(serde::de::Error::custom(
            "entry name cannot contain path separators",
        ));
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_helpers() {
        let entry = Entry::file("test.txt").unwrap();
        assert_eq!(entry.name, "test.txt");
        assert!(entry.is_file());
        assert!(!entry.is_dir());

        let entry = Entry::directory("folder").unwrap();
        assert_eq!(entry.name, "folder");
        assert!(entry.is_dir());
        assert!(!entry.is_file());
    }

    #[test]
    fn test_entry_validation() {
        // Valid names
        assert!(Entry::file("test.txt").is_ok());
        assert!(Entry::file("my-file_2.txt").is_ok());

        // Invalid names
        assert!(Entry::file("").is_err());
        assert!(Entry::file("test/file.txt").is_err());
        assert!(Entry::file("test\\file.txt").is_err());
        assert!(Entry::file("test\0file.txt").is_err());

        // Validate function
        assert!(Entry::validate_name("valid.txt").is_ok());
        assert!(Entry::validate_name("").is_err());
        assert!(Entry::validate_name("invalid/path").is_err());
    }

    #[test]
    fn test_entry_serialization() {
        let entry = Entry::file("test.txt").unwrap();
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: Entry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);

        // Test invalid entry during deserialization
        let invalid_json = r#"{"name":"test/file.txt","file_type":"file"}"#;
        let result: Result<Entry, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());

        let empty_name_json = r#"{"name":"","file_type":"file"}"#;
        let result: Result<Entry, _> = serde_json::from_str(empty_name_json);
        assert!(result.is_err());
    }
}
