/*!
 * Signal Integration Support
 * Process state integration and helper functions
 */

mod process;

// Re-export public API
pub use process::{outcome_to_state, requires_immediate_action, should_interrupt, ProcessSignalIntegration};

