/*!
 * RAII Resource Guards
 *
 * Type-safe, observable, composable resource guards with automatic cleanup.
 *
 * ## Design Principles
 *
 * 1. **Type-State Pattern**: Resource state encoded in types
 * 2. **Observable**: All guards emit events automatically
 * 3. **Composable**: Guards can wrap other guards
 * 4. **Recoverable**: Poisoned guards handled gracefully
 * 5. **Zero-Cost**: Compiles to manual management
 *
 * ## Guard Types
 *
 * - **MemoryGuard**: Scoped memory allocations
 * - **LockGuard**: Typed lock guards with state
 * - **IpcGuard**: IPC resource handles
 * - **TransactionGuard**: Atomic operations with rollback
 * - **CompositeGuard**: Multiple guards as one
 *
 * ## Example
 *
 * ```rust
 * // Memory guard with automatic cleanup
 * let guard = memory_manager.allocate_guard(1024, pid)?;
 * // Use memory
 * // Automatically freed on drop
 *
 * // Transaction guard with rollback
 * let tx = TransactionGuard::begin()?;
 * tx.execute(|| {
 *     // Operations
 * })?;
 * tx.commit(); // Or auto-rollback on drop
 * ```
 */

mod async_task;
mod composite;
mod fd;
mod ipc;
mod lock;
mod memory;
mod observe;
mod syscall;
mod traits;
mod transaction;
mod typed;

pub use async_task::AsyncTaskGuard;
pub use composite::CompositeGuard;
pub use fd::FdGuard;
pub use ipc::{IpcGuard, IpcGuardRef, IpcResourceType};
pub use lock::{LockGuard, LockState, Locked, Unlocked};
pub use memory::{MemoryGuard, MemoryGuardRef};
pub use observe::ObservableGuard;
pub use syscall::SyscallGuard;
pub use traits::{Guard, GuardDrop, GuardRef, Observable, Recoverable};
pub use transaction::{Operation, TransactionGuard, TransactionState};
pub use typed::{TypedGuard, TypedState};

/// Result type for guard operations
pub type GuardResult<T> = Result<T, GuardError>;

/// Errors that can occur during guard operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum GuardError {
    #[error("Resource already released")]
    AlreadyReleased,

    #[error("Guard is poisoned: {0}")]
    Poisoned(String),

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Resource unavailable: {0}")]
    ResourceUnavailable(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Guard metadata for observability
#[derive(Debug, Clone)]
pub struct GuardMetadata {
    pub resource_type: &'static str,
    pub creation_time: std::time::Instant,
    pub pid: Option<crate::core::types::Pid>,
    pub size_bytes: usize,
}

impl GuardMetadata {
    #[inline]
    pub fn new(resource_type: &'static str) -> Self {
        Self {
            resource_type,
            creation_time: std::time::Instant::now(),
            pid: None,
            size_bytes: 0,
        }
    }

    #[inline]
    pub fn with_pid(mut self, pid: crate::core::types::Pid) -> Self {
        self.pid = Some(pid);
        self
    }

    #[inline]
    pub fn with_size(mut self, size: usize) -> Self {
        self.size_bytes = size;
        self
    }

    #[inline]
    pub fn lifetime_micros(&self) -> u64 {
        self.creation_time.elapsed().as_micros() as u64
    }
}
