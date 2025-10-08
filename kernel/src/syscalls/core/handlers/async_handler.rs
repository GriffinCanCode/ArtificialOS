/*!
 * Async Syscall Handler Trait
 * Async interface for syscall handlers with Tokio runtime support
 */

use crate::core::types::Pid;
use crate::syscalls::types::{Syscall, SyscallResult};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Async trait for handling individual syscalls
/// Each syscall category can implement async handlers for I/O-bound operations
#[allow(dead_code)]
pub trait AsyncSyscallHandler: Send + Sync {
    /// Handle a syscall asynchronously and return the result
    fn handle_async(
        &self,
        pid: Pid,
        syscall: &Syscall,
    ) -> Pin<Box<dyn Future<Output = Option<SyscallResult>> + Send + '_>>;

    /// Get the name of this handler (for logging/debugging)
    fn name(&self) -> &'static str;
}

/// Registry for async syscall handlers
/// Dispatches syscalls to appropriate async handlers based on type
#[derive(Clone)]
#[allow(dead_code)]
pub struct AsyncSyscallHandlerRegistry {
    handlers: Arc<Vec<Arc<dyn AsyncSyscallHandler>>>,
}

#[allow(dead_code)]
impl AsyncSyscallHandlerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Vec::new().into()),
        }
    }

    /// Register a handler in the registry
    pub fn register(mut self, handler: Arc<dyn AsyncSyscallHandler>) -> Self {
        // Get mutable reference to handlers
        let handlers = Arc::make_mut(&mut self.handlers);
        handlers.push(handler);
        self
    }

    /// Dispatch a syscall to the appropriate async handler
    /// Returns None if no handler can handle this syscall
    pub async fn dispatch(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        // Try each handler until one returns Some
        for handler in self.handlers.iter() {
            if let Some(result) = handler.handle_async(pid, syscall).await {
                return Some(result);
            }
        }
        None
    }

    /// Get the number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for AsyncSyscallHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syscalls::types::Syscall;

    struct TestAsyncHandler;

    impl AsyncSyscallHandler for TestAsyncHandler {
        fn handle_async(
            &self,
            _pid: Pid,
            syscall: &Syscall,
        ) -> Pin<Box<dyn Future<Output = Option<SyscallResult>> + Send + '_>> {
            // Clone syscall to move into async block
            let syscall = syscall.clone();
            Box::pin(async move {
                match syscall {
                    Syscall::NetworkRequest { .. } => {
                        // Simulate async I/O
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        Some(SyscallResult::success())
                    }
                    _ => None,
                }
            })
        }

        fn name(&self) -> &'static str {
            "test_async_handler"
        }
    }

    #[tokio::test]
    async fn test_async_handler_dispatch() {
        let registry = AsyncSyscallHandlerRegistry::new().register(Arc::new(TestAsyncHandler));

        let result = registry
            .dispatch(
                1,
                &Syscall::NetworkRequest {
                    url: "http://example.com".to_string(),
                },
            )
            .await;

        assert!(result.is_some());
    }
}
