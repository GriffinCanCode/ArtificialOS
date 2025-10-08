/*!
 * Generic Timeout Execution
 *
 * Unified timeout handling for all blocking syscall operations.
 * Eliminates need for per-resource-type timeout wrappers.
 */

use crate::core::guard::TimeoutPolicy;
use crate::monitoring::TimeoutObserver;
use std::sync::Arc;
use std::time::Instant;

/// Generic timeout executor for blocking operations
///
/// ## Design Philosophy
///
/// Rather than creating separate timeout wrappers for each resource type
/// (TimeoutPipeOps, TimeoutQueueOps, TimeoutFileOps, etc.), we provide
/// a single, composable timeout mechanism that works with any operation.
///
/// ## Benefits
///
/// 1. **Single Implementation**: One retry loop to maintain
/// 2. **Zero Duplication**: Same logic works for pipes, queues, files, sockets
/// 3. **Minimal Memory**: No per-resource-type Arc wrappers
/// 4. **Type Safe**: Still enforces timeout policies
/// 5. **Observable**: Automatic timeout event emission
/// 6. **Composable**: Works with any Fn() -> Result<T, E>
///
/// ## Example
///
/// ```rust
/// // Before (old pattern):
/// let timeout_ops = TimeoutPipeOps::new(manager);
/// let result = timeout_ops.read_timeout(pipe_id, pid, size, timeout)?;
///
/// // After (new pattern):
/// let result = self.timeout_executor.execute_with_retry(
///     || pipe_manager.read(pipe_id, pid, size),
///     |e| matches!(e, PipeError::WouldBlock(_)),
///     timeout,
///     "pipe_read"
/// )?;
/// ```
#[derive(Clone)]
pub struct TimeoutExecutor {
    observer: Option<Arc<TimeoutObserver>>,
    enabled: bool,
}

impl TimeoutExecutor {
    /// Create new timeout executor
    pub fn new(observer: Option<Arc<TimeoutObserver>>) -> Self {
        Self {
            observer,
            enabled: true,
        }
    }

    /// Create disabled timeout executor (for testing)
    pub fn disabled() -> Self {
        Self {
            observer: None,
            enabled: false,
        }
    }

    /// Execute operation with retry-based timeout
    ///
    /// Repeatedly attempts the operation until it succeeds or timeout expires.
    /// Uses brief yields between retries to prevent busy-waiting.
    ///
    /// # Arguments
    ///
    /// * `operation` - Function that performs the operation (returns WouldBlock on retry)
    /// * `is_would_block` - Predicate to identify retryable errors
    /// * `timeout` - Timeout policy to enforce
    /// * `resource_type` - Resource type for observability (e.g., "pipe_read")
    ///
    /// # Returns
    ///
    /// - `Ok(T)` if operation succeeds
    /// - `Err(TimeoutError::Timeout)` if timeout expires
    /// - `Err(TimeoutError::Operation(E))` if non-retryable error occurs
    pub fn execute_with_retry<T, E>(
        &self,
        mut operation: impl FnMut() -> Result<T, E>,
        is_would_block: impl Fn(&E) -> bool,
        timeout: TimeoutPolicy,
        resource_type: &'static str,
    ) -> Result<T, TimeoutError<E>> {
        if !self.enabled {
            // Fast path: if timeouts disabled, execute once
            return operation().map_err(TimeoutError::Operation);
        }

        let start = Instant::now();
        let mut iteration = 0u64;

        loop {
            match operation() {
                Ok(result) => {
                    // Success - emit success metric if we retried
                    if iteration > 0 && self.observer.is_some() {
                        // Could add success-after-retry metrics here
                    }
                    return Ok(result);
                }
                Err(e) if is_would_block(&e) => {
                    iteration += 1;

                    // Check if timeout expired
                    if timeout.is_expired(start) {
                        let elapsed = start.elapsed();

                        // Emit observability event
                        if let Some(ref observer) = self.observer {
                            Self::emit_timeout_event(
                                observer,
                                resource_type,
                                elapsed,
                                timeout.duration(),
                            );
                        }

                        return Err(TimeoutError::Timeout {
                            resource_type,
                            category: timeout.category(),
                            elapsed_ms: elapsed.as_millis() as u64,
                            timeout_ms: timeout.duration().map(|d| d.as_millis() as u64),
                        });
                    }

                    // Brief yield to prevent busy-waiting
                    // This allows other threads to make progress
                    std::thread::yield_now();
                }
                Err(e) => {
                    // Non-retryable error - fail immediately
                    return Err(TimeoutError::Operation(e));
                }
            }
        }
    }

