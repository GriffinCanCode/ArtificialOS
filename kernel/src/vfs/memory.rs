/*!
 * In-Memory Filesystem Backend
 * Fast, volatile filesystem for testing and temporary storage
 */

use parking_lot::RwLock;
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use super::traits::{FileSystem, OpenFile};
use super::types::*;

/// In-memory filesystem node
#[derive(Debug, Clone)]
enum Node {
    File {
        data: Vec<u8>,
        permissions: Permissions,
        modified: SystemTime,
        created: SystemTime,
    },
    Directory {
        children: HashMap<String, PathBuf>,
        permissions: Permissions,
        created: SystemTime,
    },
}

impl Node {
    fn is_file(&self) -> bool {
        matches!(self, Node::File { .. })
    }

    fn is_dir(&self) -> bool {
        matches!(self, Node::Directory { .. })
    }

    fn file_type(&self) -> FileType {
        match self {
            Node::File { .. } => FileType::File,
            Node::Directory { .. } => FileType::Directory,
        }
    }

    fn permissions(&self) -> Permissions {
        match self {
            Node::File { permissions, .. } => *permissions,
            Node::Directory { permissions, .. } => *permissions,
        }
    }

    fn created(&self) -> SystemTime {
        match self {
            Node::File { created, .. } => *created,
            Node::Directory { created, .. } => *created,
        }
    }
}

/// In-memory filesystem implementation
#[derive(Debug, Clone)]
pub struct MemFS {
    nodes: Arc<RwLock<HashMap<PathBuf, Node>>>,
    max_size: Option<usize>,
    current_size: Arc<RwLock<usize>>,
}

impl MemFS {
    /// Create new in-memory filesystem
    pub fn new() -> Self {
        let mut nodes = HashMap::new();

        // Create root directory
        nodes.insert(
            PathBuf::from("/"),
            Node::Directory {
                children: HashMap::new(),
                permissions: Permissions::new(0o755),
                created: SystemTime::now(),
            },
        );

        Self {
            nodes: Arc::new(RwLock::new(nodes)),
            max_size: None,
            current_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with size limit
    pub fn with_capacity(max_size: usize) -> Self {
        let mut fs = Self::new();
        fs.max_size = Some(max_size);
        fs
    }

    /// Normalize path (make absolute and clean)
    fn normalize(&self, path: &Path) -> PathBuf {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            Path::new("/").join(path)
        };

        // Simplify path by removing . and ..
        let mut components = Vec::new();
        for comp in path.components() {
            match comp {
                std::path::Component::Normal(c) => components.push(c),
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::RootDir => components.clear(),
                _ => {}
            }
        }

        let mut result = PathBuf::from("/");
        for comp in components {
            result.push(comp);
        }
        result
    }

    /// Check if space is available
    fn check_space(&self, additional: usize) -> VfsResult<()> {
        if let Some(max) = self.max_size {
            let current = *self.current_size.read();
            if current + additional > max {
                return Err(VfsError::OutOfSpace);
            }
        }
        Ok(())
    }

    /// Update current size
    fn update_size(&self, delta: isize) {
        let mut size = self.current_size.write();
        if delta > 0 {
            *size += delta as usize;
        } else {
            *size = size.saturating_sub(delta.unsigned_abs());
        }
    }

    /// Get parent directory path
    fn parent_path(&self, path: &Path) -> Option<PathBuf> {
        path.parent().map(|p| p.to_path_buf())
    }

    /// Get file name from path
    fn file_name(&self, path: &Path) -> VfsResult<String> {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| VfsError::InvalidPath(format!("invalid path: {}", path.display())))
    }

    /// Ensure parent directory exists
    fn ensure_parent(&self, path: &Path) -> VfsResult<()> {
        if let Some(parent) = self.parent_path(path) {
            let nodes = self.nodes.read();
            if !nodes.contains_key(&parent) {
                drop(nodes);
                return Err(VfsError::NotFound(format!("parent directory not found: {}", parent.display())));
            }

            let node = nodes.get(&parent).unwrap();
            if !node.is_dir() {
                return Err(VfsError::NotADirectory(parent.display().to_string()));
            }
        }
        Ok(())
    }

    /// Add child to parent directory
    fn add_child(&self, parent_path: &Path, child_name: &str, child_path: &PathBuf) -> VfsResult<()> {
        let mut nodes = self.nodes.write();

        if let Some(Node::Directory { children, .. }) = nodes.get_mut(parent_path) {
            children.insert(child_name.to_string(), child_path.clone());
            Ok(())
        } else {
            Err(VfsError::NotADirectory(parent_path.display().to_string()))
        }
    }

