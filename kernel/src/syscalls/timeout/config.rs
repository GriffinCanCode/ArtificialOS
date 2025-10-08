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

    /// Timeout for file sync operations (default: 60s, sync can be slow)
    pub file_sync: TimeoutPolicy,

    /// Timeout for network operations (default: 60s)
    pub network: TimeoutPolicy,

    /// Timeout for process wait operations (default: 300s)
    pub process_wait: TimeoutPolicy,

    /// Enable timeout enforcement globally
    pub enabled: bool,
}

impl SyscallTimeoutConfig {
    /// Create default timeout configuration
    pub fn new() -> Self {
        use crate::core::limits::*;
        Self {
            pipe_read: TimeoutPolicy::Ipc(STANDARD_IPC_TIMEOUT),
            pipe_write: TimeoutPolicy::Ipc(STANDARD_IPC_TIMEOUT),
            queue_receive: TimeoutPolicy::Ipc(STANDARD_IPC_TIMEOUT),
            file_io: TimeoutPolicy::Io(STANDARD_FILE_IO_TIMEOUT),
            file_sync: TimeoutPolicy::Io(STANDARD_FSYNC_TIMEOUT),
            network: TimeoutPolicy::Io(STANDARD_NETWORK_TIMEOUT),
            process_wait: TimeoutPolicy::Io(STANDARD_PROCESS_WAIT_TIMEOUT),
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
            file_sync: TimeoutPolicy::None,
            network: TimeoutPolicy::None,
            process_wait: TimeoutPolicy::None,
            enabled: false,
        }
    }

    /// Create aggressive timeout configuration for development
    pub fn aggressive() -> Self {
        use crate::core::limits::*;
        Self {
            pipe_read: TimeoutPolicy::Ipc(RESTRICTED_IPC_TIMEOUT),
            pipe_write: TimeoutPolicy::Ipc(RESTRICTED_IPC_TIMEOUT),
            queue_receive: TimeoutPolicy::Ipc(RESTRICTED_IPC_TIMEOUT),
            file_io: TimeoutPolicy::Io(Duration::from_secs(5).into()),
            file_sync: TimeoutPolicy::Io(Duration::from_secs(10).into()),
            network: TimeoutPolicy::Io(Duration::from_secs(10).into()),
            process_wait: TimeoutPolicy::Io(Duration::from_secs(30).into()),
            enabled: true,
        }
    }

    /// Create relaxed timeout configuration for slow environments
    pub fn relaxed() -> Self {
        use crate::core::limits::*;
        Self {
            pipe_read: TimeoutPolicy::Ipc(RELAXED_IPC_TIMEOUT),
            pipe_write: TimeoutPolicy::Ipc(RELAXED_IPC_TIMEOUT),
            queue_receive: TimeoutPolicy::Ipc(RELAXED_IPC_TIMEOUT),
            file_io: TimeoutPolicy::Io(Duration::from_secs(300).into()),
            file_sync: TimeoutPolicy::Io(Duration::from_secs(600).into()),
            network: TimeoutPolicy::Io(Duration::from_secs(600).into()),
            process_wait: TimeoutPolicy::Io(Duration::from_secs(1800).into()),
            enabled: true,
        }
    }
}

impl Default for SyscallTimeoutConfig {
    fn default() -> Self {
        Self::new()
    }
}
