/*!
 * Observable Wrapper - Add events to any FileSystem
 * Decorator pattern to add observability without modifying existing implementations
 */

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast;

use super::observable::{EventBroadcaster, FileEvent, Observable};
use super::traits::{FileSystem, OpenFile};
use super::types::*;

/// Wrapper that adds observability to any FileSystem
pub struct ObservableFS<F: FileSystem> {
    /// Inner filesystem implementation
    inner: Arc<F>,

    /// Event broadcaster
    broadcaster: EventBroadcaster,
}

impl<F: FileSystem> ObservableFS<F> {
    /// Wrap a filesystem with observability
    pub fn new(inner: F) -> Self {
        Self {
            inner: Arc::new(inner),
            broadcaster: EventBroadcaster::default(),
        }
    }

    /// Wrap an Arc'd filesystem
    pub fn from_arc(inner: Arc<F>) -> Self {
        Self {
            inner,
            broadcaster: EventBroadcaster::default(),
        }
    }

    /// Get reference to inner filesystem
    pub fn inner(&self) -> &F {
        &self.inner
    }
}

impl<F: FileSystem> Observable for ObservableFS<F> {
    fn subscribe(&self) -> broadcast::Receiver<FileEvent> {
        self.broadcaster.subscribe()
    }

    fn emit(&self, event: FileEvent) {
        self.broadcaster.emit(event);
    }
}

impl<F: FileSystem> FileSystem for ObservableFS<F> {
    fn read(&self, path: &Path) -> VfsResult<Vec<u8>> {
        self.inner.read(path)
    }

    fn write(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        let existed = self.inner.exists(path);
        let result = self.inner.write(path, data);

        if result.is_ok() {
            if existed {
                self.emit(FileEvent::Modified {
                    path: path.to_path_buf(),
                });
            } else {
                self.emit(FileEvent::Created {
                    path: path.to_path_buf(),
                });
            }
        }

        result
    }

    fn append(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        let result = self.inner.append(path, data);

        if result.is_ok() {
            self.emit(FileEvent::Modified {
                path: path.to_path_buf(),
            });
        }

        result
    }

    fn create(&self, path: &Path) -> VfsResult<()> {
        let result = self.inner.create(path);

        if result.is_ok() {
            self.emit(FileEvent::Created {
                path: path.to_path_buf(),
            });
        }

        result
    }

    fn delete(&self, path: &Path) -> VfsResult<()> {
        let result = self.inner.delete(path);

        if result.is_ok() {
            self.emit(FileEvent::Deleted {
                path: path.to_path_buf(),
            });
        }

        result
    }

    fn exists(&self, path: &Path) -> bool {
        self.inner.exists(path)
    }

    fn metadata(&self, path: &Path) -> VfsResult<Metadata> {
        self.inner.metadata(path)
    }

    fn list_dir(&self, path: &Path) -> VfsResult<Vec<Entry>> {
        self.inner.list_dir(path)
    }

    fn create_dir(&self, path: &Path) -> VfsResult<()> {
        let result = self.inner.create_dir(path);

        if result.is_ok() {
            self.emit(FileEvent::Created {
                path: path.to_path_buf(),
            });
        }

        result
    }

    fn remove_dir(&self, path: &Path) -> VfsResult<()> {
        let result = self.inner.remove_dir(path);

        if result.is_ok() {
            self.emit(FileEvent::Deleted {
                path: path.to_path_buf(),
            });
        }

        result
    }

    fn remove_dir_all(&self, path: &Path) -> VfsResult<()> {
        let result = self.inner.remove_dir_all(path);

        if result.is_ok() {
            self.emit(FileEvent::Deleted {
                path: path.to_path_buf(),
            });
        }

        result
    }

    fn copy(&self, from: &Path, to: &Path) -> VfsResult<()> {
        let result = self.inner.copy(from, to);

        if result.is_ok() {
            self.emit(FileEvent::Created {
                path: to.to_path_buf(),
            });
        }

        result
    }

    fn rename(&self, from: &Path, to: &Path) -> VfsResult<()> {
        let result = self.inner.rename(from, to);

        if result.is_ok() {
            self.emit(FileEvent::Renamed {
                from: from.to_path_buf(),
                to: to.to_path_buf(),
            });
        }

        result
    }

    fn symlink(&self, src: &Path, dst: &Path) -> VfsResult<()> {
        let result = self.inner.symlink(src, dst);

        if result.is_ok() {
            self.emit(FileEvent::Created {
                path: dst.to_path_buf(),
            });
        }

        result
    }

    fn read_link(&self, path: &Path) -> VfsResult<PathBuf> {
        self.inner.read_link(path)
    }

    fn truncate(&self, path: &Path, size: u64) -> VfsResult<()> {
        let result = self.inner.truncate(path, size);

        if result.is_ok() {
            self.emit(FileEvent::Modified {
                path: path.to_path_buf(),
            });
        }

        result
    }

    fn set_permissions(&self, path: &Path, perms: Permissions) -> VfsResult<()> {
        let result = self.inner.set_permissions(path, perms);

        if result.is_ok() {
            self.emit(FileEvent::Modified {
                path: path.to_path_buf(),
            });
        }

        result
    }

    fn open(&self, path: &Path, flags: OpenFlags, mode: OpenMode) -> VfsResult<Box<dyn OpenFile>> {
        self.inner.open(path, flags, mode)
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn readonly(&self) -> bool {
        self.inner.readonly()
    }
}

impl<F: FileSystem> Clone for ObservableFS<F> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            broadcaster: self.broadcaster.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::MemFS;

    #[tokio::test]
    async fn test_observable_write() {
        let fs = ObservableFS::new(MemFS::new());
        let mut rx = fs.subscribe();

        // Write a file
        fs.write(Path::new("/test.txt"), b"hello").unwrap();

        // Should receive Created event
        let event = rx.recv().await.unwrap();
        assert_eq!(event, FileEvent::Created {
            path: PathBuf::from("/test.txt"),
        });

        // Write again
        fs.write(Path::new("/test.txt"), b"world").unwrap();

        // Should receive Modified event
        let event = rx.recv().await.unwrap();
        assert_eq!(event, FileEvent::Modified {
            path: PathBuf::from("/test.txt"),
        });
    }

    #[tokio::test]
    async fn test_observable_delete() {
        let fs = ObservableFS::new(MemFS::new());
        let mut rx = fs.subscribe();

        fs.write(Path::new("/test.txt"), b"hello").unwrap();
        let _ = rx.recv().await; // Consume Created event

        // Delete
        fs.delete(Path::new("/test.txt")).unwrap();

        // Should receive Deleted event
        let event = rx.recv().await.unwrap();
        assert_eq!(event, FileEvent::Deleted {
            path: PathBuf::from("/test.txt"),
        });
    }

    #[tokio::test]
    async fn test_observable_rename() {
        let fs = ObservableFS::new(MemFS::new());
        let mut rx = fs.subscribe();

        fs.write(Path::new("/old.txt"), b"data").unwrap();
        let _ = rx.recv().await; // Consume Created event

        // Rename
        fs.rename(Path::new("/old.txt"), Path::new("/new.txt")).unwrap();

        // Should receive Renamed event
        let event = rx.recv().await.unwrap();
        assert_eq!(event, FileEvent::Renamed {
            from: PathBuf::from("/old.txt"),
            to: PathBuf::from("/new.txt"),
        });
    }
}

