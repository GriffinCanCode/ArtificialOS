/*!
 * Policy Engine
 * Evaluates permission requests against defined policies
 */

use super::context::EvaluationContext;
use crate::permissions::types::{Action, PermissionRequest, PermissionResponse, Resource};
use crate::security::sandbox::capability::{can_access_file, FileOperation};
use crate::security::sandbox::network::check_network_access;
use log::debug;

/// Policy decision
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny,
    Abstain,
}

/// Policy that can evaluate permission requests
pub trait Policy: Send + Sync {
    /// Evaluate a request
    fn evaluate(&self, request: &PermissionRequest, context: &EvaluationContext) -> PolicyDecision;

    /// Policy name
    fn name(&self) -> &str;
}

/// Default policy that uses existing sandbox capabilities
pub struct DefaultPolicy;

impl Policy for DefaultPolicy {
    fn evaluate(&self, request: &PermissionRequest, context: &EvaluationContext) -> PolicyDecision {
        match (&request.resource, request.action) {
            // File system operations
            (Resource::File { path }, Action::Read) => {
                if can_access_file(&context.sandbox.capabilities, FileOperation::Read, path)
                    && context.sandbox.can_access_path(path)
                {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }
            (Resource::File { path }, Action::Write) => {
                if can_access_file(&context.sandbox.capabilities, FileOperation::Write, path)
                    && context.sandbox.can_access_path(path)
                {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }
            (Resource::File { path }, Action::Create) => {
                if can_access_file(&context.sandbox.capabilities, FileOperation::Create, path)
                    && context.sandbox.can_access_path(path)
                {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }
            (Resource::File { path }, Action::Delete) => {
                if can_access_file(&context.sandbox.capabilities, FileOperation::Delete, path)
                    && context.sandbox.can_access_path(path)
                {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }
            (Resource::Directory { path }, Action::List) => {
                if can_access_file(&context.sandbox.capabilities, FileOperation::List, path)
                    && context.sandbox.can_access_path(path)
                {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }

            // Network operations
            (Resource::Network { host, port }, Action::Connect) => {
                if check_network_access(&context.sandbox.network_rules, host, *port) {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }

            // Process operations
            (Resource::Process { .. }, Action::Kill) => {
                use crate::security::types::Capability;
                if context.sandbox.has_capability(&Capability::KillProcess) {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }

            (Resource::Process { .. }, Action::Create) => {
                use crate::security::types::Capability;
                if context.sandbox.has_capability(&Capability::SpawnProcess) {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }

            (Resource::Process { .. }, Action::Inspect) => {
                use crate::security::types::Capability;
                // Allow process inspection with SystemInfo capability
                if context.sandbox.has_capability(&Capability::SystemInfo) {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }

            // System operations
            (Resource::System { name }, Action::Inspect | Action::Read | Action::List) => {
                use crate::security::types::Capability;
                // Time-related system resources require TimeAccess
                if name == "time" && context.sandbox.has_capability(&Capability::TimeAccess) {
                    PolicyDecision::Allow
                // Other system resources require SystemInfo
                } else if context.sandbox.has_capability(&Capability::SystemInfo) {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }

            (Resource::System { .. }, Action::Execute | Action::Write) => {
                use crate::security::types::Capability;
                // Allow execute/write on system resources if SystemInfo capability present
                // This covers operations like GC trigger, setting env vars, etc.
                if context.sandbox.has_capability(&Capability::SystemInfo) {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }

            // IPC operations - check SendMessage/ReceiveMessage
            (Resource::IpcChannel { .. }, Action::Send) => {
                use crate::security::types::Capability;
                if context.sandbox.has_capability(&Capability::SendMessage) {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }

            (Resource::IpcChannel { .. }, Action::Receive) => {
                use crate::security::types::Capability;
                if context.sandbox.has_capability(&Capability::ReceiveMessage) {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::Deny
                }
            }

            // Default deny for unknown combinations
            _ => PolicyDecision::Deny,
        }
    }

    fn name(&self) -> &str {
        "default"
    }
}

/// Policy engine that evaluates requests through multiple policies
pub struct PolicyEngine {
    policies: Vec<Box<dyn Policy>>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            policies: vec![Box::new(DefaultPolicy)],
        }
    }

    /// Add a policy
    pub fn add_policy(&mut self, policy: Box<dyn Policy>) {
        self.policies.push(policy);
    }

    /// Evaluate a request through all policies
    pub fn evaluate(
        &self,
        request: &PermissionRequest,
        context: &EvaluationContext,
    ) -> PermissionResponse {
        debug!(
            "Evaluating permission request: PID={}, action={:?}, resource={:?}",
            request.pid, request.action, request.resource
        );

        // Evaluate through all policies
        for policy in &self.policies {
            match policy.evaluate(request, context) {
                PolicyDecision::Allow => {
                    debug!("Policy '{}' allowed request", policy.name());
                    return PermissionResponse::allow(
                        request.clone(),
                        format!("Allowed by policy '{}'", policy.name().into()),
                    );
                }
                PolicyDecision::Deny => {
                    debug!("Policy '{}' denied request", policy.name());
                    return PermissionResponse::deny(
                        request.clone(),
                        format!("Denied by policy '{}'", policy.name().into()),
                    );
                }
                PolicyDecision::Abstain => {
                    debug!("Policy '{}' abstained", policy.name());
                    continue;
                }
            }
        }

        // If all policies abstained, deny by default
        PermissionResponse::deny(request.clone(), "No policy allowed this request")
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::types::PermissionRequest;
    use crate::security::types::{Capability, SandboxConfig};
    use std::path::PathBuf;

    #[test]
    fn test_default_policy_file_read() {
        let mut config = SandboxConfig::minimal(100);
        config.grant_capability(Capability::ReadFile(None));
        config.allow_path(PathBuf::from("/tmp"));

        let ctx = EvaluationContext::new(config);
        let req = PermissionRequest::file_read(100, PathBuf::from("/tmp/test.txt"));

        let policy = DefaultPolicy;
        assert_eq!(policy.evaluate(&req, &ctx), PolicyDecision::Allow);
    }

    #[test]
    fn test_default_policy_deny() {
        let config = SandboxConfig::minimal(100);
        let ctx = EvaluationContext::new(config);
        let req = PermissionRequest::file_read(100, PathBuf::from("/etc/passwd"));

        let policy = DefaultPolicy;
        assert_eq!(policy.evaluate(&req, &ctx), PolicyDecision::Deny);
    }

    #[test]
    fn test_policy_engine() {
        let mut config = SandboxConfig::minimal(100);
        config.grant_capability(Capability::ReadFile(None));
        config.allow_path(PathBuf::from("/tmp"));

        let ctx = EvaluationContext::new(config);
        let req = PermissionRequest::file_read(100, PathBuf::from("/tmp/test.txt"));

        let engine = PolicyEngine::new();
        let response = engine.evaluate(&req, &ctx);
        assert!(response.is_allowed());
    }
}
