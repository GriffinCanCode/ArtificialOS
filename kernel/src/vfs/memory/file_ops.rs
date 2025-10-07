/*!
 * File Operations Implementation
 * FileSystem trait methods for file I/O
 */

use std::path::Path;
use std::time::SystemTime;

use super::super::types::*;
use super::node::Node;
use super::MemFS;
use crate::memory::simd_memcpy;

impl MemFS {
    pub(super) fn read_impl(&self, path: &Path) -> VfsResult<Vec<u8>> {
        let path = self.normalize(path);

        match self.nodes.get(&path).map(|n| n.clone()) {
            Some(Node::File { data, .. }) => {
                // Use SIMD-accelerated copy for large files
                let mut result = vec![0u8; data.len()];
                simd_memcpy(&mut result, &data);
                Ok(result)
            }
            Some(Node::Directory { .. }) => Err(VfsError::IsADirectory(path.display().to_string())),
            None => Err(VfsError::NotFound(path.display().to_string())),
        }
    }

    pub(super) fn write_impl(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        let path = self.normalize(path);
        self.ensure_parent(&path)?;

        // Check if file exists and is readonly
        let file_exists = if let Some(node) = self.nodes.get(&path) {
            if let Node::File { permissions, .. } = node.value() {
                if permissions.is_readonly() {
                    return Err(VfsError::PermissionDenied(format!(
                        "file is readonly: {}",
                        path.display()
                    )));
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
                            return Err(VfsError::PermissionDenied(format!(
                                "parent directory is readonly: {}",
                                parent_path.display()
                            )));
                        }
                    }
                }
            }
        }

        // Calculate space needed
        let space_needed = if let Some(node) = self.nodes.get(&path) {
            // Replacing existing file - only need additional space
            if let Node::File { data: old_data, .. } = node.value() {
                if data.len() > old_data.len() {
                    data.len() - old_data.len()
                } else {
                    0
                }
            } else {
                data.len()
            }
        } else {
            // New file - need full space
            data.len()
        };

        // Check and reserve space atomically
        self.check_and_reserve_space(space_needed)?;

        let now = SystemTime::now();

        // Get old size for accurate tracking
        let old_size = if let Some(node) = self.nodes.get(&path) {
            if let Node::File { data: old_data, .. } = node.value() {
                old_data.len()
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

        // Use SIMD-accelerated copy for file data
        let mut file_data = vec![0u8; data.len()];
        simd_memcpy(&mut file_data, data);

        self.nodes.insert(
            path,
            Node::File {
                data: file_data,
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
                    return Err(VfsError::PermissionDenied(format!(
                        "file is readonly: {}",
                        path.display()
                    )));
                }
            }
        }

        // Reserve space atomically
        self.check_and_reserve_space(data.len())?;

        match self.nodes.get_mut(&path) {
            Some(mut entry) => {
                match entry.value_mut() {
                    Node::File {
                        data: file_data,
                        modified,
                        ..
                    } => {
                        file_data.extend_from_slice(data);
                        *modified = SystemTime::now();
                        Ok(())
                    }
                    Node::Directory { .. } => {
                        // Release reserved space on error
                        self.release_space(data.len());
                        Err(VfsError::IsADirectory(path.display().to_string()))
                    }
                }
            }
            None => {
                // Release our reservation and let write handle it
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
                        return Err(VfsError::PermissionDenied(format!(
                            "parent directory is readonly: {}",
                            parent_path.display()
                        )));
                    }
                }
            }
        }

        match self.nodes.get(&path).map(|n| n.clone()) {
            Some(Node::File { data, .. }) => {
                let size = data.len();
                self.nodes.remove(&path);

                // Remove from parent
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
            Some(Node::Directory { .. }) => Err(VfsError::IsADirectory(path.display().to_string())),
            None => Err(VfsError::NotFound(path.display().to_string())),
        }
    }

    pub(super) fn truncate_impl(&self, path: &Path, size: u64) -> VfsResult<()> {
        let path = self.normalize(path);
        let new_size = size as usize;

        // Check node type, get current size, and check permissions
        let old_size = match self.nodes.get(&path).map(|n| n.clone()) {
            Some(Node::Directory { .. }) => {
                return Err(VfsError::IsADirectory(path.display().to_string()))
            }
            None => return Err(VfsError::NotFound(path.display().to_string())),
            Some(Node::File { data, permissions, .. }) => {
                // Check if file is readonly
                if permissions.is_readonly() {
                    return Err(VfsError::PermissionDenied(format!(
                        "file is readonly: {}",
                        path.display()
                    )));
                }
                data.len()
            }
        };

        // Reserve space if growing
        if new_size > old_size {
            let additional = new_size - old_size;
            self.check_and_reserve_space(additional)?;
        }

        // Perform the truncate
        if let Some(mut entry) = self.nodes.get_mut(&path) {
            if let Node::File { data, modified, .. } = entry.value_mut() {
                data.resize(new_size, 0);
                *modified = SystemTime::now();

                // Update size tracking
                drop(entry);
                self.update_size_atomic(old_size, new_size);
                Ok(())
            } else {
                if new_size > old_size {
                    self.release_space(new_size - old_size);
                }
                Err(VfsError::NotFound(path.display().to_string()))
            }
        } else {
            // File was removed between checks
            if new_size > old_size {
                // Release reserved space
                self.release_space(new_size - old_size);
            }
            Err(VfsError::NotFound(path.display().to_string()))
        }
    }
}
