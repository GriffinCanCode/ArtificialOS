/*!
 * Process Module
 * Process management, execution, and scheduling
 */

pub mod executor;
pub mod manager;
pub mod scheduler;

// Re-export for convenience
pub use executor::{ExecutionConfig, ProcessExecutor};
pub use manager::{Process, ProcessManager, ProcessManagerBuilder, ProcessState};
pub use scheduler::{Policy, ProcessStats, Scheduler, Stats as SchedulerStats};
