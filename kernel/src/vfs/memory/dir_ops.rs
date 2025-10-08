/*!
 * Directory Operations Implementation
 * FileSystem trait methods for directory management
 */

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::super::types::*;
use super::node::Node;
use super::MemFS;

impl MemFS {
    pub(super) fn list_dir_impl(&self, path: &Path) -> VfsResult<Vec<Entry>> {
        let path = self.normalize(path);

        match self.nodes.get(&path).map(|n| n.clone()) {
            Some(Node::Directory { children, .. }) => {
                let mut entries = Vec::new();
                for (name, child_path) in children {
                    if let Some(node) = self.nodes.get(&child_path) {
                        entries.push(Entry::new_unchecked(name.clone(), node.file_type()));
                    }
                }
                Ok(entries)
            }
            Some(Node::File { .. }) => Err(VfsError::NotADirectory(path.display().to_string())),
            None => Err(VfsError::NotFound(path.display().to_string())),
        }
    }

    pub(super) fn create_dir_impl(&self, path: &Path) -> VfsResult<()> {
        let path = self.normalize(path);

        // Check parent directory write permissions
        if let Some(parent_path) = self.parent_path(&path) {
            if let Some(parent_node) = self.nodes.get(&parent_path) {
                if let Node::Directory { permissions, .. } = parent_node.value() {
                    // Check if parent directory is writable (owner write permission)
                    if permissions.mode & 0o200 == 0 {
                        return Err(VfsError::PermissionDenied(format!(
                            "parent directory is readonly: {}",
                            parent_path.display()
                        )));
                    }
                }
            }
        }

        // Create parent directories if needed
        let mut current = PathBuf::from("/");
        for component in path.components().skip(1) {
            current.push(component);

            if !self.nodes.contains_key(&current) {
                // Safe: current always has parent since we start from "/" and push components
                let parent = current
                    .parent()
                    .ok_or_else(|| VfsError::InvalidPath("path has no parent".to_string()))?
                    .to_path_buf();

                // Safe: current always has filename since we just pushed a component
                let name = current
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| {
                        VfsError::InvalidPath("invalid UTF-8 in path component".to_string())
                    })?
                    .to_string();

                self.nodes.insert(
                    current.clone(),
                    Node::Directory {
                        children: HashMap::default(),
                        permissions: Permissions::new(0o755),
                        created: SystemTime::now(),
                    },
                );

                if let Some(mut entry) = self.nodes.get_mut(&parent) {
                    if let Node::Directory { children, .. } = entry.value_mut() {
                        children.insert(name, current.clone());
                    }
                }
            }
        }

        Ok(())
    }

    pub(super) fn remove_dir_impl(&self, path: &Path) -> VfsResult<()> {
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
            Some(Node::Directory { children, .. }) => {
                if !children.is_empty() {
                    return Err(VfsError::InvalidArgument("directory not empty".to_string()));
                }

                self.nodes.remove(&path);

                // Remove from parent
                if let Some(parent) = self.parent_path(&path) {
                    let dir_name = self.file_name(&path)?;
                    self.remove_child(&parent, &dir_name)?;
                }

                Ok(())
            }
            Some(Node::File { .. }) => Err(VfsError::NotADirectory(path.display().to_string())),
            None => Err(VfsError::NotFound(path.display().to_string())),
        }
    }

    pub(super) fn remove_dir_all_impl(&self, path: &Path) -> VfsResult<()> {
        let path = self.normalize(path);

        // Collect all paths to remove
        let mut to_remove = Vec::new();
        let mut to_visit = vec![path.clone()];

        while let Some(current) = to_visit.pop() {
            if let Some(entry) = self.nodes.get(&current) {
                if let Node::Directory { children, .. } = entry.value() {
                    for child_path in children.values() {
                        to_visit.push(child_path.clone());
                    }
                }
            }
            to_remove.push(current);
        }

        // Remove in reverse order (children before parents)
        to_remove.reverse();
        let mut total_size = 0;

        for path_to_remove in to_remove {
            if let Some(entry) = self.nodes.get(&path_to_remove) {
                if let Node::File { data, .. } = entry.value() {
                    total_size += data.len();
                }
            }
            self.nodes.remove(&path_to_remove);
        }

        // Remove from parent
        if let Some(parent) = self.parent_path(&path) {
            let dir_name = self.file_name(&path)?;
            self.remove_child(&parent, &dir_name)?;
        }

        self.update_size_delta(-(total_size as isize));
        Ok(())
    }
}
