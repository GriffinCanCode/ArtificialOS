/*!
 * Local Filesystem Backend
 * Wraps std::fs for host filesystem access
 */

use log::info;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::traits::{FileSystem, OpenFile};
use super::types::*;

/// Local filesystem implementation using std::fs
#[derive(Debug, Clone)]
pub struct LocalFS {
    root: PathBuf,
    readonly: bool,
}

impl LocalFS {
    /// Create new local filesystem rooted at specified path
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self {
            root: root.into(),
            readonly: false,
        }
    }

    /// Create read-only local filesystem
    pub fn readonly<P: Into<PathBuf>>(root: P) -> Self {
        Self {
            root: root.into(),
            readonly: true,
        }
    }

    /// Resolve path relative to root
    fn resolve(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        }
    }

    /// Check write permission
    fn check_write(&self) -> VfsResult<()> {
        if self.readonly {
            return Err(VfsError::ReadOnly);
        }
        Ok(())
    }

    /// Convert std::io::Error to VfsError
    fn io_error(e: std::io::Error, context: impl Into<String>) -> VfsError {
        use std::io::ErrorKind;
        match e.kind() {
            ErrorKind::NotFound => VfsError::NotFound(context.into()),
            ErrorKind::PermissionDenied => VfsError::PermissionDenied(context.into()),
            ErrorKind::AlreadyExists => VfsError::AlreadyExists(context.into()),
            _ => VfsError::IoError(format!("{}: {}", context.into(), e)),
        }
    }

    /// Convert std::fs::FileType to VFS FileType
    fn convert_file_type(ft: fs::FileType) -> FileType {
        if ft.is_dir() {
            FileType::Directory
        } else if ft.is_symlink() {
            FileType::Symlink
        } else if ft.is_file() {
            FileType::File
        } else {
            FileType::Unknown
        }
    }

    /// Convert std::fs::Metadata to VFS Metadata
    fn convert_metadata(md: fs::Metadata) -> Metadata {
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

        Metadata {
            file_type: Self::convert_file_type(md.file_type()),
            size: md.len(),
            permissions: Permissions::new(mode),
            modified: md.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            accessed: md.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
            created: md.created().unwrap_or(SystemTime::UNIX_EPOCH),
        }
    }
}

impl FileSystem for LocalFS {
    fn read(&self, path: &Path) -> VfsResult<Vec<u8>> {
        let full_path = self.resolve(path);
        fs::read(&full_path)
            .map_err(|e| Self::io_error(e, format!("read {}", path.display())))
    }

