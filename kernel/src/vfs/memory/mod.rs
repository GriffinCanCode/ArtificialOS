/*!
 * In-Memory Filesystem Backend
 * Fast, volatile filesystem for testing and temporary storage
 */

mod file_handle;
mod file_ops;
mod dir_ops;
mod metadata_ops;
mod node;

use ahash::RandomState;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use super::types::*;
use node::Node;

/// In-memory filesystem implementation
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic size counter (high-frequency file operations)
#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct MemFS {
    pub(super) nodes: Arc<DashMap<PathBuf, Node, RandomState>>,
    pub(super) max_size: Option<usize>,
    pub(super) current_size: Arc<AtomicUsize>,
}

impl MemFS {
    /// Create new in-memory filesystem
    pub fn new() -> Self {
        let nodes = DashMap::with_hasher(RandomState::new());

        // Create root directory
        nodes.insert(
            PathBuf::from("/"),
            Node::Directory {
                children: HashMap::default(),
                permissions: Permissions::new(0o755),
                created: SystemTime::now(),
            },
        );

        Self {
            nodes: Arc::new(nodes),
            max_size: None,
            current_size: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create with size limit
    pub fn with_capacity(max_size: usize) -> Self {
        let mut fs = Self::new();
        fs.max_size = Some(max_size);
        fs
    }

    /// Normalize path (make absolute and clean)
    pub(super) fn normalize(&self, path: &Path) -> PathBuf {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            Path::new("/").join(path)
        };

        // Use battle-tested path cleaning (handles ., .., multiple /)
        PathBuf::from(path_clean::clean(&path))
    }

    /// Check if space is available and reserve it atomically
    pub(super) fn check_and_reserve_space(&self, additional: usize) -> VfsResult<()> {
        if let Some(max) = self.max_size {
            loop {
                let current = self.current_size.load(Ordering::SeqCst);
                if current + additional > max {
                    return Err(VfsError::OutOfSpace);
                }
                // Try to atomically update
                if self
                    .current_size
                    .compare_exchange(
                        current,
                        current + additional,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    )
                    .is_ok()
                {
                    break;
                }
                // Retry on failure
            }
        }
        Ok(())
    }

    /// Release reserved space (on error)
    pub(super) fn release_space(&self, amount: usize) {
        self.current_size.fetch_sub(amount, Ordering::SeqCst);
    }

    /// Update current size delta
    /// For operations that change size without reservation
    pub(super) fn update_size_delta(&self, delta: isize) {
        if delta > 0 {
            // This path should not be used for new allocations
            // Use check_and_reserve_space instead
            if let Some(max) = self.max_size {
                // Safety check to prevent overflow
                let current = self.current_size.load(Ordering::SeqCst);
                let new_size = current.saturating_add(delta as usize);
                if new_size <= max {
                    self.current_size.store(new_size, Ordering::SeqCst);
                }
            } else {
                self.current_size
                    .fetch_add(delta as usize, Ordering::SeqCst);
            }
        } else {
            self.current_size
                .fetch_sub(delta.unsigned_abs(), Ordering::SeqCst);
        }
    }

    /// Update size after successful operation (for resizing existing files)
    pub(super) fn update_size_atomic(&self, old_size: usize, new_size: usize) {
        if new_size > old_size {
            let delta = new_size - old_size;
            self.current_size.fetch_add(delta, Ordering::SeqCst);
        } else {
            let delta = old_size - new_size;
            self.current_size.fetch_sub(delta, Ordering::SeqCst);
        }
    }

    /// Get parent directory path
    pub(super) fn parent_path(&self, path: &Path) -> Option<PathBuf> {
        path.parent().map(|p| p.to_path_buf())
    }

    /// Get file name from path
    pub(super) fn file_name(&self, path: &Path) -> VfsResult<String> {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| VfsError::InvalidPath(format!("invalid path: {}", path.display())))
    }

    /// Ensure parent directory exists
    pub(super) fn ensure_parent(&self, path: &Path) -> VfsResult<()> {
        if let Some(parent) = self.parent_path(path) {
            if !self.nodes.contains_key(&parent) {
                return Err(VfsError::NotFound(format!(
                    "parent directory not found: {}",
                    parent.display()
                )));
            }

            let node = self.nodes.get(&parent).unwrap();
            if !node.is_dir() {
                return Err(VfsError::NotADirectory(parent.display().to_string()));
            }
        }
        Ok(())
    }

    /// Add child to parent directory
    pub(super) fn add_child(
        &self,
        parent_path: &Path,
        child_name: &str,
        child_path: &PathBuf,
    ) -> VfsResult<()> {
        if let Some(mut node) = self.nodes.get_mut(parent_path) {
            if let Node::Directory { children, .. } = node.value_mut() {
                children.insert(child_name.to_string(), child_path.clone());
                Ok(())
            } else {
                Err(VfsError::NotADirectory(parent_path.display().to_string()))
            }
        } else {
            Err(VfsError::NotADirectory(parent_path.display().to_string()))
        }
    }

    /// Remove child from parent directory
    pub(super) fn remove_child(&self, parent_path: &Path, child_name: &str) -> VfsResult<()> {
        if let Some(mut node) = self.nodes.get_mut(parent_path) {
            if let Node::Directory { children, .. } = node.value_mut() {
                children.remove(child_name);
                Ok(())
            } else {
                Err(VfsError::NotADirectory(parent_path.display().to_string()))
            }
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
