/*!
 * Sandbox Module
 * Provides secure, isolated execution environment for processes
 */

use log::{info, warn};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Capabilities that can be granted to sandboxed processes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    // File system
    ReadFile,
    WriteFile,
    CreateFile,
    DeleteFile,
    ListDirectory,

    // Process
    SpawnProcess,
    KillProcess,

    // Network
    NetworkAccess,
    BindPort,

    // System
    SystemInfo,
    TimeAccess,

    // IPC
    SendMessage,
    ReceiveMessage,
}

/// Resource limits for sandboxed processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: usize,
    pub max_cpu_time_ms: u64,
    pub max_file_descriptors: u32,
    pub max_processes: u32,
    pub max_network_connections: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024, // 512MB
            max_cpu_time_ms: 60_000,             // 60 seconds
            max_file_descriptors: 100,
            max_processes: 10,
            max_network_connections: 20,
        }
    }
}

/// Sandbox configuration for a process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub pid: u32,
    pub capabilities: HashSet<Capability>,
    pub resource_limits: ResourceLimits,
    pub allowed_paths: Vec<PathBuf>,
    pub blocked_paths: Vec<PathBuf>,
    pub allow_network: bool,
    pub environment_vars: Vec<(String, String)>,
}

impl SandboxConfig {
    /// Create a minimal sandbox (most restrictive)
    pub fn minimal(pid: u32) -> Self {
        Self {
            pid,
            capabilities: HashSet::new(),
            resource_limits: ResourceLimits {
                max_memory_bytes: 128 * 1024 * 1024, // 128MB
                max_cpu_time_ms: 30_000,             // 30 seconds
                max_file_descriptors: 20,
                max_processes: 1,
                max_network_connections: 0,
            },
            allowed_paths: vec![],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/bin"),
                PathBuf::from("/sbin"),
                PathBuf::from("/usr/bin"),
                PathBuf::from("/usr/sbin"),
            ],
            allow_network: false,
            environment_vars: vec![],
        }
    }

    /// Create a standard sandbox (balanced)
    pub fn standard(pid: u32) -> Self {
        let mut capabilities = HashSet::new();
        capabilities.insert(Capability::ReadFile);
        capabilities.insert(Capability::WriteFile);
        capabilities.insert(Capability::SystemInfo);
        capabilities.insert(Capability::TimeAccess);

        Self {
            pid,
            capabilities,
            resource_limits: ResourceLimits::default(),
            allowed_paths: vec![PathBuf::from("/tmp"), PathBuf::from("/var/tmp")],
            blocked_paths: vec![PathBuf::from("/etc/passwd"), PathBuf::from("/etc/shadow")],
            allow_network: false,
            environment_vars: vec![],
        }
    }

    /// Create a privileged sandbox (for trusted apps)
    pub fn privileged(pid: u32) -> Self {
        let mut capabilities = HashSet::new();
        capabilities.insert(Capability::ReadFile);
        capabilities.insert(Capability::WriteFile);
        capabilities.insert(Capability::CreateFile);
        capabilities.insert(Capability::DeleteFile);
        capabilities.insert(Capability::ListDirectory);
        capabilities.insert(Capability::SpawnProcess);
        capabilities.insert(Capability::NetworkAccess);
        capabilities.insert(Capability::SystemInfo);
        capabilities.insert(Capability::TimeAccess);
        capabilities.insert(Capability::SendMessage);
        capabilities.insert(Capability::ReceiveMessage);

        Self {
            pid,
            capabilities,
            resource_limits: ResourceLimits {
                max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB
                max_cpu_time_ms: 300_000,                 // 5 minutes
                max_file_descriptors: 500,
                max_processes: 50,
                max_network_connections: 100,
            },
            allowed_paths: vec![PathBuf::from("/")], // Full access
            blocked_paths: vec![],
            allow_network: true,
            environment_vars: vec![],
        }
    }

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
    sandboxes: Arc<RwLock<std::collections::HashMap<u32, SandboxConfig>>>,
    // Track spawned processes per PID for limit enforcement
    spawned_counts: Arc<RwLock<std::collections::HashMap<u32, u32>>>,
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
    pub fn remove_sandbox(&self, pid: u32) -> bool {
        if self.sandboxes.write().remove(&pid).is_some() {
            info!("Removed sandbox for PID {}", pid);
            true
        } else {
            false
        }
    }

    /// Check if an operation is allowed
    pub fn check_permission(&self, pid: u32, cap: &Capability) -> bool {
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
    pub fn check_path_access(&self, pid: u32, path: &PathBuf) -> bool {
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
    pub fn get_limits(&self, pid: u32) -> Option<ResourceLimits> {
        self.sandboxes
            .read()
            .get(&pid)
            .map(|s| s.resource_limits.clone())
    }

    /// Get sandbox config for a process (clones entire config - use sparingly)
    pub fn get_sandbox(&self, pid: u32) -> Option<SandboxConfig> {
        self.sandboxes.read().get(&pid).cloned()
    }

    /// Check if sandbox exists without cloning
    pub fn has_sandbox(&self, pid: u32) -> bool {
        self.sandboxes.read().contains_key(&pid)
    }

    /// Update sandbox config
    pub fn update_sandbox(&self, pid: u32, config: SandboxConfig) -> bool {
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
    pub fn can_spawn_process(&self, pid: u32) -> bool {
        let sandboxes = self.sandboxes.read();
        if let Some(sandbox) = sandboxes.get(&pid) {
            let current_count = *self.spawned_counts.read().get(&pid).unwrap_or(&0);
            current_count < sandbox.resource_limits.max_processes
        } else {
            false
        }
    }

    /// Record a spawned process for a PID
    pub fn record_spawn(&self, pid: u32) {
        let mut counts = self.spawned_counts.write();
        *counts.entry(pid).or_insert(0) += 1;
    }

    /// Record a terminated spawned process for a PID
    pub fn record_termination(&self, pid: u32) {
        let mut counts = self.spawned_counts.write();
        if let Some(count) = counts.get_mut(&pid) {
            *count = count.saturating_sub(1);
        }
    }

    /// Get spawned process count for a PID
    pub fn get_spawn_count(&self, pid: u32) -> u32 {
        *self.spawned_counts.read().get(&pid).unwrap_or(&0)
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
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
