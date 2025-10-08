/*!
 * Mount Manager
 * Manages filesystem mount points and routing
 */

use ahash::RandomState;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::traits::{FileSystem, OpenFile};
use super::types::*;

/// Mount point configuration
#[derive(Debug, Clone)]
pub struct MountPoint {
    pub path: PathBuf,
    pub name: String,
    pub readonly: bool,
}

impl MountPoint {
    pub fn new<P: Into<PathBuf>, S: Into<String>>(path: P, name: S) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            readonly: false,
        }
    }

    pub fn readonly<P: Into<PathBuf>, S: Into<String>>(path: P, name: S) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            readonly: true,
        }
    }
}

/// Internal mount entry with filesystem and options
struct MountEntry {
    fs: Arc<dyn FileSystem>,
    readonly: bool,
}

/// Mount manager for filesystem routing
pub struct MountManager {
    mounts: Arc<DashMap<PathBuf, MountEntry, RandomState>>,
    mount_order: Arc<RwLock<Vec<PathBuf>>>, // Longest paths first for proper resolution
}

impl MountManager {
    /// Create new mount manager
    pub fn new() -> Self {
        Self {
            mounts: Arc::new(DashMap::with_hasher(RandomState::new())),
            mount_order: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Mount a filesystem at specified path
    pub fn mount<P: Into<PathBuf>>(&self, mount_path: P, fs: Arc<dyn FileSystem>) -> VfsResult<()> {
        self.mount_with_options(mount_path, fs, false)
    }

    /// Mount a filesystem at specified path with readonly option
    pub fn mount_with_options<P: Into<PathBuf>>(
        &self,
        mount_path: P,
        fs: Arc<dyn FileSystem>,
        readonly: bool,
    ) -> VfsResult<()> {
        let mount_path = self.normalize_path(&mount_path.into());

        if self.mounts.contains_key(&mount_path) {
            return Err(VfsError::AlreadyExists(format!(
                "mount point already exists: {}",
                mount_path.display()
            )));
        }

        self.mounts
            .insert(mount_path.clone(), MountEntry { fs, readonly });

        // Update mount order (longest paths first)
        let mut order = self.mount_order.write();
        order.push(mount_path);
        order.sort_by(|a, b| b.as_os_str().len().cmp(&a.as_os_str().len()));

        Ok(())
    }

    /// Mount a filesystem using a MountPoint configuration
    pub fn mount_from_config(&self, config: &MountPoint, fs: Arc<dyn FileSystem>) -> VfsResult<()> {
        self.mount_with_options(&config.path, fs, config.readonly)
    }

    /// Unmount filesystem at specified path
    pub fn unmount<P: AsRef<Path>>(&self, mount_path: P) -> VfsResult<()> {
        let mount_path = self.normalize_path(mount_path.as_ref());

        if self.mounts.remove(&mount_path).is_none() {
            return Err(VfsError::NotFound(format!(
                "mount point not found: {}",
                mount_path.display()
            )));
        }

        let mut order = self.mount_order.write();
        order.retain(|p| p != &mount_path);

        Ok(())
    }

    /// Resolve path to (filesystem, relative_path, readonly)
    fn resolve(&self, path: &Path) -> VfsResult<(Arc<dyn FileSystem>, PathBuf, bool)> {
        let path = self.normalize_path(path);
        let order = self.mount_order.read();

        // Find longest matching mount point
        for mount_path in order.iter() {
            if path.starts_with(mount_path) {
                // Atomically get mount entry - handle concurrent unmount race condition
                let entry = self.mounts.get(mount_path).ok_or_else(|| {
                    VfsError::NotFound(format!(
                        "mount point was removed concurrently: {}",
                        mount_path.display()
                    ))
                })?;

                let fs = entry.fs.clone();
                let readonly = entry.readonly;
                let rel_path = if path == *mount_path {
                    PathBuf::from("/")
                } else {
                    path.strip_prefix(mount_path)
                        .map(|p| PathBuf::from("/").join(p))
                        .unwrap_or_else(|_| PathBuf::from("/"))
                };
                return Ok((fs, rel_path, readonly));
            }
        }

        Err(VfsError::NotFound(format!(
            "no filesystem mounted for path: {}",
            path.display()
        )))
    }

    /// Check if mount point allows writes
    fn check_readonly(&self, readonly: bool) -> VfsResult<()> {
        if readonly {
            Err(VfsError::ReadOnly)
        } else {
            Ok(())
        }
    }

    /// Normalize path (make absolute)
    fn normalize_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            PathBuf::from("/").join(path)
        }
    }

    /// List all mount points
    pub fn list_mounts(&self) -> Vec<(PathBuf, String)> {
        self.mounts
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().fs.name().to_string()))
            .collect()
    }

    /// Check if path is mounted
    pub fn is_mounted<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = self.normalize_path(path.as_ref());
        self.mounts.contains_key(&path)
    }
}

