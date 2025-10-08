/*!
 * File Operations Implementation
 * FileSystem trait methods for file I/O
 */

use std::path::Path;
use std::time::SystemTime;

use super::super::types::*;
use super::node::Node;
use super::MemFS;
use crate::core::{simd_memcpy, PooledBuffer};

impl MemFS {
    pub(super) fn read_impl(&self, path: &Path) -> VfsResult<Vec<u8>> {
        use crate::core::memory::arena::with_arena;

        with_arena(|arena| {
            let path = self.normalize(path);

            match self.nodes.get(&path).map(|n| n.clone()) {
                Some(Node::File { data, .. }) => {
                    let cow_guard = data.lock();
                    let content = cow_guard.read(|buf| {
                        let mut vec = bumpalo::collections::Vec::with_capacity_in(buf.len(), arena);
                        vec.resize(buf.len(), 0);
                        simd_memcpy(&mut vec, buf);
                        vec.into_iter().collect()
                    });
                    Ok(content)
                }
                Some(Node::Directory { .. }) => {
                    Err(VfsError::IsADirectory(format!("{}", path.display()).into()))
                }
                None => Err(VfsError::NotFound(format!("{}", path.display()).into())),
            }
        })
    }

    pub(super) fn write_impl(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        let path = self.normalize(path);
        self.ensure_parent(&path)?;

        // Check if file exists and is readonly
        let file_exists = if let Some(node) = self.nodes.get(&path) {
            if let Node::File { permissions, .. } = node.value() {
                if permissions.is_readonly() {
                    return Err(VfsError::PermissionDenied(
                        format!("file is readonly: {}", path.display()).into(),
                    ));
                }
            }
            true
        } else {
            false
        };

        // If creating a new file, check parent directory write permissions
        if !file_exists {
            if let Some(parent_path) = self.parent_path(&path) {
                if let Some(parent_node) = self.nodes.get(&parent_path) {
                    if let Node::Directory { permissions, .. } = parent_node.value() {
                        if permissions.mode & 0o200 == 0 {
                            return Err(VfsError::PermissionDenied(
                                format!("parent directory is readonly: {}", parent_path.display())
                                    .into(),
                            ));
                        }
                    }
                }
            }
        }

        let space_needed = if let Some(node) = self.nodes.get(&path) {
            if let Node::File { data: old_data, .. } = node.value() {
                let old_len = old_data.lock().len();
                if data.len() > old_len {
                    data.len() - old_len
                } else {
                    0
                }
            } else {
                data.len()
            }
        } else {
            data.len()
        };

        self.check_and_reserve_space(space_needed)?;

        let now = SystemTime::now();

        let old_size = if let Some(node) = self.nodes.get(&path) {
            if let Node::File { data: old_data, .. } = node.value() {
                old_data.lock().len()
            } else {
                0
            }
        } else {
            0
        };

        // Add child to parent if new file
        if !self.nodes.contains_key(&path) {
            if let Some(parent) = self.parent_path(&path) {
                let file_name = self.file_name(&path)?;
                let result = self.add_child(&parent, &file_name, &path);
                if result.is_err() {
                    // Release reserved space on error
                    self.release_space(space_needed);
                    return result;
                }
            }
        }

        let mut file_data = PooledBuffer::get(data.len());
        file_data.resize(data.len(), 0);
        simd_memcpy(&mut file_data, data);

        use crate::core::memory::CowMemory;
        use std::sync::Arc;

        self.nodes.insert(
            path,
            Node::File {
                data: Arc::new(parking_lot::Mutex::new(
                    CowMemory::new(file_data.into_vec()).into(),
                )),
                permissions: Permissions::readwrite(),
                modified: now,
                created: now,
            },
        );

        // Adjust size if we reserved too much or too little
        // This handles the case where the actual size change differs from reservation
        if old_size > 0 {
            self.update_size_atomic(old_size, data.len());
        }

        Ok(())
    }

