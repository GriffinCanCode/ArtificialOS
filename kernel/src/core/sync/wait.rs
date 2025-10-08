/*!
 * Wait Queue
 *
 * High-level abstraction for waiting on keys (like sequence numbers).
 * Automatically selects optimal strategy based on platform and configuration.
 */

use super::condvar::CondvarWait;
use super::config::{StrategyType, SyncConfig};
use super::futex::FutexWait;
use super::spinwait::SpinWait;
use super::traits::{WaitStrategy, WakeResult};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// Result type for wait operations
pub type WaitResult<T> = Result<T, WaitError>;

/// Wait operation errors
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum WaitError {
    #[error("Wait operation timed out")]
    Timeout,

    #[error("Wait was cancelled")]
    Cancelled,

    #[error("Invalid key")]
    InvalidKey,
}

/// Generic wait queue for any key type
///
/// # Performance
///
/// - Strategy selected at creation time (zero overhead)
/// - Lock-free fast paths where possible
/// - Platform-optimized implementations
///
/// # Type Parameters
///
/// - `K`: Key type (e.g., u64 for sequence numbers, (Pid, u64) for multi-keyed waits)
///
/// # Examples
///
/// ```
/// use ai_os_kernel::core::sync::{WaitQueue, SyncConfig};
/// use std::time::Duration;
///
/// // Create with auto-selected strategy
/// let queue = WaitQueue::<u64>::new(SyncConfig::default());
///
/// // Wait for sequence 42 with timeout
/// let result = queue.wait(42, Some(Duration::from_secs(1)));
///
/// // Wake waiters from another thread
/// queue.wake_one(42);
/// ```
pub struct WaitQueue<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    strategy: Arc<dyn WaitStrategy<K>>,
}

impl<K> WaitQueue<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    /// Create a new wait queue with the specified configuration
    pub fn new(config: SyncConfig) -> Self {
        let strategy_type = config.select_strategy();

        let strategy: Arc<dyn WaitStrategy<K>> = match strategy_type {
            StrategyType::Futex => Arc::new(FutexWait::new()),
            StrategyType::Condvar => Arc::new(CondvarWait::new()),
            StrategyType::SpinWait => {
                Arc::new(SpinWait::new(config.spin_duration, config.max_spins))
            }
            StrategyType::Auto => {
                // Should have been resolved by select_strategy
                #[cfg(target_os = "linux")]
                {
                    Arc::new(FutexWait::new())
                }
                #[cfg(not(target_os = "linux"))]
                {
                    Arc::new(CondvarWait::new())
                }
            }
        };

        Self { strategy }
    }

    /// Create with default configuration (auto-selects best strategy)
    pub fn with_defaults() -> Self {
        Self::new(SyncConfig::default())
    }

    /// Create optimized for low-latency waits
    pub fn low_latency() -> Self {
        Self::new(SyncConfig::low_latency())
    }

    /// Create optimized for long waits
    pub fn long_wait() -> Self {
        Self::new(SyncConfig::long_wait())
    }

    /// Wait for a specific key with optional timeout
    ///
    /// Returns `Ok(())` if woken by notify, `Err(WaitError::Timeout)` if timeout occurred.
    ///
    /// # Performance
    ///
    /// Hot path - optimized for both short and long waits
    pub fn wait(&self, key: K, timeout: Option<Duration>) -> WaitResult<()> {
        let woken = self.strategy.wait(key, timeout);
        if woken {
            Ok(())
        } else {
            Err(WaitError::Timeout)
        }
    }

    /// Wait for a key with a predicate check
    ///
    /// The predicate is checked before waiting and after each wake.
    /// This prevents lost wakeups when the condition changes before we start waiting.
    ///
    /// # Performance
    ///
    /// The predicate should be fast (< 1µs ideally)
    pub fn wait_while<F>(&self, key: K, timeout: Option<Duration>, mut predicate: F) -> WaitResult<()>
    where
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();

        loop {
            // Check predicate first
            if !predicate() {
                return Ok(());
            }

            // Check timeout
            if let Some(timeout) = timeout {
                if start.elapsed() >= timeout {
                    return Err(WaitError::Timeout);
                }
            }

            // Calculate remaining timeout
            let remaining = timeout.map(|t| t.saturating_sub(start.elapsed()));

            // Wait for notification
            self.wait(key, remaining)?;

            // Loop will check predicate again
        }
    }

    /// Wake one waiter waiting on the specified key
    ///
    /// Returns the number of waiters woken (0 or 1)
    pub fn wake_one(&self, key: K) -> WakeResult {
        self.strategy.wake_one(key)
    }

    /// Wake all waiters waiting on the specified key
    ///
    /// Returns the number of waiters woken
    pub fn wake_all(&self, key: K) -> WakeResult {
        self.strategy.wake_all(key)
    }

    /// Get approximate count of waiters for a key (for diagnostics)
    pub fn waiter_count(&self, key: K) -> usize {
        self.strategy.waiter_count(key)
    }

    /// Get the name of the active strategy
    pub fn strategy_name(&self) -> &'static str {
        self.strategy.name()
    }

    /// Async-compatible wait using tokio::spawn_blocking
    ///
    /// This bridges the gap between async code and our sync WaitQueue.
    /// Uses futex on Linux (via spawn_blocking) without blocking the tokio runtime.
    ///
    /// # Performance
    ///
    /// - Spawns blocking task (minimal overhead, ~1µs)
    /// - Underlying wait uses futex (Linux) for zero CPU usage
    /// - No busy-waiting or polling
    #[cfg(feature = "tokio")]
    pub async fn wait_async(&self, key: K, timeout: Option<Duration>) -> WaitResult<()> {
        let strategy = self.strategy.clone();
        tokio::task::spawn_blocking(move || {
            let woken = strategy.wait(key, timeout);
            if woken {
                Ok(())
            } else {
                Err(WaitError::Timeout)
            }
        })
        .await
        .map_err(|e| WaitError::Cancelled)?
    }

    /// Async-compatible wait_while using tokio::spawn_blocking
    ///
    /// # Performance
    ///
    /// Predicate is evaluated in the blocking task (not on tokio runtime)
    #[cfg(feature = "tokio")]
    pub async fn wait_while_async<F>(
        &self,
        key: K,
        timeout: Option<Duration>,
        predicate: F,
    ) -> WaitResult<()>
    where
        F: FnMut() -> bool + Send + 'static,
    {
        let strategy = self.strategy.clone();
        tokio::task::spawn_blocking(move || {
            let start = std::time::Instant::now();
            let mut pred = predicate;

            loop {
                // Check predicate first
                if !pred() {
                    return Ok(());
                }

                // Check timeout
                if let Some(timeout) = timeout {
                    if start.elapsed() >= timeout {
                        return Err(WaitError::Timeout);
                    }
                }

                // Calculate remaining timeout
                let remaining = timeout.map(|t| t.saturating_sub(start.elapsed()));

                // Wait for notification
                if !strategy.wait(key, remaining) {
                    return Err(WaitError::Timeout);
                }
            }
        })
        .await
        .map_err(|_| WaitError::Cancelled)?
    }
}

