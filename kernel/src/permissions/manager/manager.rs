/*!
 * Permission Manager
 * Central manager for all permission checks across the kernel
 */

use crate::permissions::audit::{AuditEvent, AuditLogger, AuditStats};
use crate::permissions::cache::{CacheStats, PermissionCache};
use crate::permissions::policy::{EvaluationContext, PolicyEngine};
use crate::permissions::types::{
    PermissionChecker, PermissionProvider, PermissionRequest, PermissionResponse, PermissionSystem,
};
use crate::core::types::Pid;
use crate::monitoring::Collector;
use crate::security::traits::SandboxProvider;
use crate::security::SandboxManager;
use log::{debug, warn};
use std::sync::Arc;

/// Central permission manager
#[derive(Clone)]
pub struct PermissionManager {
    /// Sandbox manager for accessing configurations
    sandbox: SandboxManager,
    /// Policy engine
    policy: Arc<PolicyEngine>,
    /// Permission cache
    cache: Arc<PermissionCache>,
    /// Audit logger
    audit: Arc<AuditLogger>,
    /// Observability collector
    collector: Option<Arc<Collector>>,
}

impl PermissionManager {
    /// Create new permission manager
    pub fn new(sandbox: SandboxManager) -> Self {
        debug!("Initializing centralized permission manager");
        Self {
            sandbox,
            policy: Arc::new(PolicyEngine::new().into()),
            cache: Arc::new(PermissionCache::default().into()),
            audit: Arc::new(AuditLogger::new().into()),
            collector: None,
        }
    }

    /// Add observability collector
    pub fn with_collector(mut self, collector: Arc<Collector>) -> Self {
        self.collector = Some(collector);
        self
    }

    /// Set collector after construction
    pub fn set_collector(&mut self, collector: Arc<Collector>) {
        self.collector = Some(collector);
    }

    /// Create with custom configuration
    pub fn with_config(
        sandbox: SandboxManager,
        cache: PermissionCache,
        policy: PolicyEngine,
    ) -> Self {
        Self {
            sandbox,
            policy: Arc::new(policy),
            cache: Arc::new(cache),
            audit: Arc::new(AuditLogger::new().into()),
            collector: None,
        }
    }

    /// Get policy engine (for adding custom policies)
    ///
    /// Returns None if the policy Arc has other strong references (shared state)
    pub fn policy_mut(&mut self) -> Option<&mut PolicyEngine> {
        Arc::get_mut(&mut self.policy)
    }

    /// Get audit logger
    pub fn audit(&self) -> &AuditLogger {
        &self.audit
    }

    /// Invalidate cache for a PID
    pub fn invalidate_cache(&self, pid: Pid) {
        self.cache.invalidate_pid(pid);
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Get audit statistics
    pub fn audit_stats(&self) -> AuditStats {
        self.audit.stats()
    }

    /// Internal check without caching
    fn check_internal(&self, request: &PermissionRequest) -> PermissionResponse {
        // Get sandbox configuration
        let sandbox_config = match self.sandbox.get_sandbox(request.pid) {
            Some(config) => config,
            None => {
                warn!("No sandbox found for PID {}", request.pid);
                return PermissionResponse::deny(
                    request.clone(),
                    format!("No sandbox configured for PID {}", request.pid),
                );
            }
        };

        // Create evaluation context
        let context = EvaluationContext::new(sandbox_config);

        // Evaluate through policy engine
        let response = self.policy.evaluate(request, &context);

        // Emit permission denied event if denied
        if !response.is_allowed() {
            if let Some(ref collector) = self.collector {
                use crate::monitoring::{Category, Event, Payload, Severity};
                collector.emit(
                    Event::new(
                        Severity::Warn,
                        Category::Security,
                        Payload::PermissionDenied {
                            operation: format!("{:?}", request.action),
                            required: format!("{:?}", request.resource),
                        },
                    )
                    .with_pid(request.pid),
                );
            }
        }

        response
    }
}

impl PermissionChecker for PermissionManager {
    fn check(&self, request: &PermissionRequest) -> PermissionResponse {
        // Try cache first
        if let Some(cached) = self.cache.get(request) {
            debug!("Cache hit for PID {} permission check", request.pid);
            return cached;
        }

        // Perform check
        let response = self.check_internal(request);

        // Cache the result
        self.cache.put(request.clone(), response.clone());

        response
    }

