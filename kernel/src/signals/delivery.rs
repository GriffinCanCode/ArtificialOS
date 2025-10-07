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

        // Deliver all pending signals
        match self.signal_manager.deliver_pending(pid) {
            Ok(count) => {
                debug!("Delivered {} signals for PID {}", count, pid);
                // TODO: Check outcomes to determine process state changes
                // For now, return conservative values
                (count, false, false, false)
            }
            Err(e) => {
                warn!("Failed to deliver signals for PID {}: {}", pid, e);
                (0, false, false, false)
            }
        }
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
