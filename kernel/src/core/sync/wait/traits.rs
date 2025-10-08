/*!
 * Synchronization Traits
 *
 * Core abstractions for wait/notify patterns with zero-cost design.
 *
 * # Design: Trait-Based Abstraction for Implementations
 *
 * While WaitQueue itself uses enum dispatch for performance, this trait
 * allows for custom wait strategies and testing. All methods are designed
 * to inline well and have minimal overhead.
 */

use std::time::Duration;

/// Result of a wake operation
///
/// Compact representation (single usize) for efficient returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeResult {
    /// Successfully woke N waiters (N >= 1)
    Woken(usize),
    /// No waiters were waiting
    NoWaiters,
}

impl WakeResult {
    /// Check if any waiters were woken
    #[inline(always)]
    pub fn is_woken(&self) -> bool {
        matches!(self, WakeResult::Woken(_))
    }

    /// Get number of woken waiters (0 if none)
    #[inline(always)]
    pub fn count(&self) -> usize {
        match self {
            WakeResult::Woken(n) => *n,
            WakeResult::NoWaiters => 0,
        }
    }
}

/// Strategy for waiting on a condition
///
/// Implementations must be:
/// - **Thread-safe**: Safe to call from multiple threads
/// - **Efficient**: Minimize CPU usage while waiting
/// - **Fair**: Avoid starvation of waiters (when possible)
///
/// # Type Parameters
///
/// - `K`: Key type for waiting (e.g., u64 for sequence numbers, (Pid, Fd) for multi-key)
///
/// # Implementation Notes
///
/// All methods should be marked `#[inline]` or `#[inline(always)]` where appropriate
/// to enable cross-crate optimization.
pub trait WaitStrategy<K>: Send + Sync
where
    K: Eq + std::hash::Hash + Copy + Send + Sync,
{
    /// Wait for a specific key with optional timeout
    ///
    /// Returns `true` if woken by notify, `false` if timeout occurred.
    ///
    /// # Performance
    ///
    /// Hot path - must be efficient for both short and long waits
    fn wait(&self, key: K, timeout: Option<Duration>) -> bool;

    /// Wake one waiter waiting on the specified key
    ///
    /// Returns the number of waiters woken (0 or 1)
    fn wake_one(&self, key: K) -> WakeResult;

    /// Wake all waiters waiting on the specified key
    ///
    /// Returns the number of waiters woken
    fn wake_all(&self, key: K) -> WakeResult;

    /// Try to register a waiter without blocking
    ///
    /// Returns `true` if successfully registered
    fn try_register(&self, key: K) -> bool {
        // Default implementation: always allow registration
        let _ = key;
        true
    }

    /// Unregister a waiter without being woken
    ///
    /// This should be called if waiting is cancelled
    fn unregister(&self, key: K) {
        let _ = key;
        // Default: no-op (wake will handle missing waiters)
    }

    /// Get approximate count of waiters for a key (for diagnostics)
    fn waiter_count(&self, key: K) -> usize {
        let _ = key;
        0 // Default: unknown
    }

    /// Get strategy name for debugging
    fn name(&self) -> &'static str;
}
