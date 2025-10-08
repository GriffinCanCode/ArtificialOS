/*!
 * Scheduler Syscalls
 * CPU scheduling and process management
 */

use crate::core::types::Pid;
use serde::{Deserialize, Serialize};

/// Scheduler operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "syscall")]
#[non_exhaustive]
#[allow(dead_code)]
pub enum SchedulerSyscall {
    /// Schedule next process
    ScheduleNext,

    /// Yield current process
    YieldProcess,

    /// Get currently scheduled process
    GetCurrentScheduled,

    /// Get global scheduler statistics
    GetSchedulerStats,

    /// Set scheduling policy
    SetSchedulingPolicy {
        /// Policy: "round_robin", "priority", or "fair"
        policy: String,
    },

    /// Get current scheduling policy
    GetSchedulingPolicy,

    /// Set time quantum for scheduler
    SetTimeQuantum {
        /// Time quantum in microseconds
        quantum_micros: u64,
    },

    /// Get current time quantum
    GetTimeQuantum,

    /// Get scheduler stats for a process
    GetProcessSchedulerStats {
        /// Process ID to query
        target_pid: Pid,
    },

    /// Get scheduler stats for all processes
    GetAllProcessSchedulerStats,

    /// Boost process priority
    BoostPriority {
        /// Process ID to boost
        target_pid: Pid,
    },

    /// Lower process priority
    LowerPriority {
        /// Process ID to lower
        target_pid: Pid,
    },
}
