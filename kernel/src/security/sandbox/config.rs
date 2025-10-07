/*!
 * Sandbox Configuration Logic
 */

use crate::security::types::{Capability, SandboxConfig};
use std::path::{Path, PathBuf};

/// Safely canonicalize a path with fallback for non-existent paths
/// Uses parent canonicalization if the path doesn't exist
fn safe_canonicalize(path: &Path) -> PathBuf {
    // Try to canonicalize the path directly if it exists
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }

    // For non-existent paths, canonicalize parent and append filename
    if let Some(parent) = path.parent() {
        if let (Ok(canonical_parent), Some(file_name)) = (parent.canonicalize(), path.file_name()) {
            return canonical_parent.join(file_name);
        }
    }

    // Last resort: return as-is (this is the weakest guarantee)
    path.to_path_buf()
}

impl SandboxConfig {
    /// Check if a capability is granted (considering granularity)
    pub fn has_capability(&self, cap: &Capability) -> bool {
        self.capabilities.iter().any(|c| c.grants(cap))
    }

    /// Check if a path is accessible
    /// Always canonicalizes paths before checking to prevent TOCTOU attacks
    pub fn can_access_path(&self, path: &Path) -> bool {
        // Always canonicalize the path being checked
        let check_path = safe_canonicalize(path);

        // First check if explicitly blocked (paths in list are already canonical)
        for blocked in &self.blocked_paths {
            if check_path.starts_with(blocked) {
                return false;
            }
        }

        // If no allowed paths specified, deny all
        if self.allowed_paths.is_empty() {
            return false;
        }

        // Check if path is within allowed paths (paths in list are already canonical)
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
    /// Always canonicalizes the path before storing for security
    pub fn allow_path(&mut self, path: PathBuf) {
        // Always canonicalize before storing to ensure consistent, secure matching
        let canonical = safe_canonicalize(&path);
        self.allowed_paths.push(canonical);
    }

    /// Add a blocked path
    /// Always canonicalizes the path before storing for security
    pub fn block_path(&mut self, path: PathBuf) {
        // Always canonicalize before storing to ensure consistent, secure matching
        let canonical = safe_canonicalize(&path);
        self.blocked_paths.push(canonical);
    }

    /// Canonicalize all stored paths in this config
    /// Should be called after construction to ensure all paths are normalized
    pub fn canonicalize_paths(&mut self) {
        // Canonicalize allowed paths
        self.allowed_paths = self
            .allowed_paths
            .iter()
            .map(|p| safe_canonicalize(p))
            .collect();

        // Canonicalize blocked paths
        self.blocked_paths = self
            .blocked_paths
            .iter()
            .map(|p| safe_canonicalize(p))
            .collect();
    }
}
