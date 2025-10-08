/*!
 * Process Module
 * Process management, execution, and scheduling
 */

pub mod atomic_stats;
pub mod budget;
mod cleanup;
pub mod executor;
pub mod lifecycle;
pub mod manager;
pub mod manager_builder;
pub mod manager_scheduler;
pub mod preemption;
mod priority;
pub mod resources;
pub mod scheduler;
pub mod scheduler_task;
pub mod traits;
pub mod types;
mod validation;

// Re-export types for convenience
pub use types::*;

// Re-export implementations
pub use budget::{ResourceBudget, ResourceTracker, ResourceUsage};
pub use executor::ProcessExecutor as ProcessExecutorImpl;
pub use lifecycle::{LifecycleRegistry, ProcessInitConfig};
pub use manager::ProcessManager as ProcessManagerImpl;
pub use manager_builder::ProcessManagerBuilder;
pub use preemption::PreemptionController;
pub use scheduler::Scheduler;
pub use scheduler_task::{SchedulerCommand, SchedulerTask};
