/*!
 * Signals Module
 * UNIX-style signal handling for processes
 */

pub mod core;
pub mod handler;
pub mod integration_support;
pub mod management;

// Re-export public API for convenience
pub use core::{
    PendingSignal, ProcessSignalState, Signal, SignalAction, SignalDelivery, SignalDisposition,
    SignalError, SignalHandlerRegistry, SignalManager, SignalMasking, SignalQueue, SignalResult,
    SignalStateManager, SignalStats, SIGRTMAX, SIGRTMIN,
};
pub use handler::{CallbackRegistry, HandlerFn, SignalHandler, SignalOutcome};
pub use integration_support::{outcome_to_state, requires_immediate_action, should_interrupt};
pub use management::{SignalDeliveryHook, SignalManagerImpl};