impl<K> Clone for WaitQueue<K>
where
    K: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            strategy: self.strategy.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Instant;

    #[test]
    fn test_wait_queue_basic() {
        let queue = Arc::new(WaitQueue::<u64>::with_defaults());
        let queue_clone = queue.clone();

        let handle = thread::spawn(move || queue_clone.wait(42, Some(Duration::from_secs(1))));

        thread::sleep(Duration::from_millis(50));
        queue.wake_one(42);

        assert!(handle.join().unwrap().is_ok());
    }

    #[test]
    fn test_wait_queue_timeout() {
        let queue = WaitQueue::<u64>::with_defaults();
        let start = Instant::now();
        let result = queue.wait(99, Some(Duration::from_millis(50)));

        assert!(matches!(result, Err(WaitError::Timeout)));
        assert!(start.elapsed() >= Duration::from_millis(50));
    }

    #[test]
    fn test_wait_while_predicate() {
        let queue = Arc::new(WaitQueue::<u64>::with_defaults());
        let value = Arc::new(parking_lot::Mutex::new(0));

        let queue_clone = queue.clone();
        let value_clone = value.clone();

        let handle = thread::spawn(move || {
            queue_clone.wait_while(100, Some(Duration::from_secs(1)), || {
                *value_clone.lock() < 5
            })
        });

        thread::sleep(Duration::from_millis(50));

        // Update value and wake
        *value.lock() = 10;
        queue.wake_one(100);

        assert!(handle.join().unwrap().is_ok());
    }

    #[test]
    fn test_wake_all() {
        let queue = Arc::new(WaitQueue::<u64>::with_defaults());

        let handles: Vec<_> = (0..3)
            .map(|_| {
                let queue_clone = queue.clone();
                thread::spawn(move || {
                    queue_clone.wait(200, Some(Duration::from_secs(1)))
                })
            })
            .collect();

        thread::sleep(Duration::from_millis(100));

        let result = queue.wake_all(200);
        assert!(matches!(result, WakeResult::Woken(_)));

        for handle in handles {
            assert!(handle.join().unwrap().is_ok());
        }
    }

    #[test]
    fn test_low_latency_config() {
        let queue = WaitQueue::<u64>::low_latency();
        assert!(queue.strategy_name().len() > 0);
    }
}
