/*!
 * Sandbox Module
 * Provides secure, isolated execution environment for processes
 */

use super::traits::*;
use super::types::*;
use crate::core::types::{Pid, ResourceLimits};
use log::{info, warn};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl SandboxConfig {
    /// Check if a capability is granted
    pub fn has_capability(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Check if a path is accessible
    pub fn can_access_path(&self, path: &Path) -> bool {
        // Try to canonicalize the path if it exists, otherwise check parent or use as-is
        let check_path = if path.exists() {
            path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
        } else if let Some(parent) = path.parent() {
            // For non-existent paths, canonicalize the parent directory
            parent
                .canonicalize()
                .unwrap_or_else(|_| path.to_path_buf())
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
        self.allowed_paths.push(path);
    }

    /// Add a blocked path
    pub fn block_path(&mut self, path: PathBuf) {
        self.blocked_paths.push(path);
    }
}

/// Sandbox manager that enforces security policies
#[derive(Clone)]
pub struct SandboxManager {
    sandboxes: Arc<RwLock<HashMap<Pid, SandboxConfig>>>,
    // Track spawned processes per PID for limit enforcement
    spawned_counts: Arc<RwLock<HashMap<Pid, u32>>>,
}

impl SandboxManager {
    pub fn new() -> Self {
        info!("Sandbox manager initialized");
        Self {
            sandboxes: Arc::new(RwLock::new(std::collections::HashMap::new())),
            spawned_counts: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create a new sandbox for a process
    pub fn create_sandbox(&self, config: SandboxConfig) {
        let pid = config.pid;
        self.sandboxes.write().insert(pid, config);
        info!("Created sandbox for PID {}", pid);
    }

    /// Remove a sandbox
    pub fn remove_sandbox(&self, pid: Pid) -> bool {
        if self.sandboxes.write().remove(&pid).is_some() {
            info!("Removed sandbox for PID {}", pid);
            true
        } else {
            false
        }
    }

    /// Check if an operation is allowed
    pub fn check_permission(&self, pid: Pid, cap: &Capability) -> bool {
        let sandboxes = self.sandboxes.read();
        if let Some(sandbox) = sandboxes.get(&pid) {
            let allowed = sandbox.has_capability(cap);
            if !allowed {
                warn!("PID {} denied capability {:?}", pid, cap);
            }
            allowed
        } else {
            warn!("No sandbox found for PID {}", pid);
            false
        }
    }

    /// Check if a path access is allowed
    pub fn check_path_access(&self, pid: Pid, path: &PathBuf) -> bool {
        let sandboxes = self.sandboxes.read();
        if let Some(sandbox) = sandboxes.get(&pid) {
            let allowed = sandbox.can_access_path(path);
            if !allowed {
                warn!("PID {} denied path access: {:?}", pid, path);
            }
            allowed
        } else {
            warn!("No sandbox found for PID {}", pid);
            false
        }
    }

    /// Get resource limits for a process
    pub fn get_limits(&self, pid: Pid) -> Option<ResourceLimits> {
        self.sandboxes
            .read()
            .get(&pid)
            .map(|s| s.resource_limits.clone())
    }

    /// Get sandbox config for a process (clones entire config - use sparingly)
    pub fn get_sandbox(&self, pid: Pid) -> Option<SandboxConfig> {
        self.sandboxes.read().get(&pid).cloned()
    }

    /// Check if sandbox exists without cloning
    pub fn has_sandbox(&self, pid: Pid) -> bool {
        self.sandboxes.read().contains_key(&pid)
    }

    /// Update sandbox config
    pub fn update_sandbox(&self, pid: Pid, config: SandboxConfig) -> bool {
        use std::collections::hash_map::Entry;
        let mut sandboxes = self.sandboxes.write();
        if let Entry::Occupied(mut e) = sandboxes.entry(pid) {
            e.insert(config);
            info!("Updated sandbox for PID {}", pid);
            true
        } else {
            false
        }
    }

    /// Check if process can spawn another process (within limits)
    pub fn can_spawn_process(&self, pid: Pid) -> bool {
        let sandboxes = self.sandboxes.read();
        if let Some(sandbox) = sandboxes.get(&pid) {
            let current_count = *self.spawned_counts.read().get(&pid).unwrap_or(&0);
            current_count < sandbox.resource_limits.max_processes
        } else {
            false
        }
    }

    /// Record a spawned process for a PID
    pub fn record_spawn(&self, pid: Pid) {
        let mut counts = self.spawned_counts.write();
        *counts.entry(pid).or_insert(0) += 1;
    }

    /// Record a terminated spawned process for a PID
    pub fn record_termination(&self, pid: Pid) {
        let mut counts = self.spawned_counts.write();
        if let Some(count) = counts.get_mut(&pid) {
            *count = count.saturating_sub(1);
        }
    }

    /// Get spawned process count for a PID
    pub fn get_spawn_count(&self, pid: Pid) -> u32 {
        *self.spawned_counts.read().get(&pid).unwrap_or(&0)
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

// Trait implementations for SandboxManager

impl SandboxProvider for SandboxManager {
    fn create_sandbox(&self, config: SandboxConfig) {
        let pid = config.pid;
        self.sandboxes.write().insert(pid, config);
        info!("Created sandbox for PID {}", pid);
    }

    fn remove_sandbox(&self, pid: Pid) -> bool {
        if self.sandboxes.write().remove(&pid).is_some() {
            info!("Removed sandbox for PID {}", pid);
            true
        } else {
            false
        }
    }

    fn has_sandbox(&self, pid: Pid) -> bool {
        self.sandboxes.read().contains_key(&pid)
    }

    fn get_sandbox(&self, pid: Pid) -> Option<SandboxConfig> {
        self.sandboxes.read().get(&pid).cloned()
    }

    fn update_sandbox(&self, pid: Pid, config: SandboxConfig) -> bool {
        use std::collections::hash_map::Entry;
        let mut sandboxes = self.sandboxes.write();
        if let Entry::Occupied(mut e) = sandboxes.entry(pid) {
            e.insert(config);
            info!("Updated sandbox for PID {}", pid);
            true
        } else {
            false
        }
    }

    fn stats(&self) -> SandboxStats {
        let sandboxes = self.sandboxes.read();
        SandboxStats {
            total_sandboxes: sandboxes.len(),
            active_processes: sandboxes.len(),
            permission_denials: 0, // Could track this with an atomic counter
            capability_checks: 0,   // Could track this with an atomic counter
        }
    }
}

impl CapabilityChecker for SandboxManager {
    fn check_permission(&self, pid: Pid, cap: &Capability) -> bool {
        let sandboxes = self.sandboxes.read();
        if let Some(sandbox) = sandboxes.get(&pid) {
            let allowed = sandbox.has_capability(cap);
            if !allowed {
                warn!("PID {} denied capability {:?}", pid, cap);
            }
            allowed
        } else {
            warn!("No sandbox found for PID {}", pid);
            false
        }
    }

    fn check_path_access(&self, pid: Pid, path: &PathBuf) -> bool {
        let sandboxes = self.sandboxes.read();
        if let Some(sandbox) = sandboxes.get(&pid) {
            let allowed = sandbox.can_access_path(path);
            if !allowed {
                warn!("PID {} denied path access: {:?}", pid, path);
            }
            allowed
        } else {
            warn!("No sandbox found for PID {}", pid);
            false
        }
    }

    fn check_network_access(&self, pid: Pid) -> bool {
        let sandboxes = self.sandboxes.read();
        if let Some(sandbox) = sandboxes.get(&pid) {
            sandbox.allow_network
        } else {
            false
        }
    }
}

impl CapabilityManager for SandboxManager {
    fn grant_capability(&self, pid: Pid, cap: Capability) -> SecurityResult<()> {
        let mut sandboxes = self.sandboxes.write();
        if let Some(sandbox) = sandboxes.get_mut(&pid) {
            sandbox.grant_capability(cap);
            Ok(())
        } else {
            Err(SecurityError::SandboxNotFound(pid))
        }
    }

    fn revoke_capability(&self, pid: Pid, cap: &Capability) -> SecurityResult<()> {
        let mut sandboxes = self.sandboxes.write();
        if let Some(sandbox) = sandboxes.get_mut(&pid) {
            sandbox.revoke_capability(cap);
            Ok(())
        } else {
            Err(SecurityError::SandboxNotFound(pid))
        }
    }

    fn get_capabilities(&self, pid: Pid) -> Option<Vec<Capability>> {
        self.sandboxes
            .read()
            .get(&pid)
            .map(|s| s.capabilities.iter().cloned().collect())
    }
}

impl ResourceLimitProvider for SandboxManager {
    fn get_limits(&self, pid: Pid) -> Option<ResourceLimits> {
        self.sandboxes
            .read()
            .get(&pid)
            .map(|s| s.resource_limits.clone())
    }

    fn can_spawn_process(&self, pid: Pid) -> bool {
        let sandboxes = self.sandboxes.read();
        if let Some(sandbox) = sandboxes.get(&pid) {
            let current_count = *self.spawned_counts.read().get(&pid).unwrap_or(&0);
            current_count < sandbox.resource_limits.max_processes
        } else {
            false
        }
    }

    fn record_spawn(&self, pid: Pid) {
        let mut counts = self.spawned_counts.write();
        *counts.entry(pid).or_insert(0) += 1;
    }

    fn record_termination(&self, pid: Pid) {
        let mut counts = self.spawned_counts.write();
        if let Some(count) = counts.get_mut(&pid) {
            *count = count.saturating_sub(1);
        }
    }

    fn get_spawn_count(&self, pid: Pid) -> u32 {
        *self.spawned_counts.read().get(&pid).unwrap_or(&0)
    }
}

impl PathAccessControl for SandboxManager {
    fn can_access(&self, pid: Pid, path: &Path) -> bool {
        self.check_path_access(pid, &path.to_path_buf())
    }

    fn allow_path(&self, pid: Pid, path: PathBuf) -> SecurityResult<()> {
        let mut sandboxes = self.sandboxes.write();
        if let Some(sandbox) = sandboxes.get_mut(&pid) {
            sandbox.allow_path(path);
            Ok(())
        } else {
            Err(SecurityError::SandboxNotFound(pid))
        }
    }

    fn block_path(&self, pid: Pid, path: PathBuf) -> SecurityResult<()> {
        let mut sandboxes = self.sandboxes.write();
        if let Some(sandbox) = sandboxes.get_mut(&pid) {
            sandbox.block_path(path);
            Ok(())
        } else {
            Err(SecurityError::SandboxNotFound(pid))
        }
    }

    fn get_allowed_paths(&self, pid: Pid) -> Option<Vec<PathBuf>> {
        self.sandboxes
            .read()
            .get(&pid)
            .map(|s| s.allowed_paths.clone())
    }

    fn get_blocked_paths(&self, pid: Pid) -> Option<Vec<PathBuf>> {
        self.sandboxes
            .read()
            .get(&pid)
            .map(|s| s.blocked_paths.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_sandbox() {
        let sandbox = SandboxConfig::minimal(1);
        assert!(!sandbox.has_capability(&Capability::ReadFile));
        assert!(!sandbox.has_capability(&Capability::NetworkAccess));
    }

    #[test]
    fn test_path_access() {
        use std::fs;

        // Create a custom sandbox with actual temp directory
        let mut sandbox = SandboxConfig::minimal(1);
        let temp_dir = std::env::temp_dir().canonicalize().unwrap();
        sandbox.allow_path(temp_dir.clone());

        // Create a temp file to test with
        let test_file = temp_dir.join("test.txt");
        fs::write(&test_file, b"test").ok();

        // Test with actual temp directory
        assert!(sandbox.can_access_path(&test_file));

        // Test blocked path
        assert!(!sandbox.can_access_path(&PathBuf::from("/etc/passwd")));

        // Clean up
        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_capability_grant_revoke() {
        let mut sandbox = SandboxConfig::minimal(1);
        assert!(!sandbox.has_capability(&Capability::ReadFile));

        sandbox.grant_capability(Capability::ReadFile);
        assert!(sandbox.has_capability(&Capability::ReadFile));

        sandbox.revoke_capability(&Capability::ReadFile);
        assert!(!sandbox.has_capability(&Capability::ReadFile));
    }
}
