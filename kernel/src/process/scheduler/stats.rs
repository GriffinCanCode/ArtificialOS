/*!
 * Scheduler Statistics
 * Track and report scheduler performance metrics
 */

use super::Scheduler;
use crate::process::core::types::{SchedulerStats, SchedulingPolicy};

impl Scheduler {
    /// Get scheduler statistics (lock-free snapshot)
    pub fn stats(&self) -> SchedulerStats {
        self.stats.snapshot()
    }

    /// Get minimum virtual runtime (for fair scheduler normalization)
    #[allow(dead_code)]
    pub(super) fn min_vruntime(&self) -> u64 {
        let policy = *self.policy.read();
        if policy != SchedulingPolicy::Fair {
            return 0;
        }

        let mut min = u64::MAX;

        // Check current
        if let Some(ref entry) = *self.current.read() {
            min = min.min(entry.vruntime);
        }

        // Check queue
        let queue = self.fair_queue.read();
        for entry in queue.iter() {
            min = min.min(entry.0.vruntime);
        }

        if min == u64::MAX {
            0
        } else {
            min
        }
    }
}
