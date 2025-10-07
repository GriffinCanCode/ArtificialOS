/*!
 * Signal Traits
 * Signal handling abstractions
 */

use super::types::{ProcessSignalState, Signal, SignalAction, SignalResult, SignalStats};
use crate::core::types::Pid;

/// Signal delivery interface
pub trait SignalDelivery: Send + Sync {
    /// Send a signal to a process
    fn send(&self, sender_pid: Pid, target_pid: Pid, signal: Signal) -> SignalResult<()>;

    /// Broadcast signal to all processes
    fn broadcast(&self, sender_pid: Pid, signal: Signal) -> SignalResult<u32>;

    /// Deliver pending signals to a process
    fn deliver_pending(&self, pid: Pid) -> SignalResult<usize>;
}

/// Signal handler management
pub trait SignalHandlerRegistry: Send + Sync {
    /// Register signal handler
    fn register_handler(&self, pid: Pid, signal: Signal, action: SignalAction) -> SignalResult<()>;

    /// Unregister signal handler
    fn unregister_handler(&self, pid: Pid, signal: Signal) -> SignalResult<()>;

    /// Get handler action for signal
    fn get_handler(&self, pid: Pid, signal: Signal) -> Option<SignalAction>;

    /// Reset all handlers to default
    fn reset_handlers(&self, pid: Pid) -> SignalResult<()>;
}

/// Signal blocking/masking
pub trait SignalMasking: Send + Sync {
    /// Block a signal
    fn block_signal(&self, pid: Pid, signal: Signal) -> SignalResult<()>;

    /// Unblock a signal
    fn unblock_signal(&self, pid: Pid, signal: Signal) -> SignalResult<()>;

    /// Check if signal is blocked
    fn is_blocked(&self, pid: Pid, signal: Signal) -> bool;

    /// Get all blocked signals
    fn get_blocked(&self, pid: Pid) -> Vec<Signal>;

    /// Set signal mask
    fn set_mask(&self, pid: Pid, signals: Vec<Signal>) -> SignalResult<()>;
}

/// Signal queue management
pub trait SignalQueue: Send + Sync {
    /// Get pending signals for a process
    fn pending_signals(&self, pid: Pid) -> Vec<Signal>;

    /// Check if process has pending signals
    fn has_pending(&self, pid: Pid) -> bool;

    /// Clear pending signals
    fn clear_pending(&self, pid: Pid) -> SignalResult<usize>;

    /// Get pending signal count
    fn pending_count(&self, pid: Pid) -> usize;
}

/// Signal state management
pub trait SignalStateManager: Send + Sync {
    /// Get process signal state
    fn get_state(&self, pid: Pid) -> Option<ProcessSignalState>;

    /// Initialize signal state for new process
    fn initialize_process(&self, pid: Pid) -> SignalResult<()>;

    /// Cleanup signal state on process termination
    fn cleanup_process(&self, pid: Pid) -> SignalResult<()>;

    /// Get signal statistics
    fn stats(&self) -> SignalStats;
}

/// Combined signal manager trait
pub trait SignalManager:
    SignalDelivery
    + SignalHandlerRegistry
    + SignalMasking
    + SignalQueue
    + SignalStateManager
    + Clone
    + Send
    + Sync
{
}

/// Implement SignalManager for types that implement all required traits
impl<T> SignalManager for T where
    T: SignalDelivery
        + SignalHandlerRegistry
        + SignalMasking
        + SignalQueue
        + SignalStateManager
        + Clone
        + Send
        + Sync
{
}
