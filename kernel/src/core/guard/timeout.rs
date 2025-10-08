/*!
 * Unified Timeout Infrastructure
 *
 * Type-safe, hierarchical timeout system for all blocking operations.
 *
 * ## Design Principles
 *
 * 1. **Hierarchical Policies**: Different defaults for I/O, locks, IPC
 * 2. **Type-Safe**: Compile-time enforcement of timeout handling
 * 3. **Observable**: Automatic timeout event tracking
 * 4. **Composable**: Works with all guard types
 * 5. **Zero-Cost**: Monomorphized away when not used
 *
 * ## Timeout Classes
 *
 * - **Lock**: 1-100ms (short-lived critical sections)
 * - **IPC**: 1s-30s (inter-process communication)
 * - **IO**: 5s-300s (file/network operations)
 * - **Task**: 10s-3600s (async task completion)
 * - **Custom**: User-defined
 *
 * ## Example
 *
 * ```ignore
 * // Automatic timeout based on resource type
 * let guard = lock_guard.lock_timeout(TimeoutPolicy::default_for_lock())?;
 *
 * // Custom timeout
 * let guard = ipc_guard.wait_timeout(Duration::from_secs(5))?;
 *
 * // Hierarchical config
 * let config = TimeoutConfig::new()
 *     .with_lock_timeout(Duration::from_millis(50))
 *     .with_ipc_timeout(Duration::from_secs(10));
 * ```
 */

use super::{GuardError, GuardResult};
use std::time::{Duration, Instant};

/// Timeout policy for blocking operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutPolicy {
    /// No timeout (infinite wait) - use sparingly!
    None,

    /// Lock acquisition timeout (1-100ms)
    Lock(Duration),

    /// IPC operation timeout (1s-30s)
    Ipc(Duration),

    /// I/O operation timeout (5s-300s)
    Io(Duration),

    /// Async task timeout (10s-3600s)
    Task(Duration),

    /// Custom timeout
    Custom(Duration),
}

impl TimeoutPolicy {
    /// Default lock timeout: 50ms
    pub const fn default_lock() -> Self {
        Self::Lock(Duration::from_millis(50))
    }

    /// Default IPC timeout: 10s
    pub const fn default_ipc() -> Self {
        Self::Ipc(Duration::from_secs(10))
    }

    /// Default I/O timeout: 30s
    pub const fn default_io() -> Self {
        Self::Io(Duration::from_secs(30))
    }

    /// Default task timeout: 60s
    pub const fn default_task() -> Self {
        Self::Task(Duration::from_secs(60))
    }

    /// Get the duration for this policy
    pub fn duration(&self) -> Option<Duration> {
        match self {
            Self::None => None,
            Self::Lock(d) | Self::Ipc(d) | Self::Io(d) | Self::Task(d) | Self::Custom(d) => Some(*d),
        }
    }

    /// Check if this timeout has expired
    pub fn is_expired(&self, start: Instant) -> bool {
        match self.duration() {
            None => false,
            Some(d) => start.elapsed() >= d,
        }
    }

    /// Get remaining time before timeout
    pub fn remaining(&self, start: Instant) -> Option<Duration> {
        match self.duration() {
            None => None,
            Some(d) => {
                let elapsed = start.elapsed();
                if elapsed >= d {
                    Some(Duration::ZERO)
                } else {
                    Some(d - elapsed)
                }
            }
        }
    }

    /// Get timeout category as string
    pub fn category(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Lock(_) => "lock",
            Self::Ipc(_) => "ipc",
            Self::Io(_) => "io",
            Self::Task(_) => "task",
            Self::Custom(_) => "custom",
        }
    }
}

impl Default for TimeoutPolicy {
    fn default() -> Self {
        Self::None
    }
}

/// Timeout configuration for a system/process
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    lock_timeout: Duration,
    ipc_timeout: Duration,
    io_timeout: Duration,
    task_timeout: Duration,
    enabled: bool,
}

impl TimeoutConfig {
    /// Create new config with defaults
    pub fn new() -> Self {
        Self {
            lock_timeout: Duration::from_millis(50),
            ipc_timeout: Duration::from_secs(10),
            io_timeout: Duration::from_secs(30),
            task_timeout: Duration::from_secs(60),
            enabled: true,
        }
    }

    /// Create config with all timeouts disabled (for testing)
    pub fn disabled() -> Self {
        Self {
            lock_timeout: Duration::from_secs(3600),
            ipc_timeout: Duration::from_secs(3600),
            io_timeout: Duration::from_secs(3600),
            task_timeout: Duration::from_secs(3600),
            enabled: false,
        }
    }

    /// Set lock timeout
    pub fn with_lock_timeout(mut self, timeout: Duration) -> Self {
        self.lock_timeout = timeout;
        self
    }

    /// Set IPC timeout
    pub fn with_ipc_timeout(mut self, timeout: Duration) -> Self {
        self.ipc_timeout = timeout;
        self
    }

    /// Set I/O timeout
    pub fn with_io_timeout(mut self, timeout: Duration) -> Self {
        self.io_timeout = timeout;
        self
    }

    /// Set task timeout
    pub fn with_task_timeout(mut self, timeout: Duration) -> Self {
        self.task_timeout = timeout;
        self
    }

    /// Get timeout for lock operations
    pub fn lock_timeout(&self) -> TimeoutPolicy {
        if self.enabled {
            TimeoutPolicy::Lock(self.lock_timeout)
        } else {
            TimeoutPolicy::None
        }
    }

    /// Get timeout for IPC operations
    pub fn ipc_timeout(&self) -> TimeoutPolicy {
        if self.enabled {
            TimeoutPolicy::Ipc(self.ipc_timeout)
        } else {
            TimeoutPolicy::None
        }
    }

