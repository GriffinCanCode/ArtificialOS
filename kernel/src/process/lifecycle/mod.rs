/*!
 * Process Lifecycle Management
 * Process initialization, cleanup, and resource budgeting
 */

pub mod budget;
mod cleanup;
pub mod lifecycle;

// Re-export public types
pub use budget::{ResourceBudget, ResourceTracker, ResourceUsage};
pub use lifecycle::{LifecycleError, LifecycleRegistry, LifecycleResult, ProcessInitConfig};

// Internal cleanup utilities
pub(crate) use cleanup::{cleanup_os_process, cleanup_preemption, cleanup_scheduler};

