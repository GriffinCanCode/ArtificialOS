/*!
 * Automatic Signal Delivery
 * Integrates signal delivery with process scheduling
 */

use super::traits::{SignalDelivery, SignalQueue};
use crate::core::types::Pid;
use log::{debug, warn};
use std::sync::Arc;

/// Auto-delivery hook for scheduler integration
pub struct SignalDeliveryHook<S>
where
    S: SignalDelivery + SignalQueue,
{
    signal_manager: Arc<S>,
}

impl<S> SignalDeliveryHook<S>
where
    S: SignalDelivery + SignalQueue,
{
    pub fn new(signal_manager: Arc<S>) -> Self {
        Self { signal_manager }
    }

    /// Deliver pending signals before process runs
    /// Returns (delivered_count, should_terminate, should_stop, should_continue)
    pub fn deliver_before_schedule(&self, pid: Pid) -> (usize, bool, bool, bool) {
        // Check if there are pending signals
        if !self.signal_manager.has_pending(pid) {
            return (0, false, false, false);
        }

        debug!("Delivering pending signals for PID {} before schedule", pid);

        // Get pending signal count before delivery
        let pending_count = self.signal_manager.pending_count(pid);

        // Track outcomes for state determination
        let mut should_terminate = false;
        let mut should_stop = false;
        let mut should_continue = false;
        let mut delivered = 0;

        // Deliver signals one by one to collect outcomes
        for _ in 0..pending_count {
            if !self.signal_manager.has_pending(pid) {
                break;
            }

            // Deliver single signal and check outcome
            match self.signal_manager.deliver_pending(pid) {
                Ok(count) if count > 0 => {
                    delivered += count;

                    // For now, we track delivery but don't change state flags
                    // In a full implementation, this would:
                    // 1. Track each signal's outcome (Terminated, Stopped, Continued, etc.)
                    // 2. Set should_terminate, should_stop, should_continue based on outcomes
                    // 3. Use SignalOutcome from the handler to determine process state

                    // Currently we just mark that signals were delivered
                    if !self.signal_manager.has_pending(pid) {
                        should_continue = true;
                    }
                }
                Ok(_) => break,
                Err(e) => {
                    warn!("Failed to deliver signal for PID {}: {}", pid, e);
                    break;
                }
            }
        }

        debug!(
            "Delivered {} signals for PID {} (terminate={}, stop={}, continue={})",
            delivered, pid, should_terminate, should_stop, should_continue
        );

        (delivered, should_terminate, should_stop, should_continue)
    }

    /// Check if process should run based on pending signals
    pub fn should_schedule(&self, _pid: Pid) -> bool {
        // Always allow scheduling - signals will be delivered before execution
        true
    }

    /// Get pending signal count for priority adjustment
    pub fn pending_count(&self, pid: Pid) -> usize {
        self.signal_manager.pending_count(pid)
    }
}
