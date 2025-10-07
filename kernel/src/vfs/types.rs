/*!
 * VFS Types
 * Shared types for filesystem operations
 */

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;
use thiserror::Error;

/// VFS operation result
pub type VfsResult<T> = Result<T, VfsError>;

/// VFS errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum VfsError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Not a directory: {0}")]
    NotADirectory(String),

    #[error("Is a directory: {0}")]
    IsADirectory(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("Out of space")]
    OutOfSpace,

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("File too large")]
    FileTooLarge,

    #[error("Read-only filesystem")]
    ReadOnly,

    #[error("Cross-device link")]
    CrossDevice,
}

/// File type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    BlockDevice,
    CharDevice,
    Fifo,
    Socket,
    Unknown,
}

/// File permissions (Unix-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    pub mode: u32,
}

impl Permissions {
    pub fn new(mode: u32) -> Self {
        Self { mode }
    }

    pub fn readonly() -> Self {
        Self { mode: 0o444 }
    }

    pub fn readwrite() -> Self {
        Self { mode: 0o644 }
    }

    pub fn is_readonly(&self) -> bool {
        self.mode & 0o200 == 0
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            self.mode &= !0o222;
        } else {
            self.mode |= 0o200;
        }
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::readwrite()
    }
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub file_type: FileType,
    pub size: u64,
    pub permissions: Permissions,
    pub modified: SystemTime,
    pub accessed: SystemTime,
    pub created: SystemTime,
}

impl Metadata {
    pub fn is_dir(&self) -> bool {
        self.file_type == FileType::Directory
    }

    pub fn is_file(&self) -> bool {
        self.file_type == FileType::File
    }

    pub fn is_symlink(&self) -> bool {
        self.file_type == FileType::Symlink
    }
}

/// Directory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub file_type: FileType,
}

impl Entry {
    pub fn new(name: String, file_type: FileType) -> Self {
        Self { name, file_type }
    }
}

/// File open flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenFlags {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub truncate: bool,
    pub create: bool,
    pub create_new: bool,
}

impl OpenFlags {
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub fn write_only() -> Self {
        Self {
            read: false,
            write: true,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub fn create() -> Self {
        Self {
            read: false,
            write: true,
            append: false,
            truncate: false,
            create: true,
            create_new: false,
        }
    }

    pub fn from_posix(flags: u32) -> Self {
        // Extract access mode from lower 2 bits
        let access_mode = flags & 0x0003;
        let read = access_mode == 0x0001 || access_mode == 0x0003;
        let write = access_mode == 0x0002 || access_mode == 0x0003;
        let append = flags & 0x0400 != 0;
        let truncate = flags & 0x0200 != 0;
        let create = flags & 0x0040 != 0;
        let create_new = flags & 0x0080 != 0;

        Self {
            read,
            write,
            append,
            truncate,
            create,
            create_new,
        }
    }
}

/// File open mode (for creation)
#[derive(Debug, Clone, Copy)]
pub struct OpenMode {
    pub permissions: Permissions,
}

impl OpenMode {
    pub fn new(mode: u32) -> Self {
        Self {
            permissions: Permissions::new(mode),
        }
    }
}

impl Default for OpenMode {
    fn default() -> Self {
        Self {
            permissions: Permissions::readwrite(),
        }
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
    fn test_permissions() {
        let mut perms = Permissions::readwrite();
        assert!(!perms.is_readonly());

        perms.set_readonly(true);
        assert!(perms.is_readonly());

        perms.set_readonly(false);
        assert!(!perms.is_readonly());
    }

    #[test]
    fn test_open_flags() {
        let flags = OpenFlags::read_only();
        assert!(flags.read);
        assert!(!flags.write);

        let flags = OpenFlags::from_posix(0x0001);
        assert!(flags.read);
        assert!(!flags.write);

        let flags = OpenFlags::from_posix(0x0002);
        assert!(!flags.read);
        assert!(flags.write);
    }

    #[test]
    fn test_file_type_display() {
        assert_eq!(FileType::File.to_string(), "file");
        assert_eq!(FileType::Directory.to_string(), "directory");
    }
}