impl Default for MountManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MountManager {
    fn clone(&self) -> Self {
        Self {
            mounts: Arc::clone(&self.mounts),
            mount_order: Arc::clone(&self.mount_order),
        }
    }
}

// Implement FileSystem for MountManager to act as unified interface
impl FileSystem for MountManager {
    fn read(&self, path: &Path) -> VfsResult<Vec<u8>> {
        let (fs, rel_path, _) = self.resolve(path)?;
        fs.read(&rel_path)
    }

    fn write(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        let (fs, rel_path, readonly) = self.resolve(path)?;
        self.check_readonly(readonly)?;
        fs.write(&rel_path, data)
    }

    fn append(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        let (fs, rel_path, readonly) = self.resolve(path)?;
        self.check_readonly(readonly)?;
        fs.append(&rel_path, data)
    }

    fn create(&self, path: &Path) -> VfsResult<()> {
        let (fs, rel_path, readonly) = self.resolve(path)?;
        self.check_readonly(readonly)?;
        fs.create(&rel_path)
    }

    fn delete(&self, path: &Path) -> VfsResult<()> {
        let (fs, rel_path, readonly) = self.resolve(path)?;
        self.check_readonly(readonly)?;
        fs.delete(&rel_path)
    }

    fn exists(&self, path: &Path) -> bool {
        self.resolve(path)
            .and_then(|(fs, rel_path, _)| Ok(fs.exists(&rel_path)))
            .unwrap_or(false)
    }

    fn metadata(&self, path: &Path) -> VfsResult<Metadata> {
        let (fs, rel_path, _) = self.resolve(path)?;
        fs.metadata(&rel_path)
    }

    fn list_dir(&self, path: &Path) -> VfsResult<Vec<Entry>> {
        let (fs, rel_path, _) = self.resolve(path)?;
        fs.list_dir(&rel_path)
    }

    fn create_dir(&self, path: &Path) -> VfsResult<()> {
        let (fs, rel_path, readonly) = self.resolve(path)?;
        self.check_readonly(readonly)?;
        fs.create_dir(&rel_path)
    }

    fn remove_dir(&self, path: &Path) -> VfsResult<()> {
        let (fs, rel_path, readonly) = self.resolve(path)?;
        self.check_readonly(readonly)?;
        fs.remove_dir(&rel_path)
    }

    fn remove_dir_all(&self, path: &Path) -> VfsResult<()> {
        let (fs, rel_path, readonly) = self.resolve(path)?;
        self.check_readonly(readonly)?;
        fs.remove_dir_all(&rel_path)
    }

    fn copy(&self, from: &Path, to: &Path) -> VfsResult<()> {
        let (from_fs, from_rel, _) = self.resolve(from)?;
        let (to_fs, to_rel, to_readonly) = self.resolve(to)?;
        self.check_readonly(to_readonly)?;

        // Same filesystem - use native copy
        if Arc::ptr_eq(&from_fs, &to_fs) {
            from_fs.copy(&from_rel, &to_rel)
        } else {
            // Cross-filesystem - read and write
            let data = from_fs.read(&from_rel)?;
            to_fs.write(&to_rel, &data)
        }
    }

    fn rename(&self, from: &Path, to: &Path) -> VfsResult<()> {
        let (from_fs, from_rel, from_readonly) = self.resolve(from)?;
        let (to_fs, to_rel, to_readonly) = self.resolve(to)?;
        self.check_readonly(from_readonly)?;
        self.check_readonly(to_readonly)?;

        // Same filesystem - use native rename
        if Arc::ptr_eq(&from_fs, &to_fs) {
            from_fs.rename(&from_rel, &to_rel)
        } else {
            // Cross-filesystem - copy and delete
            let data = from_fs.read(&from_rel)?;
            to_fs.write(&to_rel, &data)?;
            from_fs.delete(&from_rel)?;
            Ok(())
        }
    }

    fn symlink(&self, src: &Path, dst: &Path) -> VfsResult<()> {
        let (fs, dst_rel, readonly) = self.resolve(dst)?;
        self.check_readonly(readonly)?;
        fs.symlink(src, &dst_rel)
    }

    fn read_link(&self, path: &Path) -> VfsResult<PathBuf> {
        let (fs, rel_path, _) = self.resolve(path)?;
        fs.read_link(&rel_path)
    }

    fn truncate(&self, path: &Path, size: u64) -> VfsResult<()> {
        let (fs, rel_path, readonly) = self.resolve(path)?;
        self.check_readonly(readonly)?;
        fs.truncate(&rel_path, size)
    }

