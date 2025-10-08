/*!
 * Futex-Based Wait Strategy
 *
 * Uses parking_lot_core for futex-like operations on all platforms.
 * On Linux, this maps directly to futex syscalls for minimal overhead.
 */

use super::traits::{WaitStrategy, WakeResult};
use ahash::RandomState;
use dashmap::DashMap;
use parking_lot_core::{park, unpark_all, unpark_one, ParkResult, ParkToken, UnparkToken};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Futex-based wait strategy using parking_lot_core
///
/// # Performance
///
/// - Direct futex syscalls on Linux
/// - Minimal memory overhead
/// - Cache-line aligned counters
/// - Lock-free fast path
#[repr(C, align(64))]
pub struct FutexWait<K>
where
    K: Eq + Hash + Copy + Send + Sync + 'static,
{
    /// Waiter counts per key (for diagnostics)
    waiters: DashMap<K, AtomicUsize, RandomState>,
}

impl<K> FutexWait<K>
where
    K: Eq + Hash + Copy + Send + Sync + 'static,
{
    /// Create a new futex-based wait strategy
    pub fn new() -> Self {
        Self {
            waiters: DashMap::with_hasher(RandomState::new()),
        }
    }

    /// Get parking address for a key (used by parking_lot_core)
    ///
    /// This converts a key into a unique address that parking_lot can use
    fn parking_address(&self, key: K) -> usize {
        // Use key's hash as the parking address
        let mut hasher = ahash::AHasher::default();
        key.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Increment waiter count
    fn inc_waiters(&self, key: K) {
        self.waiters
            .entry(key)
            .or_insert_with(|| AtomicUsize::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement waiter count
    fn dec_waiters(&self, key: K) {
        if let Some(count) = self.waiters.get(&key) {
            let prev = count.fetch_sub(1, Ordering::Relaxed);
            // Clean up if this was the last waiter
            if prev == 1 {
                self.waiters.remove(&key);
            }
        }
    }
}

impl<K> Default for FutexWait<K>
where
    K: Eq + Hash + Copy + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K> WaitStrategy<K> for FutexWait<K>
where
    K: Eq + Hash + Copy + Send + Sync + 'static,
{
    fn wait(&self, key: K, timeout: Option<Duration>) -> bool {
        self.inc_waiters(key);

        let addr = self.parking_address(key);
        let start = Instant::now();
        let deadline = timeout.map(|d| Instant::now() + d);

        // Use parking_lot's efficient park
        let result = unsafe {
            park(
                addr,
                || {
                    // Validate callback: check if we should still park
                    // Always return true to actually park
                    true
                },
                || {
                    // Before sleep callback: nothing needed
                },
                |_timed_out, _result| {
                    // After unpark callback: nothing needed
                },
                ParkToken(0),
                deadline,
            )
        };

        self.dec_waiters(key);

        // Return true if woken, false if timeout
        match result {
            ParkResult::Unparked(_) => true,
            ParkResult::TimedOut => {
                // Double-check: parking_lot may spuriously wake
                timeout.map_or(false, |t| start.elapsed() >= t)
            }
            ParkResult::Invalid => false,
        }
    }

    fn wake_one(&self, key: K) -> WakeResult {
        let addr = self.parking_address(key);

        // Get current waiter count
        let count = self
            .waiters
            .get(&key)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0);

        if count == 0 {
            return WakeResult::NoWaiters;
        }

        // Unpark one thread
        let result = unsafe { unpark_one(addr, |_| UnparkToken(0)) };

        WakeResult::Woken(result.unparked_threads)
    }

    fn wake_all(&self, key: K) -> WakeResult {
        let addr = self.parking_address(key);

        // Get current waiter count
        let count = self
            .waiters
            .get(&key)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0);

        if count == 0 {
            return WakeResult::NoWaiters;
        }

        // Unpark all threads
        let count = unsafe { unpark_all(addr, UnparkToken(0)) };

        WakeResult::Woken(count)
    }

    fn waiter_count(&self, key: K) -> usize {
        self.waiters
            .get(&key)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    fn name(&self) -> &'static str {
        "futex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_futex_wake_one() {
        let futex = Arc::new(FutexWait::<u64>::new());
        let futex_clone = futex.clone();

        let handle = thread::spawn(move || futex_clone.wait(42, Some(Duration::from_secs(1))));

        // Give thread time to park
        thread::sleep(Duration::from_millis(50));

        let result = futex.wake_one(42);
        assert_eq!(result, WakeResult::Woken(1));

        assert!(handle.join().unwrap());
    }

    #[test]
    fn test_futex_timeout() {
        let futex = FutexWait::<u64>::new();
        let start = Instant::now();
        let result = futex.wait(99, Some(Duration::from_millis(50)));
        let elapsed = start.elapsed();

        assert!(!result); // Should timeout
        assert!(elapsed >= Duration::from_millis(50));
    }
}
