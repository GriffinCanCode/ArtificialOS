/*!
 * Signals Module
 * UNIX-style signal handling for processes
 */

mod handler;
mod manager;
pub mod traits;
pub mod types;

// Re-export public API
pub use handler::{SignalHandler, SignalOutcome};
pub use manager::SignalManagerImpl;
pub use traits::*;
pub use types::{
    PendingSignal, ProcessSignalState, Signal, SignalAction, SignalDisposition, SignalError,
    SignalResult, SignalStats,
};
