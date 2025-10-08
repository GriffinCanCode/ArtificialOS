/*!
 * Signals Module
 * UNIX-style signal handling for processes
 */

mod callbacks;
mod delivery;
mod handler;
pub mod integration;
mod internal_types;
mod manager;
pub mod traits;
pub mod types;

// Re-export public API
pub use callbacks::{CallbackRegistry, HandlerFn};
pub use delivery::SignalDeliveryHook;
pub use handler::{SignalHandler, SignalOutcome};
pub use integration::{outcome_to_state, requires_immediate_action, should_interrupt};
pub use manager::SignalManagerImpl;
pub use traits::*;
pub use types::{
    PendingSignal, ProcessSignalState, Signal, SignalAction, SignalDisposition, SignalError,
    SignalResult, SignalStats, SIGRTMAX, SIGRTMIN,
};
