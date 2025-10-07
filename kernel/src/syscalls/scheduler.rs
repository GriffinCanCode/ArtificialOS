/*!
 * Scheduler Syscalls
 * Syscall interface for scheduler operations
 */

use crate::core::types::Pid;
use crate::scheduler::{
    PriorityControl, SchedulerControl, SchedulerPolicy, SchedulerStats, SchedulerSyscalls,
};
use crate::security::Capability;
use log::{error, info, warn};

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

impl SyscallExecutor {
    /// Schedule next process (internal implementation)
    pub(super) fn schedule_next(&self, pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        info!("Schedule next syscall requested by PID {}", pid);

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.schedule_next() {
            Some(next_pid) => {
                info!("Scheduler selected next process: PID {}", next_pid);
                let data = next_pid.to_le_bytes().to_vec();
                SyscallResult::success_with_data(data)
            }
            None => {
                info!("No processes available to schedule");
                SyscallResult::success()
            }
        }
    }

    /// Yield current process (internal implementation)
    pub(super) fn yield_process(&self, pid: Pid) -> SyscallResult {
        info!("Process {} yielding CPU", pid);

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.yield_current() {
            Some(next_pid) => {
                info!("Process {} yielded, next process: {}", pid, next_pid);
                let data = next_pid.to_le_bytes().to_vec();
                SyscallResult::success_with_data(data)
            }
            None => {
                info!("Process {} yielded, no next process", pid);
                SyscallResult::success()
            }
        }
    }

    /// Get currently scheduled process (internal implementation)
    pub(super) fn get_current_scheduled(&self, pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        info!("Get current scheduled process requested by PID {}", pid);

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.current_scheduled() {
            Some(current_pid) => {
                let data = current_pid.to_le_bytes().to_vec();
                SyscallResult::success_with_data(data)
            }
            None => {
                SyscallResult::success()
            }
        }
    }

    /// Set scheduling policy (internal implementation)
    pub(super) fn set_scheduling_policy(&self, pid: Pid, policy_str: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SpawnProcess)
        {
            return SyscallResult::permission_denied(
                "Missing SpawnProcess capability (required for scheduler control)",
            );
        }

