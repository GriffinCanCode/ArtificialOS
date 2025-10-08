/*!
 * File Handle Implementation
 * In-memory file handle for read/write operations
 */

use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use super::super::traits::{FileSystem, OpenFile};
use super::super::types::*;
use super::MemFS;

/// In-memory file handle
pub(super) struct MemFile {
    pub fs: MemFS,
    pub path: PathBuf,
    pub cursor: Cursor<Vec<u8>>,
    pub flags: OpenFlags,
}

impl Read for MemFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if !self.flags.read {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "file not opened for reading",
            ));
        }
        self.cursor.read(buf)
    }
}

impl Write for MemFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if !self.flags.write {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "file not opened for writing",
            ));
        }

        // Double-check file permissions haven't changed
        if let Ok(metadata) = self.fs.metadata(&self.path) {
            if metadata.permissions.is_readonly() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "file is readonly",
                ));
            }
        }

        self.cursor.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Seek for MemFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.cursor.seek(pos)
    }
}

impl OpenFile for MemFile {
    fn sync(&mut self) -> VfsResult<()> {
        if self.flags.write {
            // Check permissions before syncing
            if let Ok(metadata) = self.fs.metadata(&self.path) {
                if metadata.permissions.is_readonly() {
                    return Err(VfsError::PermissionDenied(format!(
                        "file is readonly: {}",
                        self.path.display()
                    ).into()));
                }
            }
            let data = self.cursor.get_ref().clone();
            self.fs.write(&self.path, &data)?;
        }
        Ok(())
    }

    fn metadata(&self) -> VfsResult<Metadata> {
        self.fs.metadata(&self.path)
    }

    fn set_len(&mut self, size: u64) -> VfsResult<()> {
        // Check permissions before resizing
        if let Ok(metadata) = self.fs.metadata(&self.path) {
            if metadata.permissions.is_readonly() {
                return Err(VfsError::PermissionDenied(format!(
                    "file is readonly: {}",
                    self.path.display()
                ).into()));
            }
        }
        let data = self.cursor.get_mut();
        data.resize(size as usize, 0);
        Ok(())
    }
}

impl Drop for MemFile {
    fn drop(&mut self) {
        // Auto-sync on drop if opened for writing
        let _ = self.sync();
    }
}
