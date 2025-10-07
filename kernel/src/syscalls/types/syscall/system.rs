/*!
 * System Syscalls
 * System information, time, memory, and signals
 */

use crate::core::types::Pid;
use serde::{Deserialize, Serialize};

/// System operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "syscall")]
#[non_exhaustive]
pub enum SystemSyscall {
    /// Get system information
    GetSystemInfo,

    /// Get current system time
    GetCurrentTime,

    /// Get environment variable
    GetEnvironmentVar {
        /// Variable name
        key: String,
    },

    /// Set environment variable
    SetEnvironmentVar {
        /// Variable name
        key: String,
        /// Variable value
        value: String,
    },

    /// Sleep for duration
    Sleep {
        /// Duration in milliseconds
        duration_ms: u64,
    },

    /// Get system uptime
    GetUptime,

    /// Get global memory statistics
    GetMemoryStats,

    /// Get process memory statistics
    GetProcessMemoryStats {
        /// Process ID to query
        target_pid: Pid,
    },

    /// Trigger garbage collection
    TriggerGC {
        /// Optional target process ID (None = global GC)
        target_pid: Option<u32>,
    },

    /// Send signal to process
    SendSignal {
        /// Target process ID
        target_pid: Pid,
        /// Signal number
        signal: u32,
    },

    /// Register signal handler
    RegisterSignalHandler {
        /// Signal number
        signal: u32,
        /// Handler ID
        handler_id: u64,
    },

    /// Block signal
    BlockSignal {
        /// Signal number
        signal: u32,
    },

    /// Unblock signal
    UnblockSignal {
        /// Signal number
        signal: u32,
    },

    /// Get pending signals
    GetPendingSignals,

    /// Get signal statistics
    GetSignalStats,

    /// Wait for signal
    WaitForSignal {
        /// Signals to wait for
        signals: Vec<u32>,
        /// Optional timeout in milliseconds
        timeout_ms: Option<u64>,
    },

    /// Get signal state
    GetSignalState {
        /// Optional target PID (None = current process)
        target_pid: Option<Pid>,
    },
}
