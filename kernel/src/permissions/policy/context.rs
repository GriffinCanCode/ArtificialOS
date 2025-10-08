/*!
 * Permission Evaluation Context
 * Provides contextual information for permission decisions
 */

use crate::core::types::Pid;
use crate::security::types::SandboxConfig;
use ahash::HashMap;
use std::time::SystemTime;

/// Request context for permission evaluation
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Current time
    pub timestamp: SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            timestamp: SystemTime::now(),
            metadata: HashMap::default(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluation context containing all information needed for permission decisions
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Process sandbox configuration
    pub sandbox: SandboxConfig,
    /// Request context
    pub request: RequestContext,
}

impl EvaluationContext {
    pub fn new(sandbox: SandboxConfig) -> Self {
        Self {
            sandbox,
            request: RequestContext::new(),
        }
    }

    pub fn with_request_context(mut self, context: RequestContext) -> Self {
        self.request = context;
        self
    }

    pub fn pid(&self) -> Pid {
        self.sandbox.pid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::types::SandboxConfig;

    #[test]
    fn test_context_creation() {
        let req_ctx = RequestContext::new()
            .with_metadata("source", "test")
            .with_metadata("version", "1.0");

        assert_eq!(
            req_ctx.metadata.get("source"),
            Some(&"test".to_string().into())
        );
        assert_eq!(
            req_ctx.metadata.get("version"),
            Some(&"1.0".to_string().into())
        );
    }

    #[test]
    fn test_evaluation_context() {
        let config = SandboxConfig::minimal(100);
        let ctx = EvaluationContext::new(config);

        assert_eq!(ctx.pid(), 100);
    }
}
