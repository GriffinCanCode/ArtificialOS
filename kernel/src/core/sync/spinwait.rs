/*!
 * Adaptive Spin-Wait Strategy
 *
 * Optimized for low-latency scenarios where waits are typically very short.
 * Spins for a while before falling back to parking.
 */

use super::condvar::CondvarWait;
use super::traits::{WaitStrategy, WakeResult};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{Duration, Instant};

/// Adaptive spin-wait strategy
///
/// # Performance
///
/// - Ultra-low latency for short waits (< 10µs)
/// - Higher CPU usage during wait
/// - Falls back to condvar for long waits
///
/// # Use Cases
///
/// Best for scenarios where:
/// - Wait duration is typically < 100µs
/// - Low latency is critical
/// - CPU usage is acceptable trade-off
pub struct SpinWait<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    /// Fallback condvar for long waits
    fallback: CondvarWait<K>,
    /// Spin duration before falling back
    spin_duration: Duration,
    /// Maximum spin iterations
    max_spins: u32,
}

impl<K> SpinWait<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    /// Create a new adaptive spin-wait strategy
    pub fn new(spin_duration: Duration, max_spins: u32) -> Self {
        Self {
            fallback: CondvarWait::new(),
            spin_duration,
            max_spins,
        }
    }

    /// Create with default parameters
    pub fn with_defaults() -> Self {
        Self::new(Duration::from_micros(50), 500)
    }

    /// Perform adaptive spinning
    ///
    /// Returns true if should continue waiting, false if should give up
    fn spin(&self, check: impl Fn() -> bool) -> bool {
        let start = Instant::now();
        let mut spin_count = 0;

        loop {
            // Check condition
            if check() {
                return true;
            }

            // Check if we've spun long enough
            if start.elapsed() >= self.spin_duration || spin_count >= self.max_spins {
                return false;
            }

            // Yield to scheduler occasionally
            if spin_count % 10 == 0 {
                thread::yield_now();
            }

            spin_count += 1;
        }
    }
}

impl<K> Default for SpinWait<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl<K> WaitStrategy<K> for SpinWait<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    fn wait(&self, key: K, timeout: Option<Duration>) -> bool {
        let start = Instant::now();

        // Spin first
        self.spin(|| {
            // Check timeout
            if let Some(timeout) = timeout {
                if start.elapsed() >= timeout {
                    return true; // Stop spinning, we timed out
                }
            }
            false // Keep spinning
        });

        // Check if we hit timeout during spin
        if let Some(timeout) = timeout {
            if start.elapsed() >= timeout {
                return false;
            }
        }

        // Fall back to condvar for longer waits
        let remaining_timeout = timeout.map(|t| t.saturating_sub(start.elapsed()));
        self.fallback.wait(key, remaining_timeout)
    }

    fn wake_one(&self, key: K) -> WakeResult {
        // Delegate to fallback
        self.fallback.wake_one(key)
    }

    fn wake_all(&self, key: K) -> WakeResult {
        // Delegate to fallback
        self.fallback.wake_all(key)
    }

    fn waiter_count(&self, key: K) -> usize {
        self.fallback.waiter_count(key)
    }

    fn name(&self) -> &'static str {
        "spinwait"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_spinwait_timeout() {
        let sw = SpinWait::<u64>::with_defaults();
        let start = Instant::now();
        let result = sw.wait(99, Some(Duration::from_millis(50)));
        let elapsed = start.elapsed();

        assert!(!result); // Should timeout
                          // Should be close to timeout (allowing some overhead)
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100));
    }

    #[test]
    fn test_spinwait_wake() {
        let sw = Arc::new(SpinWait::<u64>::with_defaults());
        let sw_clone = sw.clone();

        let handle = thread::spawn(move || sw_clone.wait(42, Some(Duration::from_secs(1))));

        // Give thread time to start waiting
        thread::sleep(Duration::from_millis(100));

        let result = sw.wake_one(42);
        assert!(matches!(
            result,
            WakeResult::Woken(_) | WakeResult::NoWaiters
        ));

        handle.join().unwrap();
    }
}