    fn write(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        self.check_write()?;
        let full_path = self.resolve(path);

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Self::io_error(e, format!("create parent dirs for {}", path.display())))?;
        }

        fs::write(&full_path, data)
            .map_err(|e| Self::io_error(e, format!("write {}", path.display())))
    }

    fn append(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        self.check_write()?;
        let full_path = self.resolve(path);

        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&full_path)
            .map_err(|e| Self::io_error(e, format!("open for append {}", path.display())))?;

        file.write_all(data)
            .map_err(|e| Self::io_error(e, format!("append {}", path.display())))
    }

    fn create(&self, path: &Path) -> VfsResult<()> {
        self.check_write()?;
        let full_path = self.resolve(path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Self::io_error(e, format!("create parent dirs for {}", path.display())))?;
        }

        fs::File::create(&full_path)
            .map_err(|e| Self::io_error(e, format!("create {}", path.display())))?;
        Ok(())
    }

    fn delete(&self, path: &Path) -> VfsResult<()> {
        self.check_write()?;
        let full_path = self.resolve(path);
        fs::remove_file(&full_path)
            .map_err(|e| Self::io_error(e, format!("delete {}", path.display())))
    }

    fn exists(&self, path: &Path) -> bool {
        self.resolve(path).exists()
    }

    fn metadata(&self, path: &Path) -> VfsResult<Metadata> {
        let full_path = self.resolve(path);
        let md = fs::metadata(&full_path)
            .map_err(|e| Self::io_error(e, format!("metadata {}", path.display())))?;
        Ok(Self::convert_metadata(md))
    }

    fn list_dir(&self, path: &Path) -> VfsResult<Vec<Entry>> {
        let full_path = self.resolve(path);
        let entries = fs::read_dir(&full_path)
            .map_err(|e| Self::io_error(e, format!("list_dir {}", path.display())))?;

        let mut result = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| Self::io_error(e, format!("read dir entry in {}", path.display())))?;
            let name = entry.file_name().into_string()
                .map_err(|_| VfsError::InvalidPath(format!("invalid UTF-8 in filename")))?;
            let file_type = entry.file_type()
                .map_err(|e| Self::io_error(e, format!("get file type for {}", name)))?;

            result.push(Entry::new(name, Self::convert_file_type(file_type)));
        }

        Ok(result)
    }

    fn create_dir(&self, path: &Path) -> VfsResult<()> {
        self.check_write()?;
        let full_path = self.resolve(path);
        fs::create_dir_all(&full_path)
            .map_err(|e| Self::io_error(e, format!("create_dir {}", path.display())))
    }

    fn remove_dir(&self, path: &Path) -> VfsResult<()> {
        self.check_write()?;
        let full_path = self.resolve(path);
        fs::remove_dir(&full_path)
            .map_err(|e| Self::io_error(e, format!("remove_dir {}", path.display())))
    }

    fn remove_dir_all(&self, path: &Path) -> VfsResult<()> {
        self.check_write()?;
        let full_path = self.resolve(path);
        fs::remove_dir_all(&full_path)
            .map_err(|e| Self::io_error(e, format!("remove_dir_all {}", path.display())))
    }

    fn copy(&self, from: &Path, to: &Path) -> VfsResult<()> {
        self.check_write()?;
        let from_full = self.resolve(from);
        let to_full = self.resolve(to);

        if let Some(parent) = to_full.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Self::io_error(e, format!("create parent dirs for {}", to.display())))?;
        }

        fs::copy(&from_full, &to_full)
            .map_err(|e| Self::io_error(e, format!("copy {} to {}", from.display(), to.display())))?;
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> VfsResult<()> {
        self.check_write()?;
        let from_full = self.resolve(from);
        let to_full = self.resolve(to);

        if let Some(parent) = to_full.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Self::io_error(e, format!("create parent dirs for {}", to.display())))?;
        }

        fs::rename(&from_full, &to_full)
            .map_err(|e| Self::io_error(e, format!("rename {} to {}", from.display(), to.display())))
    }

    fn symlink(&self, src: &Path, dst: &Path) -> VfsResult<()> {
        self.check_write()?;
        let dst_full = self.resolve(dst);

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(src, &dst_full)
                .map_err(|e| Self::io_error(e, format!("symlink {} to {}", src.display(), dst.display())))
        }

        #[cfg(windows)]
        {
            // On Windows, need to know if target is file or dir
            let src_full = self.resolve(src);
            if src_full.is_dir() {
                std::os::windows::fs::symlink_dir(src, &dst_full)
            } else {
                std::os::windows::fs::symlink_file(src, &dst_full)
            }
            .map_err(|e| Self::io_error(e, format!("symlink {} to {}", src.display(), dst.display())))
        }

        #[cfg(not(any(unix, windows)))]
        {
            Err(VfsError::NotSupported("symlinks not supported on this platform".to_string()))
        }
    }

    fn read_link(&self, path: &Path) -> VfsResult<PathBuf> {
        let full_path = self.resolve(path);
        fs::read_link(&full_path)
            .map_err(|e| Self::io_error(e, format!("read_link {}", path.display())))
    }

    fn truncate(&self, path: &Path, size: u64) -> VfsResult<()> {
        self.check_write()?;
        let full_path = self.resolve(path);
        let file = fs::OpenOptions::new()
            .write(true)
            .open(&full_path)
            .map_err(|e| Self::io_error(e, format!("open for truncate {}", path.display())))?;

        file.set_len(size)
            .map_err(|e| Self::io_error(e, format!("truncate {}", path.display())))
    }

    fn set_permissions(&self, path: &Path, perms: Permissions) -> VfsResult<()> {
        self.check_write()?;
        let full_path = self.resolve(path);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let std_perms = fs::Permissions::from_mode(perms.mode);
            fs::set_permissions(&full_path, std_perms)
                .map_err(|e| Self::io_error(e, format!("set_permissions {}", path.display())))
        }

        #[cfg(not(unix))]
        {
            let mut std_perms = fs::metadata(&full_path)
                .map_err(|e| Self::io_error(e, format!("get metadata for {}", path.display())))?
                .permissions();
            std_perms.set_readonly(perms.is_readonly());
            fs::set_permissions(&full_path, std_perms)
                .map_err(|e| Self::io_error(e, format!("set_permissions {}", path.display())))
        }
    }

    fn open(&self, path: &Path, flags: OpenFlags, _mode: OpenMode) -> VfsResult<Box<dyn OpenFile>> {
        if flags.write && self.readonly {
            return Err(VfsError::ReadOnly);
        }

        let full_path = self.resolve(path);
        let mut options = fs::OpenOptions::new();

        options.read(flags.read);
        options.write(flags.write);
        options.append(flags.append);
        options.truncate(flags.truncate);
        options.create(flags.create);
        options.create_new(flags.create_new);

        let file = options.open(&full_path)
            .map_err(|e| Self::io_error(e, format!("open {}", path.display())))?;

        Ok(Box::new(LocalFile { file }))
    }

    fn name(&self) -> &str {
        "local"
    }

    fn readonly(&self) -> bool {
        self.readonly
    }
}