    /// Get timeout for I/O operations
    pub fn io_timeout(&self) -> TimeoutPolicy {
        if self.enabled {
            TimeoutPolicy::Io(self.io_timeout)
        } else {
            TimeoutPolicy::None
        }
    }

    /// Get timeout for task operations
    pub fn task_timeout(&self) -> TimeoutPolicy {
        if self.enabled {
            TimeoutPolicy::Task(self.task_timeout)
        } else {
            TimeoutPolicy::None
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Timeout context for tracking timeout state
#[derive(Debug, Clone)]
pub struct TimeoutContext {
    policy: TimeoutPolicy,
    start: Instant,
    resource_type: &'static str,
}

impl TimeoutContext {
    /// Create new timeout context
    pub fn new(policy: TimeoutPolicy, resource_type: &'static str) -> Self {
        Self {
            policy,
            start: Instant::now(),
            resource_type,
        }
    }

    /// Check if timeout has expired
    pub fn is_expired(&self) -> bool {
        self.policy.is_expired(self.start)
    }

    /// Get remaining duration
    pub fn remaining(&self) -> Option<Duration> {
        self.policy.remaining(self.start)
    }

    /// Get elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get the policy
    pub fn policy(&self) -> TimeoutPolicy {
        self.policy
    }

    /// Get resource type
    pub fn resource_type(&self) -> &'static str {
        self.resource_type
    }

    /// Create timeout error if expired
    pub fn timeout_error(&self) -> GuardError {
        GuardError::Timeout {
            resource_type: self.resource_type,
            category: self.policy.category(),
            elapsed_ms: self.elapsed().as_millis() as u64,
            timeout_ms: self.policy.duration().map(|d| d.as_millis() as u64),
        }
    }
}

/// Extension to GuardError for timeouts
impl GuardError {
    /// Create a timeout error
    pub fn timeout(resource_type: &'static str, category: &'static str, elapsed: Duration, timeout: Option<Duration>) -> Self {
        Self::Timeout {
            resource_type,
            category,
            elapsed_ms: elapsed.as_millis() as u64,
            timeout_ms: timeout.map(|d| d.as_millis() as u64),
        }
    }
}

/// Trait for guards that support timeout-based acquisition
pub trait TimeoutAcquire: Sized {
    /// Acquire resource with timeout
    fn acquire_timeout(self, timeout: TimeoutPolicy) -> GuardResult<Self>;
}

/// Trait for guards that support timeout-based waiting
pub trait TimeoutWait {
    /// Wait for condition with timeout
    fn wait_timeout(&mut self, timeout: TimeoutPolicy) -> GuardResult<()>;
}

/// Extension trait for converting TimeoutPolicy to Duration for existing APIs
pub trait TimeoutPolicyExt {
    /// Convert to Option<Duration> for use with WaitQueue and similar APIs
    fn to_duration_opt(&self) -> Option<std::time::Duration>;
}

impl TimeoutPolicyExt for TimeoutPolicy {
    fn to_duration_opt(&self) -> Option<std::time::Duration> {
        self.duration()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_timeout_policy_defaults() {
        assert_eq!(TimeoutPolicy::default_lock(), TimeoutPolicy::Lock(Duration::from_millis(50)));
        assert_eq!(TimeoutPolicy::default_ipc(), TimeoutPolicy::Ipc(Duration::from_secs(10)));
        assert_eq!(TimeoutPolicy::default_io(), TimeoutPolicy::Io(Duration::from_secs(30)));
        assert_eq!(TimeoutPolicy::default_task(), TimeoutPolicy::Task(Duration::from_secs(60)));
    }

    #[test]
    fn test_timeout_policy_expiration() {
        let policy = TimeoutPolicy::Lock(Duration::from_millis(10));
        let start = Instant::now();

        assert!(!policy.is_expired(start));

        thread::sleep(Duration::from_millis(15));
        assert!(policy.is_expired(start));
    }

    #[test]
    fn test_timeout_policy_remaining() {
        let policy = TimeoutPolicy::Lock(Duration::from_millis(100));
        let start = Instant::now();

        let remaining = policy.remaining(start).unwrap();
        assert!(remaining <= Duration::from_millis(100));
        assert!(remaining > Duration::from_millis(50));

        thread::sleep(Duration::from_millis(110));
        let remaining = policy.remaining(start).unwrap();
        assert_eq!(remaining, Duration::ZERO);
    }

    #[test]
    fn test_timeout_config() {
        let config = TimeoutConfig::new()
            .with_lock_timeout(Duration::from_millis(100))
            .with_ipc_timeout(Duration::from_secs(5));

        assert_eq!(config.lock_timeout(), TimeoutPolicy::Lock(Duration::from_millis(100)));
        assert_eq!(config.ipc_timeout(), TimeoutPolicy::Ipc(Duration::from_secs(5)));
    }

    #[test]
    fn test_timeout_config_disabled() {
        let config = TimeoutConfig::disabled();
        assert_eq!(config.lock_timeout(), TimeoutPolicy::None);
        assert_eq!(config.ipc_timeout(), TimeoutPolicy::None);
    }

    #[test]
    fn test_timeout_context() {
        let policy = TimeoutPolicy::Lock(Duration::from_millis(50));
        let ctx = TimeoutContext::new(policy, "test_lock");

        assert!(!ctx.is_expired());
        assert!(ctx.elapsed() < Duration::from_millis(10));

        let remaining = ctx.remaining().unwrap();
        assert!(remaining > Duration::from_millis(30));
    }
}
