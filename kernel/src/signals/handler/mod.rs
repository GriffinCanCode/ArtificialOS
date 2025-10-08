/*!
 * Signal Handler - Signal Execution
 * Executes signal actions and manages callback registry
 */

mod callbacks;
mod executor;

// Re-export public API
pub use callbacks::{CallbackRegistry, HandlerFn};
pub use executor::{SignalHandler, SignalOutcome};