/// Local file handle
struct LocalFile {
    file: fs::File,
}

impl Read for LocalFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for LocalFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Seek for LocalFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl OpenFile for LocalFile {
    fn sync(&mut self) -> VfsResult<()> {
        self.file.sync_all()
            .map_err(|e| VfsError::IoError(format!("sync: {}", e)))
    }

    fn metadata(&self) -> VfsResult<Metadata> {
        let md = self.file.metadata()
            .map_err(|e| VfsError::IoError(format!("metadata: {}", e)))?;
        Ok(LocalFS::convert_metadata(md))
    }

    fn set_len(&mut self, size: u64) -> VfsResult<()> {
        self.file.set_len(size)
            .map_err(|e| VfsError::IoError(format!("set_len: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_local_fs_basic() {
        let temp = TempDir::new().unwrap();
        let fs = LocalFS::new(temp.path());

        // Write and read
        fs.write(Path::new("test.txt"), b"hello").unwrap();
        let data = fs.read(Path::new("test.txt")).unwrap();
        assert_eq!(data, b"hello");

        // Exists
        assert!(fs.exists(Path::new("test.txt")));
        assert!(!fs.exists(Path::new("missing.txt")));

        // Delete
        fs.delete(Path::new("test.txt")).unwrap();
        assert!(!fs.exists(Path::new("test.txt")));
    }

    #[test]
    fn test_local_fs_directories() {
        let temp = TempDir::new().unwrap();
        let fs = LocalFS::new(temp.path());

        // Create directory
        fs.create_dir(Path::new("testdir")).unwrap();
        assert!(fs.exists(Path::new("testdir")));

        // List empty directory
        let entries = fs.list_dir(Path::new("testdir")).unwrap();
        assert_eq!(entries.len(), 0);

        // Create file in directory
        fs.write(Path::new("testdir/file.txt"), b"content").unwrap();
        let entries = fs.list_dir(Path::new("testdir")).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "file.txt");
    }

    #[test]
    fn test_readonly() {
        let temp = TempDir::new().unwrap();
        let fs = LocalFS::new(temp.path());
        fs.write(Path::new("test.txt"), b"hello").unwrap();

        let ro_fs = LocalFS::readonly(temp.path());
        assert!(ro_fs.readonly());

        // Read should work
        let data = ro_fs.read(Path::new("test.txt")).unwrap();
        assert_eq!(data, b"hello");

        // Write should fail
        assert!(matches!(
            ro_fs.write(Path::new("test2.txt"), b"world"),
            Err(VfsError::ReadOnly)
        ));
    }
}