    /// Remove child from parent directory
    fn remove_child(&self, parent_path: &Path, child_name: &str) -> VfsResult<()> {
        let mut nodes = self.nodes.write();

        if let Some(Node::Directory { children, .. }) = nodes.get_mut(parent_path) {
            children.remove(child_name);
            Ok(())
        } else {
            Err(VfsError::NotADirectory(parent_path.display().to_string()))
        }
    }
}

impl Default for MemFS {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for MemFS {
    fn read(&self, path: &Path) -> VfsResult<Vec<u8>> {
        let path = self.normalize(path);
        let nodes = self.nodes.read();

        match nodes.get(&path) {
            Some(Node::File { data, .. }) => Ok(data.clone()),
            Some(Node::Directory { .. }) => Err(VfsError::IsADirectory(path.display().to_string())),
            None => Err(VfsError::NotFound(path.display().to_string())),
        }
    }

    fn write(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        let path = self.normalize(path);
        self.check_space(data.len())?;
        self.ensure_parent(&path)?;

        let mut nodes = self.nodes.write();
        let now = SystemTime::now();

        // Calculate size change
        let size_delta = if let Some(Node::File { data: old_data, .. }) = nodes.get(&path) {
            data.len() as isize - old_data.len() as isize
        } else {
            data.len() as isize
        };

        // Add child to parent if new file
        if !nodes.contains_key(&path) {
            if let Some(parent) = self.parent_path(&path) {
                let file_name = self.file_name(&path)?;
                drop(nodes);
                self.add_child(&parent, &file_name, &path)?;
                nodes = self.nodes.write();
            }
        }

        nodes.insert(
            path,
            Node::File {
                data: data.to_vec(),
                permissions: Permissions::readwrite(),
                modified: now,
                created: now,
            },
        );

        drop(nodes);
        self.update_size(size_delta);
        Ok(())
    }

    fn append(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        let path = self.normalize(path);
        self.check_space(data.len())?;

        let mut nodes = self.nodes.write();

        match nodes.get_mut(&path) {
            Some(Node::File { data: file_data, modified, .. }) => {
                file_data.extend_from_slice(data);
                *modified = SystemTime::now();
                drop(nodes);
                self.update_size(data.len() as isize);
                Ok(())
            }
            Some(Node::Directory { .. }) => Err(VfsError::IsADirectory(path.display().to_string())),
            None => {
                drop(nodes);
                self.write(&path, data)
            }
        }
    }

    fn create(&self, path: &Path) -> VfsResult<()> {
        self.write(path, &[])
    }

    fn delete(&self, path: &Path) -> VfsResult<()> {
        let path = self.normalize(path);
        let mut nodes = self.nodes.write();

        match nodes.get(&path) {
            Some(Node::File { data, .. }) => {
                let size = data.len();
                nodes.remove(&path);

                // Remove from parent
                if let Some(parent) = self.parent_path(&path) {
                    let file_name = self.file_name(&path)?;
                    drop(nodes);
                    self.remove_child(&parent, &file_name)?;
                    self.update_size(-(size as isize));
                    Ok(())
                } else {
                    drop(nodes);
                    self.update_size(-(size as isize));
                    Ok(())
                }
            }
            Some(Node::Directory { .. }) => Err(VfsError::IsADirectory(path.display().to_string())),
            None => Err(VfsError::NotFound(path.display().to_string())),
        }
    }

    fn exists(&self, path: &Path) -> bool {
        let path = self.normalize(path);
        self.nodes.read().contains_key(&path)
    }

    fn metadata(&self, path: &Path) -> VfsResult<Metadata> {
        let path = self.normalize(path);
        let nodes = self.nodes.read();

        match nodes.get(&path) {
            Some(node) => {
                let now = SystemTime::now();
                let size = match node {
                    Node::File { data, .. } => data.len() as u64,
                    Node::Directory { .. } => 0,
                };

                Ok(Metadata {
                    file_type: node.file_type(),
                    size,
                    permissions: node.permissions(),
                    modified: now,
                    accessed: now,
                    created: node.created(),
                })
            }
            None => Err(VfsError::NotFound(path.display().to_string())),
        }
    }

    fn list_dir(&self, path: &Path) -> VfsResult<Vec<Entry>> {
        let path = self.normalize(path);
        let nodes = self.nodes.read();

        match nodes.get(&path) {
            Some(Node::Directory { children, .. }) => {
                let mut entries = Vec::new();
                for (name, child_path) in children {
                    if let Some(node) = nodes.get(child_path) {
                        entries.push(Entry::new(name.clone(), node.file_type()));
                    }
                }
                Ok(entries)
            }
            Some(Node::File { .. }) => Err(VfsError::NotADirectory(path.display().to_string())),
            None => Err(VfsError::NotFound(path.display().to_string())),
        }
    }

