/*!
 * File Handle Abstraction
 * Unified handle for VFS and standard filesystem operations
 */

use crate::vfs::{OpenFile, VfsError, VfsResult};
use parking_lot::RwLock;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

/// File handle wrapping VFS OpenFile trait
///
/// # Performance
/// - RwLock for shared mutable access (aligned with FdManager pattern)
/// - Trait object for zero-cost abstraction across filesystem backends
pub struct FileHandle {
    inner: RwLock<Box<dyn OpenFile>>,
}

impl FileHandle {
    /// Create from VFS OpenFile
    #[inline]
    pub fn from_vfs(file: Box<dyn OpenFile>) -> Self {
        Self {
            inner: RwLock::new(file),
        }
    }

    /// Create from std::fs::File (fallback when VFS unavailable)
    #[inline]
    pub fn from_std(file: File) -> Self {
        Self {
            inner: RwLock::new(Box::new(StdFileHandle { file })),
        }
    }

    /// Read into buffer
    pub fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.write().read(buf)
    }

    /// Write from buffer
    pub fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write().write(buf)
    }

    /// Seek to position
    pub fn seek(&self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.write().seek(pos)
    }

    /// Sync to storage
    pub fn sync(&self) -> VfsResult<()> {
        self.inner.write().sync()
    }

    /// Sync data only (no metadata)
    pub fn sync_data(&self) -> std::io::Result<()> {
        // VFS sync() syncs all, map to same operation
        self.sync()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }

    /// Set file length
    pub fn set_len(&self, size: u64) -> VfsResult<()> {
        self.inner.write().set_len(size)
    }
}

/// Standard file handle implementing OpenFile
///
/// Adapter for std::fs::File to VFS OpenFile trait
struct StdFileHandle {
    file: File,
}

impl Read for StdFileHandle {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for StdFileHandle {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Seek for StdFileHandle {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl OpenFile for StdFileHandle {
    fn sync(&mut self) -> VfsResult<()> {
        self.file
            .sync_all()
            .map_err(|e| VfsError::IoError(e.to_string()))
    }

    fn metadata(&self) -> VfsResult<crate::vfs::Metadata> {
        let md = self
            .file
            .metadata()
            .map_err(|e| VfsError::IoError(e.to_string()))?;

        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::PermissionsExt;
            md.permissions().mode()
        };
        #[cfg(not(unix))]
        let mode = if md.permissions().readonly() {
            0o444
        } else {
            0o644
        };

        Ok(crate::vfs::Metadata {
            file_type: if md.is_dir() {
                crate::vfs::FileType::Directory
            } else if md.is_file() {
                crate::vfs::FileType::File
            } else {
                crate::vfs::FileType::Unknown
            },
            size: md.len(),
            permissions: crate::vfs::Permissions::new(mode),
            modified: md.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            accessed: md.accessed().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            created: md.created().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
        })
    }

    fn set_len(&mut self, size: u64) -> VfsResult<()> {
        self.file
            .set_len(size)
            .map_err(|e| VfsError::IoError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use tempfile::NamedTempFile;

    #[test]
    fn test_std_file_handle() {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(b"test content").unwrap();
        temp.flush().unwrap();

        let file = File::open(temp.path()).unwrap();
        let handle = FileHandle::from_std(file);

        let mut buf = vec![0u8; 12];
        assert_eq!(handle.read(&mut buf).unwrap(), 12);
        assert_eq!(&buf, b"test content");
    }

    #[test]
    fn test_file_handle_seek() {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(b"0123456789").unwrap();
        temp.flush().unwrap();

        let file = File::open(temp.path()).unwrap();
        let handle = FileHandle::from_std(file);

        // Seek to position 5
        assert_eq!(handle.seek(SeekFrom::Start(5)).unwrap(), 5);

        let mut buf = vec![0u8; 5];
        assert_eq!(handle.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"56789");
    }
}