    fn check_and_audit(&self, request: &PermissionRequest) -> PermissionResponse {
        let response = self.check(request);

        // Log to audit trail
        let event = AuditEvent::new(request.clone(), response.clone());
        self.audit.log(event);

        response
    }
}

impl PermissionProvider for PermissionManager {
    fn get_sandbox(&self, pid: Pid) -> Option<crate::security::types::SandboxConfig> {
        self.sandbox.get_sandbox(pid)
    }

    fn has_sandbox(&self, pid: Pid) -> bool {
        self.sandbox.has_sandbox(pid)
    }
}

impl PermissionSystem for PermissionManager {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::types::PermissionRequest;
    use crate::security::types::{Capability, SandboxConfig};
    use std::path::PathBuf;

    #[test]
    fn test_permission_manager_allow() {
        let sandbox = SandboxManager::new();
        let mut config = SandboxConfig::minimal(100);
        config.grant_capability(Capability::ReadFile(None));
        config.allow_path(PathBuf::from("/tmp"));
        sandbox.create_sandbox(config);

        let manager = PermissionManager::new(sandbox);
        let req = PermissionRequest::file_read(100, PathBuf::from("/tmp/test.txt"));

        let resp = manager.check(&req);
        assert!(resp.is_allowed());
    }

    #[test]
    fn test_permission_manager_deny() {
        let sandbox = SandboxManager::new();
        let config = SandboxConfig::minimal(100);
        sandbox.create_sandbox(config);

        let manager = PermissionManager::new(sandbox);
        let req = PermissionRequest::file_read(100, PathBuf::from("/etc/passwd"));

        let resp = manager.check(&req);
        assert!(!resp.is_allowed());
    }

    #[test]
    fn test_permission_caching() {
        let sandbox = SandboxManager::new();
        let mut config = SandboxConfig::minimal(100);
        config.grant_capability(Capability::ReadFile(None));
        config.allow_path(PathBuf::from("/tmp"));
        sandbox.create_sandbox(config);

        let manager = PermissionManager::new(sandbox);
        let req = PermissionRequest::file_read(100, PathBuf::from("/tmp/test.txt"));

        // First check - cache miss
        let resp1 = manager.check(&req);
        assert!(resp1.is_allowed());
        assert!(!resp1.cached);

        // Second check - cache hit
        let resp2 = manager.check(&req);
        assert!(resp2.is_allowed());
        assert!(resp2.cached);

        let stats = manager.cache_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_audit_logging() {
        let sandbox = SandboxManager::new();
        let config = SandboxConfig::minimal(100);
        sandbox.create_sandbox(config);

        let manager = PermissionManager::new(sandbox);
        let req = PermissionRequest::file_read(100, PathBuf::from("/etc/passwd"));

        let resp = manager.check_and_audit(&req);
        assert!(!resp.is_allowed());

        let audit_stats = manager.audit_stats();
        assert_eq!(audit_stats.total_events, 1);
        assert_eq!(audit_stats.total_denials, 1);
    }

    #[test]
    fn test_batch_check() {
        let sandbox = SandboxManager::new();
        let mut config = SandboxConfig::minimal(100);
        config.grant_capability(Capability::ReadFile(None));
        config.allow_path(PathBuf::from("/tmp"));
        sandbox.create_sandbox(config);

        let manager = PermissionManager::new(sandbox);
        let requests = vec![
            PermissionRequest::file_read(100, PathBuf::from("/tmp/test1.txt").into()),
            PermissionRequest::file_read(100, PathBuf::from("/tmp/test2.txt").into()),
            PermissionRequest::file_read(100, PathBuf::from("/etc/passwd").into()),
        ];

        let responses = manager.check_batch(&requests);
        assert_eq!(responses.len(), 3);
        assert!(responses[0].is_allowed());
        assert!(responses[1].is_allowed());
        assert!(!responses[2].is_allowed());
    }
}

