/*!
 * Security Traits
 * Security and sandboxing abstractions
 */

use super::types::*;
use crate::core::types::{Pid, ResourceLimits};
use std::path::{Path, PathBuf};

/// Sandbox management interface
pub trait SandboxProvider: Send + Sync {
    /// Create a new sandbox for a process
    fn create_sandbox(&self, config: SandboxConfig);

    /// Remove a sandbox
    fn remove_sandbox(&self, pid: Pid) -> bool;

    /// Check if a sandbox exists
    fn has_sandbox(&self, pid: Pid) -> bool;

    /// Get sandbox configuration (clones entire config)
    fn get_sandbox(&self, pid: Pid) -> Option<SandboxConfig>;

    /// Update sandbox configuration
    fn update_sandbox(&self, pid: Pid, config: SandboxConfig) -> bool;

    /// Get sandbox statistics
    fn stats(&self) -> SandboxStats;
}

/// Capability checking interface
pub trait CapabilityChecker: Send + Sync {
    /// Check if a process has a specific capability
    fn check_permission(&self, pid: Pid, cap: &Capability) -> bool;

    /// Check if a path access is allowed
    fn check_path_access(&self, pid: Pid, path: &PathBuf) -> bool;

    /// Check if network access is allowed
    fn check_network_access(&self, pid: Pid) -> bool;
}

/// Capability management interface
pub trait CapabilityManager: Send + Sync {
    /// Grant a capability to a process
    fn grant_capability(&self, pid: Pid, cap: Capability) -> SecurityResult<()>;

    /// Revoke a capability from a process
    fn revoke_capability(&self, pid: Pid, cap: &Capability) -> SecurityResult<()>;

    /// Get all capabilities for a process
    fn get_capabilities(&self, pid: Pid) -> Option<Vec<Capability>>;
}

/// Resource limits interface
pub trait ResourceLimitProvider: Send + Sync {
    /// Get resource limits for a process
    fn get_limits(&self, pid: Pid) -> Option<ResourceLimits>;

    /// Check if a process can spawn another process
    fn can_spawn_process(&self, pid: Pid) -> bool;

    /// Record a spawned process
    fn record_spawn(&self, pid: Pid);

    /// Record a terminated process
    fn record_termination(&self, pid: Pid);

    /// Get spawned process count
    fn get_spawn_count(&self, pid: Pid) -> u32;
}

/// OS-level resource limit enforcement
pub trait OsLimitEnforcer: Send + Sync {
    /// Apply OS-level resource limits to a process
    fn apply(&self, os_pid: u32, limits: &Limits) -> LimitsResult<()>;

    /// Remove OS-level resource limits for a process
    fn remove(&self, os_pid: u32) -> LimitsResult<()>;

    /// Check if limits can be enforced on this platform
    fn is_supported(&self) -> bool;
}

/// Security audit logging
pub trait SecurityAuditor: Send + Sync {
    /// Log a security event
    fn log_event(&self, event: SecurityEvent);

    /// Get recent security events
    fn recent_events(&self, limit: usize) -> Vec<SecurityEvent>;

    /// Get events for a specific process
    fn events_for_process(&self, pid: Pid) -> Vec<SecurityEvent>;
}

/// Path access control
pub trait PathAccessControl: Send + Sync {
    /// Check if a path can be accessed
    fn can_access(&self, pid: Pid, path: &Path) -> bool;

    /// Add an allowed path for a process
    fn allow_path(&self, pid: Pid, path: PathBuf) -> SecurityResult<()>;

    /// Add a blocked path for a process
    fn block_path(&self, pid: Pid, path: PathBuf) -> SecurityResult<()>;

    /// Get allowed paths for a process
    fn get_allowed_paths(&self, pid: Pid) -> Option<Vec<PathBuf>>;

    /// Get blocked paths for a process
    fn get_blocked_paths(&self, pid: Pid) -> Option<Vec<PathBuf>>;
}

/// Combined security manager interface
pub trait SecurityManager:
    SandboxProvider
    + CapabilityChecker
    + CapabilityManager
    + ResourceLimitProvider
    + PathAccessControl
    + Clone
    + Send
    + Sync
{
}

/// Implement SecurityManager for types that implement all required traits
impl<T> SecurityManager for T where
    T: SandboxProvider
        + CapabilityChecker
        + CapabilityManager
        + ResourceLimitProvider
        + PathAccessControl
        + Clone
        + Send
        + Sync
{
}
