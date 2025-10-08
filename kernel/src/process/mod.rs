/*!
 * Process Module
 * Process management, execution, and scheduling
 *
 * # Organization
 *
 * - **core**: Core types and traits (ProcessInfo, ProcessState, etc.)
 * - **execution**: OS-level process execution and preemption
 * - **lifecycle**: Process initialization, cleanup, and budgeting
 * - **management**: Process manager implementations
 * - **scheduler**: CPU scheduling with multiple policies
 * - **resources**: Resource cleanup system
 */

pub mod core;
pub mod execution;
pub mod lifecycle;
pub mod management;
pub mod resources;
pub mod scheduler;

// Re-export core types for convenience
pub use core::*;

// Re-export execution types
pub use execution::{ExecutingProcess, PreemptionController, ProcessExecutor};

// Re-export lifecycle types
pub use lifecycle::{
    LifecycleError, LifecycleRegistry, LifecycleResult, ProcessInitConfig, ResourceBudget,
    ResourceTracker, ResourceUsage,
};

// Re-export management types
pub use management::{Process, ProcessManager, ProcessManagerBuilder, ProcessManagerImpl};

// Re-export scheduler types
pub use scheduler::{Scheduler, SchedulerCommand, SchedulerTask};

// Backwards compatibility aliases
pub use execution::ProcessExecutor as ProcessExecutorImpl;
