/*!
 * Sandbox Manager
 */

use super::capability;
use super::network;
use crate::core::types::{Pid, ResourceLimits};
use crate::security::namespace::{IsolationMode, NamespaceConfig, NamespaceManager};
use crate::security::traits::*;
use crate::security::types::*;
use dashmap::DashMap;
use log::{info, warn};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Sandbox manager that enforces security policies
#[derive(Clone)]
pub struct SandboxManager {
    sandboxes: Arc<DashMap<Pid, SandboxConfig>>,
    spawned_counts: Arc<DashMap<Pid, u32>>,
    namespace_manager: Option<NamespaceManager>,
}

impl SandboxManager {
    pub fn new() -> Self {
        info!("Sandbox manager initialized");
        Self {
            sandboxes: Arc::new(DashMap::new()),
            spawned_counts: Arc::new(DashMap::new()),
            namespace_manager: None,
        }
    }

    /// Create sandbox manager with network namespace support
    pub fn with_namespaces() -> Self {
        let ns_manager = NamespaceManager::new();
        info!(
            "Sandbox manager initialized with network namespace support ({})",
            match ns_manager.platform() {
                crate::security::namespace::PlatformType::LinuxNetns => "Linux",
                crate::security::namespace::PlatformType::MacOSFilter => "macOS",
                _ => "Simulation",
            }
        );
        Self {
            sandboxes: Arc::new(DashMap::new()),
            spawned_counts: Arc::new(DashMap::new()),
            namespace_manager: Some(ns_manager),
        }
    }

    /// Get namespace manager reference
    pub fn namespace_manager(&self) -> Option<&NamespaceManager> {
        self.namespace_manager.as_ref()
    }

    /// Create network namespace for a process
    pub fn create_namespace(&self, pid: Pid, mode: IsolationMode) -> Result<(), String> {
        if let Some(ref ns_mgr) = self.namespace_manager {
            let config = match mode {
                IsolationMode::Full => NamespaceConfig::full_isolation(pid),
                IsolationMode::Private => NamespaceConfig::private_network(pid),
                IsolationMode::Shared => NamespaceConfig::shared_network(pid),
                IsolationMode::Bridged => {
                    let mut config = NamespaceConfig::private_network(pid);
                    config.mode = IsolationMode::Bridged;
                    config
                }
            };

            ns_mgr
                .create(config)
                .map_err(|e| format!("Failed to create namespace: {}", e))?;

            info!("Created network namespace for PID {} with mode {:?}", pid, mode);
            Ok(())
        } else {
            Err("Network namespace support not enabled".to_string())
        }
    }

    /// Destroy network namespace for a process
    pub fn destroy_namespace(&self, pid: Pid) -> Result<(), String> {
        if let Some(ref ns_mgr) = self.namespace_manager {
            if let Some(info) = ns_mgr.get_by_pid(pid) {
                ns_mgr
                    .destroy(&info.config.id)
                    .map_err(|e| format!("Failed to destroy namespace: {}", e))?;

                info!("Destroyed network namespace for PID {}", pid);
            }
            Ok(())
        } else {
            Err("Network namespace support not enabled".to_string())
        }
    }

