/*!
 * Sandbox Manager
 */

use super::capability;
use super::network;
use crate::core::types::{Pid, ResourceLimits};
use crate::security::traits::*;
use crate::security::types::*;
use log::{info, warn};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Sandbox manager that enforces security policies
#[derive(Clone)]
pub struct SandboxManager {
    sandboxes: Arc<RwLock<HashMap<Pid, SandboxConfig>>>,
    spawned_counts: Arc<RwLock<HashMap<Pid, u32>>>,
}

impl SandboxManager {
    pub fn new() -> Self {
        info!("Sandbox manager initialized");
        Self {
            sandboxes: Arc::new(RwLock::new(HashMap::new())),
            spawned_counts: Arc::new(RwLock::new(HashMap::new())),
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

    /// Check if a file operation is allowed on a specific path
    pub fn check_file_operation(
        &self,
        pid: Pid,
        operation: capability::FileOperation,
        path: &Path,
    ) -> bool {
        let sandboxes = self.sandboxes.read();
        if let Some(sandbox) = sandboxes.get(&pid) {
            capability::can_access_file(&sandbox.capabilities, operation, path)
                && sandbox.can_access_path(path)
        } else {
            false
        }
    }

    /// Check if network access to a host/port is allowed
    pub fn check_network_access(&self, pid: Pid, host: &str, port: Option<u16>) -> bool {
        let sandboxes = self.sandboxes.read();
        if let Some(sandbox) = sandboxes.get(&pid) {
            network::check_network_access(&sandbox.network_rules, host, port)
        } else {
            false
        }
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

// Trait implementations

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
            permission_denials: 0,
            capability_checks: 0,
        }
    }
}

impl CapabilityChecker for SandboxManager {
    fn check_permission(&self, pid: Pid, cap: &Capability) -> bool {
        self.check_permission(pid, cap)
    }

    fn check_path_access(&self, pid: Pid, path: &PathBuf) -> bool {
        self.check_path_access(pid, path)
    }

    fn check_network_access(&self, pid: Pid) -> bool {
        self.check_network_access(pid, "*", None)
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
