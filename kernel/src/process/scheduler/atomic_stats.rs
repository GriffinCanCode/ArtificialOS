/*!
 * Lock-Free Scheduler Statistics
 * Uses atomic counters for zero-contention stats tracking in hot scheduling paths
 */

use crate::process::core::types::{SchedulerStats, SchedulingPolicy};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

/// Atomic scheduler statistics for lock-free updates
///
/// # Performance
/// - Cache-line aligned to prevent false sharing
/// - All operations use relaxed ordering for maximum performance
/// - Read-only snapshot requires no synchronization
#[repr(C, align(64))]
pub struct AtomicSchedulerStats {
    total_scheduled: AtomicU64,
    context_switches: AtomicU64,
    preemptions: AtomicU64,
    active_processes: AtomicUsize,
    // These don't change frequently, can use parking_lot::RwLock for snapshots
    policy: parking_lot::RwLock<SchedulingPolicy>,
    quantum: parking_lot::RwLock<Duration>,
}

impl AtomicSchedulerStats {
    /// Create new atomic stats
    #[inline]
    pub fn new(policy: SchedulingPolicy, quantum: Duration) -> Self {
        Self {
            total_scheduled: AtomicU64::new(0),
            context_switches: AtomicU64::new(0),
            preemptions: AtomicU64::new(0),
            active_processes: AtomicUsize::new(0),
            policy: parking_lot::RwLock::new(policy),
            quantum: parking_lot::RwLock::new(quantum),
        }
    }

    /// Increment total scheduled (lock-free)
    ///
    /// # Performance
    /// Hot path - called on every schedule operation
    #[inline(always)]
    pub fn inc_scheduled(&self) {
        self.total_scheduled.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment context switches (lock-free)
    ///
    /// # Performance
    /// Hot path - called on every context switch
    #[inline(always)]
    pub fn inc_context_switches(&self) {
        self.context_switches.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment preemptions (lock-free)
    ///
    /// # Performance
    /// Hot path - called on every preemption
    #[inline(always)]
    pub fn inc_preemptions(&self) {
        self.preemptions.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment active processes (lock-free)
    ///
    /// # Performance
    /// Hot path - called when adding processes to scheduler
    #[inline(always)]
    pub fn inc_active(&self) {
        self.active_processes.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active processes (lock-free)
    ///
    /// # Performance
    /// Hot path - called when removing processes from scheduler
    #[inline(always)]
    pub fn dec_active(&self) {
        self.active_processes.fetch_sub(1, Ordering::Relaxed);
    }

    /// Set active process count (used for initialization)
    #[inline]
    pub fn set_active(&self, count: usize) {
        self.active_processes.store(count, Ordering::Relaxed);
    }

    /// Update policy (infrequent operation)
    #[inline]
    pub fn set_policy(&self, policy: SchedulingPolicy) {
        *self.policy.write() = policy;
    }

    /// Update quantum (infrequent operation)
    #[inline]
    pub fn set_quantum(&self, quantum: Duration) {
        *self.quantum.write() = quantum;
    }

    /// Get snapshot of current stats (minimal locking)
    ///
    /// # Note
    /// Counter values may not be perfectly consistent with each other due to concurrent updates,
    /// but each individual value is accurate. This is acceptable for monitoring.
    #[inline]
    pub fn snapshot(&self) -> SchedulerStats {
        SchedulerStats {
            total_scheduled: self.total_scheduled.load(Ordering::Relaxed),
            context_switches: self.context_switches.load(Ordering::Relaxed),
            preemptions: self.preemptions.load(Ordering::Relaxed),
            active_processes: self.active_processes.load(Ordering::Relaxed),
            policy: *self.policy.read(),
            quantum_micros: self.quantum.read().as_micros() as u64,
        }
    }
}