        let syscall_policy = match crate::scheduler::Policy::from_str(policy_str) {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid scheduling policy requested: {}", policy_str);
                return SyscallResult::error(e);
            }
        };

        let policy = match syscall_policy {
            crate::scheduler::Policy::RoundRobin => crate::process::types::SchedulingPolicy::RoundRobin,
            crate::scheduler::Policy::Priority => crate::process::types::SchedulingPolicy::Priority,
            crate::scheduler::Policy::Fair => crate::process::types::SchedulingPolicy::Fair,
        };

        info!("PID {} changing scheduling policy to {:?}", pid, policy);

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        if process_manager.set_scheduling_policy(policy) {
            info!("Scheduling policy changed to {:?} successfully", policy);
            SyscallResult::success()
        } else {
            error!("Failed to change scheduling policy");
            SyscallResult::error("Scheduler not available or policy change failed")
        }
    }

    /// Get current scheduling policy (internal implementation)
    pub(super) fn get_scheduling_policy(&self, pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_scheduling_policy() {
            Some(policy) => {
                let policy_name = match policy {
                    crate::process::types::SchedulingPolicy::RoundRobin => "round_robin",
                    crate::process::types::SchedulingPolicy::Priority => "priority",
                    crate::process::types::SchedulingPolicy::Fair => "fair",
                };

                match serde_json::to_vec(&policy_name) {
                    Ok(data) => {
                        info!("PID {} retrieved scheduling policy: {:?}", pid, policy);
                        SyscallResult::success_with_data(data)
                    }
                    Err(e) => {
                        error!("Failed to serialize policy: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            None => SyscallResult::error("Scheduler not available"),
        }
    }

    /// Set time quantum (internal implementation)
    pub(super) fn set_time_quantum(&self, pid: Pid, quantum_micros: u64) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SpawnProcess)
        {
            return SyscallResult::permission_denied(
                "Missing SpawnProcess capability (required for scheduler control)",
            );
        }

        if let Err(e) = crate::scheduler::TimeQuantum::new(quantum_micros) {
            error!("Invalid time quantum requested: {} microseconds", quantum_micros);
            return SyscallResult::error(e);
        }

        info!(
            "PID {} setting time quantum to {} microseconds",
            pid, quantum_micros
        );

        warn!("SetTimeQuantum not fully implemented - quantum is set at scheduler creation");
        SyscallResult::error("Dynamic quantum adjustment not yet supported")
    }

    /// Get current time quantum (internal implementation)
    pub(super) fn get_time_quantum(&self, pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_scheduler_stats() {
            Some(stats) => {
                let data = stats.quantum_micros.to_le_bytes().to_vec();
                info!(
                    "PID {} retrieved time quantum: {} microseconds",
                    pid, stats.quantum_micros
                );
                SyscallResult::success_with_data(data)
            }
            None => SyscallResult::error("Scheduler not available"),
        }
    }

    /// Get global scheduler statistics (internal implementation)
    pub(super) fn get_scheduler_stats(&self, pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        info!("PID {} requested scheduler statistics", pid);

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_scheduler_stats() {
            Some(stats) => match serde_json::to_vec(&stats) {
                Ok(data) => SyscallResult::success_with_data(data),
                Err(e) => {
                    error!("Failed to serialize scheduler stats: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            None => SyscallResult::error("Scheduler not available"),
        }
    }

    /// Get process scheduler statistics (internal implementation)
    pub(super) fn get_process_scheduler_stats(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_process_stats(target_pid) {
            Some(stats) => match serde_json::to_vec(&stats) {
                Ok(data) => {
                    info!(
                        "PID {} retrieved scheduler stats for PID {}: CPU time: {}μs",
                        pid, target_pid, stats.cpu_time_micros
                    );
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize process scheduler stats: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            None => SyscallResult::error(format!(
                "No scheduler stats available for process {}",
                target_pid
            )),
        }
    }

    /// Get all process scheduler statistics (internal implementation)
    pub(super) fn get_all_process_scheduler_stats(&self, pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        let stats = process_manager.get_all_process_stats();
        match serde_json::to_vec(&stats) {
            Ok(data) => {
                info!(
                    "PID {} retrieved scheduler stats for {} processes",
                    pid,
                    stats.len()
                );
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Failed to serialize all process scheduler stats: {}", e);
                SyscallResult::error("Serialization failed")
            }
        }
    }

    /// Boost process priority (internal implementation)
    pub(super) fn boost_priority(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SpawnProcess)
        {
            return SyscallResult::permission_denied(
                "Missing SpawnProcess capability (required for priority control)",
            );
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_process(target_pid) {
            Some(process_info) => {
                let current_priority = process_info.priority;

                match crate::scheduler::apply_priority_op(
                    current_priority,
                    crate::scheduler::PriorityOp::Boost,
                ) {
                    Ok(new_priority) => {
                        info!(
                            "PID {} boosting priority of PID {} from {} to {}",
                            pid, target_pid, current_priority, new_priority
                        );

                        warn!("Priority boost not fully implemented - requires mutable process access and scheduler update");
                        SyscallResult::error(
                            "Priority management not yet fully integrated with scheduler",
                        )
                    }
                    Err(e) => {
                        info!(
                            "PID {} attempted to boost priority of PID {}: {}",
                            pid, target_pid, e
                        );
                        SyscallResult::error(e)
                    }
                }
            }
            None => SyscallResult::error(format!("Process {} not found", target_pid)),
        }
    }

    /// Lower process priority (internal implementation)
    pub(super) fn lower_priority(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SpawnProcess)
        {
            return SyscallResult::permission_denied(
                "Missing SpawnProcess capability (required for priority control)",
            );
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_process(target_pid) {
            Some(process_info) => {
                let current_priority = process_info.priority;

                match crate::scheduler::apply_priority_op(
                    current_priority,
                    crate::scheduler::PriorityOp::Lower,
                ) {
                    Ok(new_priority) => {
                        info!(
                            "PID {} lowering priority of PID {} from {} to {}",
                            pid, target_pid, current_priority, new_priority
                        );

                        warn!("Priority lowering not fully implemented - requires mutable process access and scheduler update");
                        SyscallResult::error(
                            "Priority management not yet fully integrated with scheduler",
                        )
                    }
                    Err(e) => {
                        info!(
                            "PID {} attempted to lower priority of PID {}: {}",
                            pid, target_pid, e
                        );
                        SyscallResult::error(e)
                    }
                }
            }
            None => SyscallResult::error(format!("Process {} not found", target_pid)),
        }
    }
}

// Implement trait interfaces by delegating to internal methods
impl SchedulerControl for SyscallExecutor {
    fn schedule_next(&self, pid: Pid) -> SyscallResult {
        self.schedule_next(pid)
    }

    fn yield_process(&self, pid: Pid) -> SyscallResult {
        self.yield_process(pid)
    }

    fn get_current_scheduled(&self, pid: Pid) -> SyscallResult {
        self.get_current_scheduled(pid)
    }
}

impl SchedulerPolicy for SyscallExecutor {
    fn set_scheduling_policy(&self, pid: Pid, policy: &str) -> SyscallResult {
        self.set_scheduling_policy(pid, policy)
    }

    fn get_scheduling_policy(&self, pid: Pid) -> SyscallResult {
        self.get_scheduling_policy(pid)
    }

    fn set_time_quantum(&self, pid: Pid, quantum_micros: u64) -> SyscallResult {
        self.set_time_quantum(pid, quantum_micros)
    }

    fn get_time_quantum(&self, pid: Pid) -> SyscallResult {
        self.get_time_quantum(pid)
    }
}

impl SchedulerStats for SyscallExecutor {
    fn get_scheduler_stats(&self, pid: Pid) -> SyscallResult {
        self.get_scheduler_stats(pid)
    }

    fn get_process_scheduler_stats(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        self.get_process_scheduler_stats(pid, target_pid)
    }

    fn get_all_process_scheduler_stats(&self, pid: Pid) -> SyscallResult {
        self.get_all_process_scheduler_stats(pid)
    }
}

impl PriorityControl for SyscallExecutor {
    fn boost_priority(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        self.boost_priority(pid, target_pid)
    }

    fn lower_priority(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        self.lower_priority(pid, target_pid)
    }
}
