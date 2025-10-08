/*!
 * Signal Callback Registry
 * Manages executable signal handlers
 */

use crate::core::types::Pid;
use crate::signals::core::types::{Signal, SignalError, SignalResult};
use ahash::RandomState;
use dashmap::DashMap;
use log::{debug, info};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Signal handler callback function type
pub type HandlerFn = Arc<dyn Fn(Pid, Signal) -> SignalResult<()> + Send + Sync>;

/// Handler registry for executable callbacks
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic ID counter
#[repr(C, align(64))]
#[derive(Clone)]
pub struct CallbackRegistry {
    handlers: Arc<DashMap<u64, HandlerFn, RandomState>>,
    next_id: Arc<AtomicU64>,
}

impl CallbackRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(DashMap::with_hasher(RandomState::new())),
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Register a new handler callback
    pub fn register<F>(&self, handler: F) -> u64
    where
        F: Fn(Pid, Signal) -> SignalResult<()> + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        self.handlers.insert(id, Arc::new(handler));
        info!("Registered signal handler {}", id);
        id
    }

    /// Execute a handler by ID
    pub fn execute(&self, handler_id: u64, pid: Pid, signal: Signal) -> SignalResult<()> {
        let handler = self.handlers.get(&handler_id).ok_or_else(|| {
            SignalError::HandlerError(format!("Handler {} not found", handler_id))
        })?;

        debug!(
            "Executing handler {} for signal {:?} on PID {}",
            handler_id, signal, pid
        );
        handler(pid, signal)
    }

    /// Unregister a handler
    pub fn unregister(&self, handler_id: u64) -> bool {
        let removed = self.handlers.remove(&handler_id).is_some();
        if removed {
            info!("Unregistered signal handler {}", handler_id);
        }
        removed
    }

    /// Check if handler exists
    pub fn exists(&self, handler_id: u64) -> bool {
        self.handlers.contains_key(&handler_id)
    }

    /// Get handler count
    pub fn count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for CallbackRegistry {
    fn default() -> Self {
        Self::new()
    }
}
