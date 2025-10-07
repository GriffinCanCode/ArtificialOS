/*!
 * Syscall Handler Trait
 * Defines the interface for syscall handlers and handler registration
 */

use crate::core::types::Pid;
use super::types::{Syscall, SyscallResult};
use std::sync::Arc;

/// Trait for handling individual syscalls
/// Each syscall category (fs, process, ipc, etc.) implements this
pub trait SyscallHandler: Send + Sync {
    /// Handle a syscall and return the result
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult>;

    /// Get the name of this handler (for logging/debugging)
    fn name(&self) -> &'static str;
}

/// Registry for syscall handlers
/// Dispatches syscalls to appropriate handlers based on type
#[derive(Clone)]
pub struct SyscallHandlerRegistry {
    handlers: Arc<Vec<Arc<dyn SyscallHandler>>>,
}

impl SyscallHandlerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Vec::new()),
        }
    }

    /// Register a handler in the registry
    pub fn register(mut self, handler: Arc<dyn SyscallHandler>) -> Self {
        // Get mutable reference to handlers
        let handlers = Arc::make_mut(&mut self.handlers);
        handlers.push(handler);
        self
    }

    /// Dispatch a syscall to the appropriate handler
    /// Returns None if no handler can handle this syscall
    pub fn dispatch(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        // Try each handler until one returns Some
        for handler in self.handlers.iter() {
            if let Some(result) = handler.handle(pid, syscall) {
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

impl Default for SyscallHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syscalls::types::Syscall;

    struct TestHandler;

    impl SyscallHandler for TestHandler {
        fn handle(&self, _pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
            match syscall {
                Syscall::GetSystemInfo => Some(SyscallResult::success()),
                _ => None,
            }
        }

        fn name(&self) -> &'static str {
            "test_handler"
        }
    }

    #[test]
    fn test_registry_dispatch() {
        let registry = SyscallHandlerRegistry::new()
            .register(Arc::new(TestHandler));

        assert_eq!(registry.handler_count(), 1);

        let result = registry.dispatch(1, &Syscall::GetSystemInfo);
        assert!(result.is_some());
        assert!(result.unwrap().is_success());

        let result = registry.dispatch(1, &Syscall::GetCurrentTime);
        assert!(result.is_none());
    }
}