    /// Check if an operation is allowed
    pub fn check_permission(&self, pid: Pid, cap: &Capability) -> bool {
        if let Some(sandbox) = self.sandboxes.get(&pid) {
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
        if let Some(sandbox) = self.sandboxes.get(&pid) {
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
        if let Some(sandbox) = self.sandboxes.get(&pid) {
            capability::can_access_file(&sandbox.capabilities, operation, path)
                && sandbox.can_access_path(path)
        } else {
            false
        }
    }

    /// Check if network access to a host/port is allowed
    pub fn check_network_access(&self, pid: Pid, host: &str, port: Option<u16>) -> bool {
        if let Some(sandbox) = self.sandboxes.get(&pid) {
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

        // Create network namespace if capability is granted
        if config.has_capability(&Capability::NetworkNamespace) {
            if let Err(e) = self.create_namespace(pid, IsolationMode::Private) {
                warn!("Failed to create network namespace for PID {}: {}", pid, e);
            }
        }

        self.sandboxes.insert(pid, config);
        info!("Created sandbox for PID {}", pid);
    }

    fn remove_sandbox(&self, pid: Pid) -> bool {
        // Destroy network namespace if it exists
        if self.namespace_manager.is_some() {
            let _ = self.destroy_namespace(pid);
        }

        if self.sandboxes.remove(&pid).is_some() {
            info!("Removed sandbox for PID {}", pid);
            true
        } else {
            false
        }
    }

    fn has_sandbox(&self, pid: Pid) -> bool {
        self.sandboxes.contains_key(&pid)
    }

    fn get_sandbox(&self, pid: Pid) -> Option<SandboxConfig> {
        self.sandboxes.get(&pid).map(|s| s.clone())
    }

    fn update_sandbox(&self, pid: Pid, config: SandboxConfig) -> bool {
        if self.sandboxes.contains_key(&pid) {
            self.sandboxes.insert(pid, config);
            info!("Updated sandbox for PID {}", pid);
            true
        } else {
            false
        }
    }

    fn stats(&self) -> SandboxStats {
        SandboxStats {
            total_sandboxes: self.sandboxes.len(),
            active_processes: self.sandboxes.len(),
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
        if let Some(mut sandbox) = self.sandboxes.get_mut(&pid) {
            sandbox.grant_capability(cap);
            Ok(())
        } else {
            Err(SecurityError::SandboxNotFound(pid))
        }
    }

    fn revoke_capability(&self, pid: Pid, cap: &Capability) -> SecurityResult<()> {
        if let Some(mut sandbox) = self.sandboxes.get_mut(&pid) {
            sandbox.revoke_capability(cap);
            Ok(())
        } else {
            Err(SecurityError::SandboxNotFound(pid))
        }
    }

    fn get_capabilities(&self, pid: Pid) -> Option<Vec<Capability>> {
        self.sandboxes
            .get(&pid)
            .map(|s| s.capabilities.iter().cloned().collect())
    }
}

impl ResourceLimitProvider for SandboxManager {
    fn get_limits(&self, pid: Pid) -> Option<ResourceLimits> {
        self.sandboxes
            .get(&pid)
            .map(|s| s.resource_limits.clone())
    }

    fn can_spawn_process(&self, pid: Pid) -> bool {
        if let Some(sandbox) = self.sandboxes.get(&pid) {
            let current_count = *self.spawned_counts.get(&pid).map(|r| r.value()).unwrap_or(&0);
            current_count < sandbox.resource_limits.max_processes
        } else {
            false
        }
    }

    fn record_spawn(&self, pid: Pid) {
        self.spawned_counts.entry(pid).and_modify(|count| *count += 1).or_insert(1);
    }

    fn record_termination(&self, pid: Pid) {
        if let Some(mut count) = self.spawned_counts.get_mut(&pid) {
            *count = count.saturating_sub(1);
        }
    }

    fn get_spawn_count(&self, pid: Pid) -> u32 {
        *self.spawned_counts.get(&pid).map(|r| r.value()).unwrap_or(&0)
    }
}

impl PathAccessControl for SandboxManager {
    fn can_access(&self, pid: Pid, path: &Path) -> bool {
        self.check_path_access(pid, &path.to_path_buf())
    }

    fn allow_path(&self, pid: Pid, path: PathBuf) -> SecurityResult<()> {
        if let Some(mut sandbox) = self.sandboxes.get_mut(&pid) {
            sandbox.allow_path(path);
            Ok(())
        } else {
            Err(SecurityError::SandboxNotFound(pid))
        }
    }

    fn block_path(&self, pid: Pid, path: PathBuf) -> SecurityResult<()> {
        if let Some(mut sandbox) = self.sandboxes.get_mut(&pid) {
            sandbox.block_path(path);
            Ok(())
        } else {
            Err(SecurityError::SandboxNotFound(pid))
        }
    }

    fn get_allowed_paths(&self, pid: Pid) -> Option<Vec<PathBuf>> {
        self.sandboxes
            .get(&pid)
            .map(|s| s.allowed_paths.clone())
    }

    fn get_blocked_paths(&self, pid: Pid) -> Option<Vec<PathBuf>> {
        self.sandboxes
            .get(&pid)
            .map(|s| s.blocked_paths.clone())
    }
}
