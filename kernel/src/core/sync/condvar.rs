/*!
 * Condvar-Based Wait Strategy
 *
 * Cross-platform fallback using parking_lot::Condvar for reliability
 */

use super::traits::{WaitStrategy, WakeResult};
use dashmap::DashMap;
use parking_lot::{Condvar, Mutex};
use std::sync::Arc;
use std::time::Duration;
use ahash::RandomState;

/// Waiter state for a specific key
struct WaiterState {
    condvar: Condvar,
    mutex: Mutex<bool>,
    count: Mutex<usize>,
}

impl WaiterState {
    fn new() -> Self {
        Self {
            condvar: Condvar::new(),
            mutex: Mutex::new(false),
            count: Mutex::new(0),
        }
    }
}

/// Condvar-based wait strategy
///
/// # Performance
///
/// - Slightly more overhead than futex
/// - Works on all platforms
/// - Reliable and well-tested
pub struct CondvarWait<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    waiters: Arc<DashMap<K, Arc<WaiterState>, RandomState>>,
}

impl<K> CondvarWait<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    /// Create a new condvar-based wait strategy
    pub fn new() -> Self {
        Self {
            waiters: Arc::new(DashMap::with_hasher(RandomState::new())),
        }
    }

    /// Get or create waiter state for a key
    fn get_waiter(&self, key: K) -> Arc<WaiterState> {
        self.waiters
            .entry(key)
            .or_insert_with(|| Arc::new(WaiterState::new()))
            .clone()
    }
}

impl<K> Default for CondvarWait<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K> WaitStrategy<K> for CondvarWait<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    fn wait(&self, key: K, timeout: Option<Duration>) -> bool {
        let state = self.get_waiter(key);

        // Increment waiter count
        *state.count.lock() += 1;

        // Wait on condvar
        let mut notified = state.mutex.lock();

        let result = if let Some(timeout) = timeout {
            state.condvar.wait_for(&mut notified, timeout).timed_out()
        } else {
            state.condvar.wait(&mut notified);
            false
        };

        // Decrement waiter count
        let mut count = state.count.lock();
        *count -= 1;
        let is_last = *count == 0;
        drop(count);

        // Clean up if last waiter
        if is_last {
            self.waiters.remove(&key);
        }

        // Return true if woken, false if timeout
        !result
    }

    fn wake_one(&self, key: K) -> WakeResult {
        if let Some(state) = self.waiters.get(&key) {
            let count = *state.count.lock();
            if count > 0 {
                state.condvar.notify_one();
                WakeResult::Woken(1)
            } else {
                WakeResult::NoWaiters
            }
        } else {
            WakeResult::NoWaiters
        }
    }

    fn wake_all(&self, key: K) -> WakeResult {
        if let Some(state) = self.waiters.get(&key) {
            let count = *state.count.lock();
            if count > 0 {
                state.condvar.notify_all();
                WakeResult::Woken(count)
            } else {
                WakeResult::NoWaiters
            }
        } else {
            WakeResult::NoWaiters
        }
    }

    fn waiter_count(&self, key: K) -> usize {
        self.waiters
            .get(&key)
            .map(|s| *s.count.lock())
            .unwrap_or(0)
    }

    fn name(&self) -> &'static str {
        "condvar"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Instant;

    #[test]
    fn test_condvar_wake_one() {
        let cv = Arc::new(CondvarWait::<u64>::new());
        let cv_clone = cv.clone();

        let handle = thread::spawn(move || {
            cv_clone.wait(42, Some(Duration::from_secs(1)))
        });

        // Give thread time to wait
        thread::sleep(Duration::from_millis(50));

        let result = cv.wake_one(42);
        assert!(matches!(result, WakeResult::Woken(1)));

        assert!(handle.join().unwrap());
    }

    #[test]
    fn test_condvar_timeout() {
        let cv = CondvarWait::<u64>::new();
        let start = Instant::now();
        let result = cv.wait(99, Some(Duration::from_millis(50)));
        let elapsed = start.elapsed();

        assert!(!result); // Should timeout
        assert!(elapsed >= Duration::from_millis(50));
    }

    #[test]
    fn test_condvar_wake_all() {
        let cv = Arc::new(CondvarWait::<u64>::new());

        let handles: Vec<_> = (0..3)
            .map(|_| {
                let cv_clone = cv.clone();
                thread::spawn(move || {
                    cv_clone.wait(100, Some(Duration::from_secs(1)))
                })
            })
            .collect();

        // Give threads time to wait
        thread::sleep(Duration::from_millis(100));

        let result = cv.wake_all(100);
        assert!(matches!(result, WakeResult::Woken(3)));

        for handle in handles {
            assert!(handle.join().unwrap());
        }
    }
}