    fn create_dir(&self, path: &Path) -> VfsResult<()> {
        let path = self.normalize(path);
        let mut nodes = self.nodes.write();

        // Create parent directories if needed
        let mut current = PathBuf::from("/");
        for component in path.components().skip(1) {
            current.push(component);

            if !nodes.contains_key(&current) {
                let parent = current.parent().unwrap().to_path_buf();
                let name = current.file_name().unwrap().to_str().unwrap().to_string();

                nodes.insert(
                    current.clone(),
                    Node::Directory {
                        children: HashMap::new(),
                        permissions: Permissions::new(0o755),
                        created: SystemTime::now(),
                    },
                );

                if let Some(Node::Directory { children, .. }) = nodes.get_mut(&parent) {
                    children.insert(name, current.clone());
                }
            }
        }

        Ok(())
    }

    fn remove_dir(&self, path: &Path) -> VfsResult<()> {
        let path = self.normalize(path);
        let mut nodes = self.nodes.write();

        match nodes.get(&path) {
            Some(Node::Directory { children, .. }) => {
                if !children.is_empty() {
                    return Err(VfsError::InvalidArgument("directory not empty".to_string()));
                }

                nodes.remove(&path);

                // Remove from parent
                if let Some(parent) = self.parent_path(&path) {
                    let dir_name = self.file_name(&path)?;
                    drop(nodes);
                    self.remove_child(&parent, &dir_name)?;
                }

                Ok(())
            }
            Some(Node::File { .. }) => Err(VfsError::NotADirectory(path.display().to_string())),
            None => Err(VfsError::NotFound(path.display().to_string())),
        }
    }

    fn remove_dir_all(&self, path: &Path) -> VfsResult<()> {
        let path = self.normalize(path);
        let nodes = self.nodes.read();

        // Collect all paths to remove
        let mut to_remove = Vec::new();
        let mut to_visit = vec![path.clone()];

        while let Some(current) = to_visit.pop() {
            if let Some(Node::Directory { children, .. }) = nodes.get(&current) {
                for child_path in children.values() {
                    to_visit.push(child_path.clone());
                }
            }
            to_remove.push(current);
        }

        drop(nodes);

        // Remove in reverse order (children before parents)
        to_remove.reverse();
        let mut total_size = 0;

        for path_to_remove in to_remove {
            let mut nodes = self.nodes.write();
            if let Some(Node::File { data, .. }) = nodes.get(&path_to_remove) {
                total_size += data.len();
            }
            nodes.remove(&path_to_remove);
        }

        // Remove from parent
        if let Some(parent) = self.parent_path(&path) {
            let dir_name = self.file_name(&path)?;
            self.remove_child(&parent, &dir_name)?;
        }

        self.update_size(-(total_size as isize));
        Ok(())
    }

    fn copy(&self, from: &Path, to: &Path) -> VfsResult<()> {
        let data = self.read(from)?;
        self.write(to, &data)
    }

    fn rename(&self, from: &Path, to: &Path) -> VfsResult<()> {
        let from = self.normalize(from);
        let to = self.normalize(to);

        let mut nodes = self.nodes.write();
        let node = nodes.remove(&from)
            .ok_or_else(|| VfsError::NotFound(from.display().to_string()))?;

        // Update parent directories
        if let Some(from_parent) = self.parent_path(&from) {
            let from_name = self.file_name(&from)?;
            drop(nodes);
            self.remove_child(&from_parent, &from_name)?;
            nodes = self.nodes.write();
        }

        if let Some(to_parent) = self.parent_path(&to) {
            let to_name = self.file_name(&to)?;
            nodes.insert(to.clone(), node);
            drop(nodes);
            self.add_child(&to_parent, &to_name, &to)?;
        } else {
            nodes.insert(to, node);
        }

        Ok(())
    }

    fn symlink(&self, _src: &Path, _dst: &Path) -> VfsResult<()> {
        Err(VfsError::NotSupported("symlinks not supported in MemFS".to_string()))
    }

    fn read_link(&self, _path: &Path) -> VfsResult<PathBuf> {
        Err(VfsError::NotSupported("symlinks not supported in MemFS".to_string()))
    }