    fn set_permissions(&self, path: &Path, perms: Permissions) -> VfsResult<()> {
        let (fs, rel_path, readonly) = self.resolve(path)?;
        self.check_readonly(readonly)?;
        fs.set_permissions(&rel_path, perms)
    }

    fn open(&self, path: &Path, flags: OpenFlags, mode: OpenMode) -> VfsResult<Box<dyn OpenFile>> {
        let (fs, rel_path, readonly) = self.resolve(path)?;
        // Check readonly only if opening for write
        if flags.write || flags.append || flags.truncate || flags.create || flags.create_new {
            self.check_readonly(readonly)?;
        }
        fs.open(&rel_path, flags, mode)
    }

    fn name(&self) -> &str {
        "mount"
    }

    fn readonly(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::{LocalFS, MemFS};
    use tempfile::TempDir;

    #[test]
    fn test_mount_basic() {
        let mgr = MountManager::new();
        let fs = Arc::new(MemFS::new());

        // Mount at /tmp
        mgr.mount("/tmp", fs.clone()).unwrap();
        assert!(mgr.is_mounted("/tmp"));

        // Unmount
        mgr.unmount("/tmp").unwrap();
        assert!(!mgr.is_mounted("/tmp"));
    }

    #[test]
    fn test_mount_operations() {
        let mgr = MountManager::new();
        let fs = Arc::new(MemFS::new());
        mgr.mount("/data", fs).unwrap();

        // Write through mount manager
        mgr.write(Path::new("/data/test.txt"), b"hello").unwrap();

        // Read through mount manager
        let data = mgr.read(Path::new("/data/test.txt")).unwrap();
        assert_eq!(data, b"hello");

        // Check existence
        assert!(mgr.exists(Path::new("/data/test.txt")));
        assert!(!mgr.exists(Path::new("/data/missing.txt")));
    }

    #[test]
    fn test_multiple_mounts() {
        let mgr = MountManager::new();

        let mem_fs = Arc::new(MemFS::new());
        let temp = TempDir::new().unwrap();
        let local_fs = Arc::new(LocalFS::new(temp.path()));

        mgr.mount("/mem", mem_fs).unwrap();
        mgr.mount("/local", local_fs).unwrap();

        // Write to different filesystems
        mgr.write(Path::new("/mem/test1.txt"), b"memory").unwrap();
        mgr.write(Path::new("/local/test2.txt"), b"local").unwrap();

        // Verify they're separate
        assert_eq!(mgr.read(Path::new("/mem/test1.txt")).unwrap(), b"memory");
        assert_eq!(mgr.read(Path::new("/local/test2.txt")).unwrap(), b"local");

        // Verify isolation
        assert!(!mgr.exists(Path::new("/mem/test2.txt")));
        assert!(!mgr.exists(Path::new("/local/test1.txt")));
    }

    #[test]
    fn test_nested_mounts() {
        let mgr = MountManager::new();

        let fs1 = Arc::new(MemFS::new());
        let fs2 = Arc::new(MemFS::new());

        mgr.mount("/data", fs1).unwrap();
        mgr.mount("/data/special", fs2).unwrap();

        // Write to nested mount
        mgr.write(Path::new("/data/special/file.txt"), b"special")
            .unwrap();
        mgr.write(Path::new("/data/normal.txt"), b"normal").unwrap();

        // Verify resolution
        assert_eq!(
            mgr.read(Path::new("/data/special/file.txt")).unwrap(),
            b"special"
        );
        assert_eq!(mgr.read(Path::new("/data/normal.txt")).unwrap(), b"normal");
    }

    #[test]
    fn test_cross_filesystem_copy() {
        let mgr = MountManager::new();

        let fs1 = Arc::new(MemFS::new());
        let fs2 = Arc::new(MemFS::new());

        mgr.mount("/src", fs1).unwrap();
        mgr.mount("/dst", fs2).unwrap();

        // Write to source
        mgr.write(Path::new("/src/file.txt"), b"content").unwrap();

        // Copy across filesystems
        mgr.copy(Path::new("/src/file.txt"), Path::new("/dst/file.txt"))
            .unwrap();

        // Verify both exist
        assert_eq!(mgr.read(Path::new("/src/file.txt")).unwrap(), b"content");
        assert_eq!(mgr.read(Path::new("/dst/file.txt")).unwrap(), b"content");
    }

    #[test]
    fn test_list_mounts() {
        let mgr = MountManager::new();

        mgr.mount("/data", Arc::new(MemFS::new())).unwrap();
        mgr.mount("/tmp", Arc::new(MemFS::new())).unwrap();

        let mounts = mgr.list_mounts();
        assert_eq!(mounts.len(), 2);
        assert!(mounts.iter().any(|(p, _)| p == &PathBuf::from("/data")));
        assert!(mounts.iter().any(|(p, _)| p == &PathBuf::from("/tmp")));
    }
}
