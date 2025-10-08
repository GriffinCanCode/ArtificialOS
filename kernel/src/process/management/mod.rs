/*!
 * Process Management
 * Process manager implementations and priority management
 */

pub mod manager;
pub mod manager_builder;
pub mod manager_scheduler;
mod priority;

// Re-export public types
pub use manager::{Process, ProcessManager};
pub use manager_builder::ProcessManagerBuilder;

// Type alias for backwards compatibility
pub use manager::ProcessManager as ProcessManagerImpl;
