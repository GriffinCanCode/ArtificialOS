/*!
 * Syscall Timeout Configuration
 *
 * Global timeout policies for all blocking syscalls.
 */

use crate::core::guard::TimeoutPolicy;
use std::time::Duration;

/// Timeout configuration for syscall operations
#[derive(Debug, Clone)]
pub struct SyscallTimeoutConfig {
    /// Timeout for pipe read operations (default: 10s)
    pub pipe_read: TimeoutPolicy,

    /// Timeout for pipe write operations (default: 10s)
    pub pipe_write: TimeoutPolicy,

    /// Timeout for queue receive operations (default: 10s)
    pub queue_receive: TimeoutPolicy,

    /// Timeout for file I/O operations (default: 30s)
    pub file_io: TimeoutPolicy,

    /// Timeout for network operations (default: 60s)
    pub network: TimeoutPolicy,

    /// Enable timeout enforcement globally
    pub enabled: bool,
}

impl SyscallTimeoutConfig {
    /// Create default timeout configuration
    pub fn new() -> Self {
        Self {
            pipe_read: TimeoutPolicy::Ipc(Duration::from_secs(10)),
            pipe_write: TimeoutPolicy::Ipc(Duration::from_secs(10)),
            queue_receive: TimeoutPolicy::Ipc(Duration::from_secs(10)),
            file_io: TimeoutPolicy::Io(Duration::from_secs(30)),
            network: TimeoutPolicy::Io(Duration::from_secs(60)),
            enabled: true,
        }
    }

    /// Create configuration with all timeouts disabled (testing only)
    pub fn disabled() -> Self {
        Self {
            pipe_read: TimeoutPolicy::None,
            pipe_write: TimeoutPolicy::None,
            queue_receive: TimeoutPolicy::None,
            file_io: TimeoutPolicy::None,
            network: TimeoutPolicy::None,
            enabled: false,
        }
    }

    /// Create aggressive timeout configuration for development
    pub fn aggressive() -> Self {
        Self {
            pipe_read: TimeoutPolicy::Ipc(Duration::from_secs(2)),
            pipe_write: TimeoutPolicy::Ipc(Duration::from_secs(2)),
            queue_receive: TimeoutPolicy::Ipc(Duration::from_secs(2)),
            file_io: TimeoutPolicy::Io(Duration::from_secs(5)),
            network: TimeoutPolicy::Io(Duration::from_secs(10)),
            enabled: true,
        }
    }

    /// Create relaxed timeout configuration for slow environments
    pub fn relaxed() -> Self {
        Self {
            pipe_read: TimeoutPolicy::Ipc(Duration::from_secs(60)),
            pipe_write: TimeoutPolicy::Ipc(Duration::from_secs(60)),
            queue_receive: TimeoutPolicy::Ipc(Duration::from_secs(60)),
            file_io: TimeoutPolicy::Io(Duration::from_secs(300)),
            network: TimeoutPolicy::Io(Duration::from_secs(600)),
            enabled: true,
        }
    }
}

impl Default for SyscallTimeoutConfig {
    fn default() -> Self {
        Self::new()
    }
}
