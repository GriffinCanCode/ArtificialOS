/*!
 * Process Syscalls
 * Process management operations
 */

use crate::core::types::{Pid, Priority};
use serde::{Deserialize, Serialize};

/// Process operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "syscall")]
#[non_exhaustive]
#[allow(dead_code)]
pub enum ProcessSyscall {
    /// Spawn a new process
    SpawnProcess {
        /// Command to execute
        command: String,
        /// Command arguments
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
    },

    /// Kill/terminate process
    KillProcess {
        /// Process ID to kill
        target_pid: Pid,
    },

    /// Get process information
    GetProcessInfo {
        /// Process ID to query
        target_pid: Pid,
    },

    /// Get list of all processes
    GetProcessList,

    /// Set process priority
    SetProcessPriority {
        /// Process ID to modify
        target_pid: Pid,
        /// New priority level
        priority: Priority,
    },

    /// Get process state
    GetProcessState {
        /// Process ID to query
        target_pid: Pid,
    },

    /// Get process statistics
    GetProcessStats {
        /// Process ID to query
        target_pid: Pid,
    },

    /// Wait for process to complete
    WaitProcess {
        /// Process ID to wait for
        target_pid: Pid,
        /// Optional timeout in milliseconds
        timeout_ms: Option<u64>,
    },
}
