/*!
 * VFS Metadata
 * File metadata including timestamps and permissions
 */

use super::file_type::FileType;
use super::permissions::Permissions;
use crate::core::serde::{is_default, is_zero_u64, serde_as, system_time_micros};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// File metadata with optimized serialization
///
/// Timestamps are serialized as microseconds since UNIX epoch for precision and efficiency.
/// Size and permissions are skipped when they are default values to reduce payload size.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Metadata {
    pub file_type: FileType,
    #[serde(skip_serializing_if = "is_zero_u64", default)]
    pub size: u64,
    #[serde(skip_serializing_if = "is_default", default)]
    pub permissions: Permissions,
    #[serde(with = "system_time_micros")]
    pub modified: SystemTime,
    #[serde(with = "system_time_micros")]
    pub accessed: SystemTime,
    #[serde(with = "system_time_micros")]
    pub created: SystemTime,
}

impl Metadata {
    /// Check if this is a directory
    ///
    /// # Performance
    /// Hot path - very frequently called in path resolution
    #[inline(always)]
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self.file_type, FileType::Directory)
    }

    /// Check if this is a regular file
    ///
    /// # Performance
    /// Hot path - very frequently called in file operations
    #[inline(always)]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self.file_type, FileType::File)
    }

    /// Check if this is a symbolic link
    ///
    /// # Performance
    /// Hot path - frequently called during path resolution
    #[inline(always)]
    #[must_use]
    pub const fn is_symlink(&self) -> bool {
        matches!(self.file_type, FileType::Symlink)
    }

    /// Check if this is a special file (device, fifo, socket)
    #[inline]
    #[must_use]
    pub const fn is_special(&self) -> bool {
        matches!(
            self.file_type,
            FileType::BlockDevice | FileType::CharDevice | FileType::Fifo | FileType::Socket
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_helpers() {
        let metadata = Metadata {
            file_type: FileType::File,
            size: 100,
            permissions: Permissions::readwrite(),
            modified: SystemTime::now(),
            accessed: SystemTime::now(),
            created: SystemTime::now(),
        };

        assert!(metadata.is_file());
        assert!(!metadata.is_dir());
        assert!(!metadata.is_symlink());
        assert!(!metadata.is_special());

        let dir_metadata = Metadata {
            file_type: FileType::Directory,
            size: 0,
            permissions: Permissions::executable(),
            modified: SystemTime::now(),
            accessed: SystemTime::now(),
            created: SystemTime::now(),
        };

        assert!(dir_metadata.is_dir());
        assert!(!dir_metadata.is_file());
    }
}