    pub(super) fn append_impl(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        let path = self.normalize(path);

        // Check if file exists and is readonly
        if let Some(node) = self.nodes.get(&path) {
            if let Node::File { permissions, .. } = node.value() {
                if permissions.is_readonly() {
                    return Err(VfsError::PermissionDenied(
                        format!("file is readonly: {}", path.display()).into(),
                    ));
                }
            }
        }

        // Reserve space atomically
        self.check_and_reserve_space(data.len())?;

        match self.nodes.get(&path) {
            Some(entry) => match entry.value() {
                Node::File { data: cow_data, .. } => {
                    let mut cow_guard = cow_data.lock();
                    cow_guard.write(|buf| {
                        buf.extend_from_slice(data);
                    });
                    drop(cow_guard);

                    if let Some(mut entry_mut) = self.nodes.get_mut(&path) {
                        if let Node::File { modified, .. } = entry_mut.value_mut() {
                            *modified = SystemTime::now();
                        }
                    }
                    Ok(())
                }
                Node::Directory { .. } => {
                    self.release_space(data.len());
                    Err(VfsError::IsADirectory(path.display().to_string().into()))
                }
            },
            None => {
                self.release_space(data.len());
                self.write_impl(&path, data)
            }
        }
    }

    pub(super) fn create_impl(&self, path: &Path) -> VfsResult<()> {
        self.write_impl(path, &[])
    }

    pub(super) fn delete_impl(&self, path: &Path) -> VfsResult<()> {
        let path = self.normalize(path);

        // Check parent directory write permissions
        if let Some(parent_path) = self.parent_path(&path) {
            if let Some(parent_node) = self.nodes.get(&parent_path) {
                if let Node::Directory { permissions, .. } = parent_node.value() {
                    if permissions.mode & 0o200 == 0 {
                        return Err(VfsError::PermissionDenied(
                            format!("parent directory is readonly: {}", parent_path.display())
                                .into(),
                        ));
                    }
                }
            }
        }

        match self.nodes.get(&path).map(|n| n.clone()) {
            Some(Node::File { data, .. }) => {
                let size = data.lock().len();
                self.nodes.remove(&path);

                if let Some(parent) = self.parent_path(&path) {
                    let file_name = self.file_name(&path)?;
                    self.remove_child(&parent, &file_name)?;
                    self.update_size_delta(-(size as isize));
                    Ok(())
                } else {
                    self.update_size_delta(-(size as isize));
                    Ok(())
                }
            }
            Some(Node::Directory { .. }) => {
                Err(VfsError::IsADirectory(path.display().to_string().into()))
            }
            None => Err(VfsError::NotFound(path.display().to_string().into())),
        }
    }

    pub(super) fn truncate_impl(&self, path: &Path, size: u64) -> VfsResult<()> {
        let path = self.normalize(path);
        let new_size = size as usize;

        let old_size = match self.nodes.get(&path).map(|n| n.clone()) {
            Some(Node::Directory { .. }) => {
                return Err(VfsError::IsADirectory(path.display().to_string().into()))
            }
            None => return Err(VfsError::NotFound(path.display().to_string().into())),
            Some(Node::File {
                data, permissions, ..
            }) => {
                if permissions.is_readonly() {
                    return Err(VfsError::PermissionDenied(
                        format!("file is readonly: {}", path.display()).into(),
                    ));
                }
                data.lock().len()
            }
        };

        if new_size > old_size {
            let additional = new_size - old_size;
            self.check_and_reserve_space(additional)?;
        }

        if let Some(entry) = self.nodes.get(&path) {
            if let Node::File { data, .. } = entry.value() {
                let mut cow_guard = data.lock();
                cow_guard.write(|buf| {
                    buf.resize(new_size, 0);
                });
                drop(cow_guard);

                if let Some(mut entry_mut) = self.nodes.get_mut(&path) {
                    if let Node::File { modified, .. } = entry_mut.value_mut() {
                        *modified = SystemTime::now();
                    }
                }

                self.update_size_atomic(old_size, new_size);
                Ok(())
            } else {
                if new_size > old_size {
                    self.release_space(new_size - old_size);
                }
                Err(VfsError::NotFound(path.display().to_string().into()))
            }
        } else {
            if new_size > old_size {
                self.release_space(new_size - old_size);
            }
            Err(VfsError::NotFound(path.display().to_string().into()))
        }
    }
}
