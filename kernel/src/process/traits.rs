/*!
 * Process Traits
 * Process management abstractions
 */

use super::types::*;
use crate::core::types::{Pid, Priority};

/// Process lifecycle management
pub trait ProcessLifecycle: Send + Sync {
    /// Create a new process
    fn create(&self, name: String, priority: Priority) -> ProcessResult<Pid>;

    /// Create a process with OS execution
    fn create_with_command(
        &self,
        name: String,
        priority: Priority,
        config: ExecutionConfig,
    ) -> ProcessResult<Pid>;

    /// Terminate a process
    fn terminate(&self, pid: Pid) -> ProcessResult<()>;

    /// Check if a process exists
    fn exists(&self, pid: Pid) -> bool;

    /// Get process information
    fn get_process(&self, pid: Pid) -> Option<ProcessInfo>;

    /// List all processes
    fn list_processes(&self) -> Vec<ProcessInfo>;
}

/// Process execution interface
pub trait ProcessExecutor: Send + Sync {
    /// Spawn an OS process
    fn spawn(&self, pid: Pid, name: String, config: ExecutionConfig) -> ProcessResult<u32>;

    /// Kill a running OS process
    fn kill(&self, pid: Pid) -> ProcessResult<()>;

    /// Wait for a process to complete
    fn wait(&self, pid: Pid) -> ProcessResult<i32>;

    /// Check if a process is running
    fn is_running(&self, pid: Pid) -> bool;

    /// Get OS PID for internal PID
    fn get_os_pid(&self, pid: Pid) -> Option<u32>;

    /// Get count of running processes
    fn count(&self) -> usize;

    /// Cleanup zombie processes
    fn cleanup(&self);
}

/// CPU scheduler interface
pub trait ProcessScheduler: Send + Sync {
    /// Add a process to the scheduler
    fn add(&self, pid: Pid, priority: Priority);

    /// Remove a process from the scheduler
    fn remove(&self, pid: Pid) -> bool;

    /// Schedule the next process to run
    fn schedule(&self) -> Option<Pid>;

    /// Yield the current process voluntarily
    fn yield_process(&self) -> Option<Pid>;

    /// Get currently scheduled process
    fn current(&self) -> Option<Pid>;

    /// Change scheduling policy
    fn set_policy(&self, policy: SchedulingPolicy);

    /// Get current scheduling policy
    fn policy(&self) -> SchedulingPolicy;

    /// Get scheduler statistics
    fn stats(&self) -> SchedulerStats;

    /// Get number of processes in scheduler
    fn len(&self) -> usize;

    /// Check if scheduler is empty
    fn is_empty(&self) -> bool;
}

/// Process statistics provider
pub trait ProcessStatistics: Send + Sync {
    /// Get CPU statistics for a specific process
    fn process_stats(&self, pid: Pid) -> Option<ProcessStats>;

    /// Get all process CPU statistics
    fn all_process_stats(&self) -> Vec<ProcessStats>;

    /// Get process execution statistics
    fn execution_stats(&self, pid: Pid) -> Option<ExecutionStats>;

    /// Get process resource usage
    fn resource_usage(&self, pid: Pid) -> Option<ProcessResources>;
}

/// Priority management
pub trait PriorityManager: Send + Sync {
    /// Set process priority
    fn set_priority(&self, pid: Pid, priority: Priority) -> ProcessResult<()>;

    /// Get process priority
    fn get_priority(&self, pid: Pid) -> Option<Priority>;

    /// Increase priority (boost)
    fn boost_priority(&self, pid: Pid) -> ProcessResult<()>;

    /// Decrease priority
    fn lower_priority(&self, pid: Pid) -> ProcessResult<()>;
}

/// Process state management
pub trait ProcessStateManager: Send + Sync {
    /// Get process state
    fn get_state(&self, pid: Pid) -> Option<ProcessState>;

    /// Set process state
    fn set_state(&self, pid: Pid, state: ProcessState) -> ProcessResult<()>;

    /// Transition process state with validation
    fn transition_state(&self, pid: Pid, from: ProcessState, to: ProcessState)
        -> ProcessResult<()>;
}

/// Combined process manager trait
pub trait ProcessManager:
    ProcessLifecycle + ProcessStatistics + ProcessStateManager + Clone + Send + Sync
{
    /// Get child process count for a PID
    fn get_child_count(&self, pid: Pid) -> u32;

    /// Check if process has OS execution
    fn has_os_process(&self, pid: Pid) -> bool;
}
