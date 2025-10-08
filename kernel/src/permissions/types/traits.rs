/*!
 * Permission Traits
 * Interfaces for permission checking and management
 */

use super::core::{PermissionRequest, PermissionResponse};
use crate::core::types::Pid;

/// Core permission checking interface
pub trait PermissionChecker: Send + Sync {
    /// Check if a permission request is allowed
    fn check(&self, request: &PermissionRequest) -> PermissionResponse;

    /// Check with audit logging
    fn check_and_audit(&self, request: &PermissionRequest) -> PermissionResponse;

    /// Batch check multiple requests
    fn check_batch(&self, requests: &[PermissionRequest]) -> Vec<PermissionResponse> {
        requests.iter().map(|req| self.check(req)).collect()
    }
}

/// Permission provider interface for accessing sandbox configurations
pub trait PermissionProvider: Send + Sync {
    /// Get sandbox configuration for a PID
    fn get_sandbox(&self, pid: Pid) -> Option<crate::security::types::SandboxConfig>;

    /// Check if a PID has an active sandbox
    fn has_sandbox(&self, pid: Pid) -> bool;
}

/// Combined interface
#[allow(dead_code)]
pub trait PermissionSystem: PermissionChecker + PermissionProvider + Clone + Send + Sync {}

