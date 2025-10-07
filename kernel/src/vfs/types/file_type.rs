/*!
 * VFS File Type Enum
 * Defines the type of filesystem objects
 */

use serde::{Deserialize, Serialize};
use std::fmt;

/// File type enumeration with complete serde support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    File,
    Directory,
    Symlink,
    #[serde(rename = "block_device")]
    BlockDevice,
    #[serde(rename = "char_device")]
    CharDevice,
    Fifo,
    Socket,
    Unknown,
}

impl Default for FileType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileType::File => write!(f, "file"),
            FileType::Directory => write!(f, "directory"),
            FileType::Symlink => write!(f, "symlink"),
            FileType::BlockDevice => write!(f, "block device"),
            FileType::CharDevice => write!(f, "char device"),
            FileType::Fifo => write!(f, "fifo"),
            FileType::Socket => write!(f, "socket"),
            FileType::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_display() {
        assert_eq!(FileType::File.to_string(), "file");
        assert_eq!(FileType::Directory.to_string(), "directory");
        assert_eq!(FileType::Symlink.to_string(), "symlink");
    }

    #[test]
    fn test_file_type_serialization() {
        let ft = FileType::BlockDevice;
        let json = serde_json::to_string(&ft).unwrap();
        assert_eq!(json, "\"block_device\"");

        let deserialized: FileType = serde_json::from_str(&json).unwrap();
        assert_eq!(ft, deserialized);
    }
}
