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
}