    fn truncate(&self, path: &Path, size: u64) -> VfsResult<()> {
        let path = self.normalize(path);

        // Check node type first
        {
            let nodes = self.nodes.read();
            match nodes.get(&path) {
                Some(Node::Directory { .. }) => return Err(VfsError::IsADirectory(path.display().to_string())),
                None => return Err(VfsError::NotFound(path.display().to_string())),
                Some(Node::File { .. }) => {}, // Continue
            }
        }

        let mut nodes = self.nodes.write();
        if let Some(Node::File { data, modified, .. }) = nodes.get_mut(&path) {
            let old_size = data.len();
            let new_size = size as usize;

            if new_size < old_size {
                data.truncate(new_size);
                *modified = SystemTime::now();
                drop(nodes);
                self.update_size(-((old_size - new_size) as isize));
            } else if new_size > old_size {
                let additional = new_size - old_size;
                drop(nodes);
                self.check_space(additional)?;
                let mut nodes = self.nodes.write();
                if let Some(Node::File { data, modified, .. }) = nodes.get_mut(&path) {
                    data.resize(new_size, 0);
                    *modified = SystemTime::now();
                    self.update_size(additional as isize);
                }
            } else {
                // Size unchanged, just update modified time
                *modified = SystemTime::now();
            }

            Ok(())
        } else {
            Err(VfsError::NotFound(path.display().to_string()))
        }
    }

    fn set_permissions(&self, path: &Path, perms: Permissions) -> VfsResult<()> {
        let path = self.normalize(path);
        let mut nodes = self.nodes.write();

        match nodes.get_mut(&path) {
            Some(Node::File { permissions, .. }) => {
                *permissions = perms;
                Ok(())
            }
            Some(Node::Directory { permissions, .. }) => {
                *permissions = perms;
                Ok(())
            }
            None => Err(VfsError::NotFound(path.display().to_string())),
        }
    }

    fn open(&self, path: &Path, flags: OpenFlags, _mode: OpenMode) -> VfsResult<Box<dyn OpenFile>> {
        let path = self.normalize(path);

        // Read initial data
        let data = if self.exists(&path) {
            if flags.truncate {
                Vec::new()
            } else {
                self.read(&path)?
            }
        } else if flags.create || flags.create_new {
            Vec::new()
        } else {
            return Err(VfsError::NotFound(path.display().to_string()));
        };

        Ok(Box::new(MemFile {
            fs: self.clone(),
            path: path.clone(),
            cursor: Cursor::new(data),
            flags,
        }))
    }

    fn name(&self) -> &str {
        "memory"
    }

    fn readonly(&self) -> bool {
        false
    }
}

/// In-memory file handle
struct MemFile {
    fs: MemFS,
    path: PathBuf,
    cursor: Cursor<Vec<u8>>,
    flags: OpenFlags,
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
            let data = self.cursor.get_ref().clone();
            self.fs.write(&self.path, &data)?;
        }
        Ok(())
    }

    fn metadata(&self) -> VfsResult<Metadata> {
        self.fs.metadata(&self.path)
    }

    fn set_len(&mut self, size: u64) -> VfsResult<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memfs_basic() {
        let fs = MemFS::new();

        // Write and read
        fs.write(Path::new("/test.txt"), b"hello").unwrap();
        let data = fs.read(Path::new("/test.txt")).unwrap();
        assert_eq!(data, b"hello");

        // Exists
        assert!(fs.exists(Path::new("/test.txt")));
        assert!(!fs.exists(Path::new("/missing.txt")));

        // Delete
        fs.delete(Path::new("/test.txt")).unwrap();
        assert!(!fs.exists(Path::new("/test.txt")));
    }

    #[test]
    fn test_memfs_directories() {
        let fs = MemFS::new();

        // Create directory
        fs.create_dir(Path::new("/testdir")).unwrap();
        assert!(fs.exists(Path::new("/testdir")));

        // Create nested
        fs.create_dir(Path::new("/testdir/nested")).unwrap();
        assert!(fs.exists(Path::new("/testdir/nested")));

        // List
        fs.write(Path::new("/testdir/file.txt"), b"content").unwrap();
        let entries = fs.list_dir(Path::new("/testdir")).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_capacity_limit() {
        let fs = MemFS::with_capacity(10);

        // Should succeed
        fs.write(Path::new("/small.txt"), b"hello").unwrap();

        // Should fail - exceeds capacity
        assert!(matches!(
            fs.write(Path::new("/large.txt"), b"hello world"),
            Err(VfsError::OutOfSpace)
        ));
    }

    #[test]
    fn test_path_normalization() {
        let fs = MemFS::new();

        fs.write(Path::new("/test.txt"), b"hello").unwrap();

        // Different path representations should work
        assert!(fs.exists(Path::new("test.txt")));
        assert!(fs.exists(Path::new("/test.txt")));
        assert!(fs.exists(Path::new("//test.txt")));
    }
}
