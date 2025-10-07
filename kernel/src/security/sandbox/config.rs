/*!
 * Sandbox Configuration Logic
 */

use crate::security::types::{Capability, SandboxConfig};
use std::path::{Path, PathBuf};

impl SandboxConfig {
    /// Check if a capability is granted (considering granularity)
    pub fn has_capability(&self, cap: &Capability) -> bool {
        self.capabilities.iter().any(|c| c.grants(cap))
    }

    /// Check if a path is accessible
    pub fn can_access_path(&self, path: &Path) -> bool {
        // Try to canonicalize the path if it exists, otherwise construct from parent
        let check_path = if path.exists() {
            path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
        } else if let Some(parent) = path.parent() {
            // For non-existent paths, canonicalize parent and append file name
            if let (Ok(canonical_parent), Some(file_name)) = (parent.canonicalize(), path.file_name()) {
                canonical_parent.join(file_name)
            } else {
                path.to_path_buf()
            }
        } else {
            path.to_path_buf()
        };

        // First check if explicitly blocked
        for blocked in &self.blocked_paths {
            if check_path.starts_with(blocked) {
                return false;
            }
        }

        // If no allowed paths specified, deny all
        if self.allowed_paths.is_empty() {
            return false;
        }

        // Check if path is within allowed paths
        for allowed in &self.allowed_paths {
            if check_path.starts_with(allowed) {
                return true;
            }
        }

        false
    }

    /// Add a capability
    pub fn grant_capability(&mut self, cap: Capability) {
        self.capabilities.insert(cap);
    }

    /// Remove a capability
    pub fn revoke_capability(&mut self, cap: &Capability) {
        self.capabilities.remove(cap);
    }

    /// Add an allowed path
    pub fn allow_path(&mut self, path: PathBuf) {
        // Canonicalize if possible for consistent matching
        let canonical = path.canonicalize().unwrap_or(path);
        self.allowed_paths.push(canonical);
    }

    /// Add a blocked path
    pub fn block_path(&mut self, path: PathBuf) {
        // Canonicalize if possible for consistent matching
        let canonical = path.canonicalize().unwrap_or(path);
        self.blocked_paths.push(canonical);
    }
}
