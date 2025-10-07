/*!
 * Scheduler Syscall Traits
 * Interface definitions for scheduler operations
 */

use crate::core::types::Pid;
use crate::syscalls::types::SyscallResult;

/// Core scheduler control operations
pub trait SchedulerControl: Send + Sync {
    /// Schedule the next process to run
    fn schedule_next(&self, pid: Pid) -> SyscallResult;

    /// Yield current process voluntarily
    fn yield_process(&self, pid: Pid) -> SyscallResult;

    /// Get currently scheduled process
    fn get_current_scheduled(&self, pid: Pid) -> SyscallResult;
}

/// Scheduler policy management
pub trait SchedulerPolicy: Send + Sync {
    /// Set scheduling policy
    fn set_scheduling_policy(&self, pid: Pid, policy: &str) -> SyscallResult;

    /// Get current scheduling policy
    fn get_scheduling_policy(&self, pid: Pid) -> SyscallResult;

    /// Set time quantum for round-robin scheduling
    fn set_time_quantum(&self, pid: Pid, quantum_micros: u64) -> SyscallResult;

    /// Get current time quantum
    fn get_time_quantum(&self, pid: Pid) -> SyscallResult;
}

/// Scheduler statistics and monitoring
pub trait SchedulerStats: Send + Sync {
    /// Get global scheduler statistics
    fn get_scheduler_stats(&self, pid: Pid) -> SyscallResult;

    /// Get CPU statistics for specific process
    fn get_process_scheduler_stats(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Get CPU statistics for all processes
    fn get_all_process_scheduler_stats(&self, pid: Pid) -> SyscallResult;
}

/// Priority management operations
pub trait PriorityControl: Send + Sync {
    /// Boost process priority
    fn boost_priority(&self, pid: Pid, target_pid: Pid) -> SyscallResult;

    /// Lower process priority
    fn lower_priority(&self, pid: Pid, target_pid: Pid) -> SyscallResult;
}

/// Combined scheduler syscall interface
pub trait SchedulerSyscalls:
    SchedulerControl + SchedulerPolicy + SchedulerStats + PriorityControl + Send + Sync
{
}

// Blanket implementation for any type that implements all component traits
impl<T> SchedulerSyscalls for T where
    T: SchedulerControl + SchedulerPolicy + SchedulerStats + PriorityControl + Send + Sync
{
}
