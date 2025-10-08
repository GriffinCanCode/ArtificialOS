/*!
 * Metadata Operations Implementation
 * FileSystem trait methods for metadata and file manipulation
 */

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::super::traits::{FileSystem, OpenFile};
use super::super::types::*;
use super::file_handle::MemFile;
use super::node::Node;
use super::MemFS;

impl FileSystem for MemFS {
    fn read(&self, path: &Path) -> VfsResult<Vec<u8>> {
        self.read_impl(path)
    }

    fn write(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        self.write_impl(path, data)
    }

    fn append(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        self.append_impl(path, data)
    }

    fn create(&self, path: &Path) -> VfsResult<()> {
        self.create_impl(path)
    }

    fn delete(&self, path: &Path) -> VfsResult<()> {
        self.delete_impl(path)
    }

    fn exists(&self, path: &Path) -> bool {
        let path = self.normalize(path);
        self.nodes.contains_key(&path)
    }

    fn metadata(&self, path: &Path) -> VfsResult<Metadata> {
        let path = self.normalize(path);

        match self.nodes.get(&path).map(|n| n.clone()) {
            Some(node) => {
                let now = SystemTime::now();
                let size = match &node {
                    Node::File { data, .. } => data.lock().len() as u64,
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
            None => Err(VfsError::NotFound(path.display().to_string().into())),
        }
    }

    fn list_dir(&self, path: &Path) -> VfsResult<Vec<Entry>> {
        self.list_dir_impl(path)
    }

    fn create_dir(&self, path: &Path) -> VfsResult<()> {
        self.create_dir_impl(path)
    }

    fn remove_dir(&self, path: &Path) -> VfsResult<()> {
        self.remove_dir_impl(path)
    }

    fn remove_dir_all(&self, path: &Path) -> VfsResult<()> {
        self.remove_dir_all_impl(path)
    }

    fn copy(&self, from: &Path, to: &Path) -> VfsResult<()> {
        let data = self.read(from)?;
        self.write(to, &data)
    }

    fn rename(&self, from: &Path, to: &Path) -> VfsResult<()> {
        let from = self.normalize(from);
        let to = self.normalize(to);

        let node = self
            .nodes
            .remove(&from)
            .ok_or_else(|| VfsError::NotFound(from.display().to_string().into()))?
            .1;

        // Update parent directories
        if let Some(from_parent) = self.parent_path(&from) {
            let from_name = self.file_name(&from)?;
            self.remove_child(&from_parent, &from_name)?;
        }

        if let Some(to_parent) = self.parent_path(&to) {
            let to_name = self.file_name(&to)?;
            self.nodes.insert(to.clone(), node);
            self.add_child(&to_parent, &to_name, &to)?;
        } else {
            self.nodes.insert(to, node);
        }

        Ok(())
    }

    fn symlink(&self, _src: &Path, _dst: &Path) -> VfsResult<()> {
        Err(VfsError::NotSupported(
            "symlinks not supported in MemFS".to_string().into(),
        ))
    }

    fn read_link(&self, _path: &Path) -> VfsResult<PathBuf> {
        Err(VfsError::NotSupported(
            "symlinks not supported in MemFS".to_string().into(),
        ))
    }

    fn truncate(&self, path: &Path, size: u64) -> VfsResult<()> {
        self.truncate_impl(path, size)
    }

    fn set_permissions(&self, path: &Path, perms: Permissions) -> VfsResult<()> {
        let path = self.normalize(path);

        match self.nodes.get_mut(&path) {
            Some(mut entry) => match entry.value_mut() {
                Node::File { permissions, .. } => {
                    *permissions = perms;
                    Ok(())
                }
                Node::Directory { permissions, .. } => {
                    *permissions = perms;
                    Ok(())
                }
            },
            None => Err(VfsError::NotFound(path.display().to_string().into())),
        }
    }

    fn open(&self, path: &Path, flags: OpenFlags, mode: OpenMode) -> VfsResult<Box<dyn OpenFile>> {
        let path = self.normalize(path);

        // Check if file exists and verify permissions for write operations
        if self.exists(&path) {
            if flags.write || flags.append || flags.truncate {
                // Check file permissions
                let metadata = self.metadata(&path)?;
                if metadata.permissions.is_readonly() {
                    return Err(VfsError::PermissionDenied(
                        format!("file is readonly: {}", path.display()).into(),
                    ));
                }
            }
        }

        // Read initial data
        let data = if self.exists(&path) {
            if flags.truncate {
                Vec::new()
            } else {
                self.read(&path)?
            }
        } else if flags.create || flags.create_new {
            // Create new file with specified permissions
            self.ensure_parent(&path)?;
            let now = SystemTime::now();

            use crate::core::memory::CowMemory;
            use std::sync::Arc;

            self.nodes.insert(
                path.clone(),
                Node::File {
                    data: Arc::new(parking_lot::Mutex::new(CowMemory::new(Vec::new().into()))),
                    permissions: mode.permissions,
                    modified: now,
                    created: now,
                },
            );

            if let Some(parent) = self.parent_path(&path) {
                let file_name = self.file_name(&path)?;
                self.add_child(&parent, &file_name, &path)?;
            }

            Vec::new()
        } else {
            return Err(VfsError::NotFound(path.display().to_string().into()));
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