    /// Execute operation with simple duration-based timeout
    ///
    /// Simpler variant that just tries once with a timeout check.
    /// Useful for operations that handle blocking internally.
    pub fn execute_with_deadline<T, E>(
        &self,
        operation: impl FnOnce() -> Result<T, E>,
        timeout: TimeoutPolicy,
        resource_type: &'static str,
    ) -> Result<T, TimeoutError<E>> {
        if !self.enabled {
            return operation().map_err(TimeoutError::Operation);
        }

        let start = Instant::now();
        let result = operation();

        // Check if we exceeded timeout
        if timeout.is_expired(start) {
            let elapsed = start.elapsed();

            if let Some(ref observer) = self.observer {
                Self::emit_timeout_event(
                    observer,
                    resource_type,
                    elapsed,
                    timeout.duration(),
                );
            }

            return Err(TimeoutError::Timeout {
                resource_type,
                category: timeout.category(),
                elapsed_ms: elapsed.as_millis() as u64,
                timeout_ms: timeout.duration().map(|d| d.as_millis() as u64),
            });
        }

        result.map_err(TimeoutError::Operation)
    }

    /// Helper to emit timeout observability event
    fn emit_timeout_event(
        observer: &TimeoutObserver,
        resource_type: &'static str,
        elapsed: std::time::Duration,
        timeout: Option<std::time::Duration>,
    ) {
        // Categorize by resource type
        match resource_type {
            s if s.starts_with("pipe_") || s.starts_with("queue_") => {
                if let Some(timeout_dur) = timeout {
                    observer.emit_ipc_timeout(resource_type, 0, 0, elapsed, timeout_dur);
                }
            }
            s if s.starts_with("file_") || s.starts_with("fd_") => {
                if let Some(timeout_dur) = timeout {
                    observer.emit_io_timeout(resource_type, 0, 0, elapsed, timeout_dur);
                }
            }
            s if s.starts_with("socket_") || s.starts_with("net_") => {
                if let Some(timeout_dur) = timeout {
                    observer.emit_io_timeout(resource_type, 0, 0, elapsed, timeout_dur);
                }
            }
            _ => {
                // Generic timeout event
                if let Some(timeout_dur) = timeout {
                    observer.emit_lock_timeout(resource_type, None, elapsed, timeout_dur);
                }
            }
        }
    }

    /// Check if timeouts are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable timeout enforcement
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable timeout enforcement (for testing)
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

/// Timeout execution error
#[derive(Debug, thiserror::Error)]
pub enum TimeoutError<E> {
    /// Operation timed out
    #[error("Operation timed out: {resource_type} ({category}) after {elapsed_ms}ms (timeout: {}ms)", timeout_ms.map(|t| t.to_string()).unwrap_or_else(|| "none".to_string()))]
    Timeout {
        resource_type: &'static str,
        category: &'static str,
        elapsed_ms: u64,
        timeout_ms: Option<u64>,
    },

    /// Operation failed with non-timeout error
    #[error("Operation failed: {0}")]
    Operation(#[source] E),
}

impl<E> TimeoutError<E> {
    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }

    /// Check if this is an operation error
    pub fn is_operation_error(&self) -> bool {
        matches!(self, Self::Operation(_))
    }

    /// Convert to operation error (panics if timeout)
    pub fn into_operation_error(self) -> E {
        match self {
            Self::Operation(e) => e,
            Self::Timeout { .. } => panic!("Called into_operation_error on timeout"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[derive(Debug, PartialEq)]
    enum TestError {
        WouldBlock,
        Fatal(String),
    }

    #[test]
    fn test_successful_operation() {
        let executor = TimeoutExecutor::disabled();

        let result = executor.execute_with_retry(
            || Ok::<i32, TestError>(42),
            |_| false,
            TimeoutPolicy::None,
            "test",
        );

        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_retry_then_success() {
        let executor = TimeoutExecutor::disabled();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = executor.execute_with_retry(
            || {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count < 3 {
                    Err(TestError::WouldBlock)
                } else {
                    Ok(42)
                }
            },
            |e| matches!(e, TestError::WouldBlock),
            TimeoutPolicy::None,
            "test",
        );

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 4); // 3 retries + 1 success
    }

    #[test]
    fn test_fatal_error_no_retry() {
        let executor = TimeoutExecutor::disabled();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result: Result<(), _> = executor.execute_with_retry(
            || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Err(TestError::Fatal("boom".to_string()))
            },
            |e| matches!(e, TestError::WouldBlock),
            TimeoutPolicy::None,
            "test",
        );

        assert!(matches!(result, Err(TimeoutError::Operation(TestError::Fatal(_)))));
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only called once
    }

    #[test]
    fn test_timeout_enforcement() {
        let executor = TimeoutExecutor::new(None);

        let result = executor.execute_with_retry(
            || Err::<i32, TestError>(TestError::WouldBlock),
            |e| matches!(e, TestError::WouldBlock),
            TimeoutPolicy::Lock(Duration::from_millis(50)),
            "test_lock",
        );

        assert!(result.is_err());
        assert!(matches!(result, Err(TimeoutError::Timeout { .. })));
    }

    #[test]
    fn test_disabled_executor() {
        let executor = TimeoutExecutor::disabled();

        // Should execute immediately without retry
        let result = executor.execute_with_retry(
            || Err::<i32, TestError>(TestError::WouldBlock),
            |e| matches!(e, TestError::WouldBlock),
            TimeoutPolicy::Lock(Duration::from_millis(50)),
            "test",
        );

        // Returns the error immediately (no timeout)
        assert!(matches!(result, Err(TimeoutError::Operation(TestError::WouldBlock))));
    }
}
