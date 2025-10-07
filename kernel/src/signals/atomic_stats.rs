/*!
 * Lock-Free Signal Statistics
 * Uses atomic counters for zero-contention stats tracking in hot paths
 */

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use crate::signals::types::SignalStats;

/// Atomic signal statistics for lock-free updates
///
/// # Performance
/// - Cache-line aligned to prevent false sharing
/// - All operations use relaxed ordering for maximum performance
/// - Read-only snapshot requires no synchronization
#[repr(C, align(64))]
pub struct AtomicSignalStats {
    total_signals_sent: AtomicU64,
    total_signals_delivered: AtomicU64,
    total_signals_blocked: AtomicU64,
    total_signals_queued: AtomicU64,
    pending_signals: AtomicUsize,
    handlers_registered: AtomicUsize,
}

impl AtomicSignalStats {
    /// Create new atomic stats
    #[inline]
    pub const fn new() -> Self {
        Self {
            total_signals_sent: AtomicU64::new(0),
            total_signals_delivered: AtomicU64::new(0),
            total_signals_blocked: AtomicU64::new(0),
            total_signals_queued: AtomicU64::new(0),
            pending_signals: AtomicUsize::new(0),
            handlers_registered: AtomicUsize::new(0),
        }
    }

    /// Increment signals sent (lock-free)
    ///
    /// # Performance
    /// Hot path - called on every signal send
    #[inline(always)]
    pub fn inc_signals_sent(&self) {
        self.total_signals_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment signals delivered (lock-free)
    ///
    /// # Performance
    /// Hot path - called on every signal delivery
    #[inline(always)]
    pub fn inc_signals_delivered(&self) {
        self.total_signals_delivered.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment signals blocked (lock-free)
    ///
    /// # Performance
    /// Hot path - called when signals are blocked
    #[inline(always)]
    pub fn inc_signals_blocked(&self) {
        self.total_signals_blocked.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment signals queued (lock-free)
    ///
    /// # Performance
    /// Hot path - called when signals are queued
    #[inline(always)]
    pub fn inc_signals_queued(&self) {
        self.total_signals_queued.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment pending signals (lock-free)
    ///
    /// # Performance
    /// Hot path - called on queue/dequeue operations
    #[inline(always)]
    pub fn inc_pending(&self) {
        self.pending_signals.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement pending signals (lock-free)
    ///
    /// # Performance
    /// Hot path - called on queue/dequeue operations
    #[inline(always)]
    pub fn dec_pending(&self, count: usize) {
        self.pending_signals.fetch_sub(count, Ordering::Relaxed);
    }

    /// Increment handlers registered (lock-free)
    #[inline(always)]
    pub fn inc_handlers(&self) {
        self.handlers_registered.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement handlers registered (lock-free)
    #[inline(always)]
    pub fn dec_handlers(&self, count: usize) {
        self.handlers_registered.fetch_sub(count, Ordering::Relaxed);
    }

    /// Get snapshot of current stats (no locks required)
    ///
    /// # Note
    /// Values may not be perfectly consistent with each other due to concurrent updates,
    /// but each individual value is accurate. This is acceptable for monitoring.
    #[inline]
    pub fn snapshot(&self) -> SignalStats {
        SignalStats {
            total_signals_sent: self.total_signals_sent.load(Ordering::Relaxed),
            total_signals_delivered: self.total_signals_delivered.load(Ordering::Relaxed),
            total_signals_blocked: self.total_signals_blocked.load(Ordering::Relaxed),
            total_signals_queued: self.total_signals_queued.load(Ordering::Relaxed),
            pending_signals: self.pending_signals.load(Ordering::Relaxed),
            handlers_registered: self.handlers_registered.load(Ordering::Relaxed),
        }
    }
}

impl Default for AtomicSignalStats {
    fn default() -> Self {
        Self::new()
    }
}
