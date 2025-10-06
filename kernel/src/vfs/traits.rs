/*!
 * VFS Traits
 * Core filesystem abstraction traits
 */

use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use super::types::*;

/// Virtual filesystem trait
///
/// All filesystem implementations must implement this trait.
/// Operations should be atomic where possible and return
/// appropriate errors on failure.
pub trait FileSystem: Send + Sync {
    /// Read entire file contents
    fn read(&self, path: &Path) -> VfsResult<Vec<u8>>;

    /// Write entire file contents (create or overwrite)
    fn write(&self, path: &Path, data: &[u8]) -> VfsResult<()>;

    /// Append data to file
    fn append(&self, path: &Path, data: &[u8]) -> VfsResult<()>;

    /// Create empty file
    fn create(&self, path: &Path) -> VfsResult<()>;

    /// Delete file
    fn delete(&self, path: &Path) -> VfsResult<()>;

    /// Check if file/directory exists
    fn exists(&self, path: &Path) -> bool;

    /// Get file metadata
    fn metadata(&self, path: &Path) -> VfsResult<Metadata>;

    /// List directory contents
    fn list_dir(&self, path: &Path) -> VfsResult<Vec<Entry>>;

    /// Create directory (including parents)
    fn create_dir(&self, path: &Path) -> VfsResult<()>;

    /// Remove directory (must be empty)
    fn remove_dir(&self, path: &Path) -> VfsResult<()>;

    /// Remove directory recursively
    fn remove_dir_all(&self, path: &Path) -> VfsResult<()>;

    /// Copy file
    fn copy(&self, from: &Path, to: &Path) -> VfsResult<()>;

    /// Move/rename file
    fn rename(&self, from: &Path, to: &Path) -> VfsResult<()>;

    /// Create symbolic link
    fn symlink(&self, src: &Path, dst: &Path) -> VfsResult<()>;

    /// Read symbolic link target
    fn read_link(&self, path: &Path) -> VfsResult<PathBuf>;

    /// Truncate file to specified size
    fn truncate(&self, path: &Path, size: u64) -> VfsResult<()>;

    /// Set file permissions
    fn set_permissions(&self, path: &Path, perms: Permissions) -> VfsResult<()>;

    /// Open file with specified flags and mode
    fn open(&self, path: &Path, flags: OpenFlags, mode: OpenMode) -> VfsResult<Box<dyn OpenFile>>;

    /// Get filesystem name/type
    fn name(&self) -> &str;

    /// Check if filesystem is read-only
    fn readonly(&self) -> bool {
        false
    }
}

/// Open file handle trait
///
/// Represents an open file with read/write/seek capabilities.
/// Automatically closed when dropped.
pub trait OpenFile: Read + Write + Seek + Send + Sync {
    /// Sync file data to storage
    fn sync(&mut self) -> VfsResult<()>;

    /// Get file metadata
    fn metadata(&self) -> VfsResult<Metadata>;

    /// Set file length
    fn set_len(&mut self, size: u64) -> VfsResult<()>;
}

/// Filesystem builder trait for configuration
pub trait FileSystemBuilder {
    type Output: FileSystem;

    /// Build the filesystem instance
    fn build(self) -> VfsResult<Self::Output>;
}
