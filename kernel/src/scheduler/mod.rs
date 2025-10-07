/*!
 * Scheduler Module
 * CPU scheduling operations and policy management
 */

pub mod traits;
pub mod types;

// Re-export public API
pub use traits::{
    PriorityControl, SchedulerControl, SchedulerPolicy, SchedulerStats, SchedulerSyscalls,
};
pub use types::{
    apply_priority_op, validate_priority, PriorityOp, SchedulerPolicy as Policy, TimeQuantum,
    DEFAULT_PRIORITY, MAX_PRIORITY, MIN_PRIORITY,
};
