/*!
 * Scheduler Syscalls
 * Syscall interface for scheduler operations
 */

use crate::core::serialization::json;
use crate::core::types::Pid;
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};
use crate::scheduler::{PriorityControl, SchedulerControl, SchedulerPolicy, SchedulerStats};
use log::{error, info};

use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::types::SyscallResult;

impl SyscallExecutorWithIpc {
    /// Schedule next process (internal implementation)
    pub(in crate::syscalls) fn schedule_next(&self, pid: Pid) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "scheduler".into(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        info!("Schedule next syscall requested by PID {}", pid);

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.schedule_next() {
            Some(next_pid) => {
                info!("Scheduler selected next process: PID {}", next_pid);
                use crate::core::PooledBuffer;
                let bytes = next_pid.to_le_bytes();
                let mut buf = PooledBuffer::small();
                buf.extend_from_slice(&bytes);
                SyscallResult::success_with_data(buf.into_vec())
            }
            None => {
                info!("No processes available to schedule");
                SyscallResult::success()
            }
        }
    }

    /// Yield current process (internal implementation)
    pub(in crate::syscalls) fn yield_process(&self, pid: Pid) -> SyscallResult {
        info!("Process {} yielding CPU", pid);

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.yield_current() {
            Some(next_pid) => {
                info!("Process {} yielded, next process: {}", pid, next_pid);
                use crate::core::PooledBuffer;
                let bytes = next_pid.to_le_bytes();
                let mut buf = PooledBuffer::small();
                buf.extend_from_slice(&bytes);
                SyscallResult::success_with_data(buf.into_vec())
            }
            None => {
                info!("Process {} yielded, no next process", pid);
                SyscallResult::success()
            }
        }
    }

    /// Get currently scheduled process (internal implementation)
    pub(in crate::syscalls) fn get_current_scheduled(&self, pid: Pid) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "scheduler".into(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        info!("Get current scheduled process requested by PID {}", pid);

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.current_scheduled() {
            Some(current_pid) => {
                use crate::core::PooledBuffer;
                let bytes = current_pid.to_le_bytes();
                let mut buf = PooledBuffer::small();
                buf.extend_from_slice(&bytes);
                SyscallResult::success_with_data(buf.into_vec())
            }
            None => SyscallResult::success(),
        }
    }

    /// Set scheduling policy (internal implementation)
    pub(in crate::syscalls) fn set_scheduling_policy(
        &self,
        pid: Pid,
        policy_str: &str,
    ) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "scheduler".into(),
            },
            Action::Write,
        );
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let syscall_policy = match crate::scheduler::Policy::from_str(policy_str) {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid scheduling policy requested: {}", policy_str);
                return SyscallResult::error(e);
            }
        };

        let policy = match syscall_policy {
            crate::scheduler::Policy::RoundRobin => {
                crate::process::types::SchedulingPolicy::RoundRobin
            }
            crate::scheduler::Policy::Priority => crate::process::types::SchedulingPolicy::Priority,
            crate::scheduler::Policy::Fair => crate::process::types::SchedulingPolicy::Fair,
        };

        info!("PID {} changing scheduling policy to {:?}", pid, policy);

        let process_manager = match &self.optional().process_manager {
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
    pub(in crate::syscalls) fn get_scheduling_policy(&self, pid: Pid) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "scheduler".into(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
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

                match json::to_vec(&policy_name) {
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
    pub(in crate::syscalls) fn set_time_quantum(
        &self,
        pid: Pid,
        quantum_micros: u64,
    ) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "scheduler".into(),
            },
            Action::Write,
        );
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        info!(
            "PID {} setting time quantum to {} microseconds",
            pid, quantum_micros
        );

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.set_time_quantum(quantum_micros) {
            Ok(()) => {
                info!("Time quantum updated to {} microseconds", quantum_micros);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to set time quantum: {}", e);
                SyscallResult::error(e)
            }
        }
    }

    /// Get current time quantum (internal implementation)
    pub(in crate::syscalls) fn get_time_quantum(&self, pid: Pid) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "scheduler".into(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_scheduler_stats() {
            Some(stats) => {
                use crate::core::PooledBuffer;
                let bytes = stats.quantum_micros.to_le_bytes();
                let mut buf = PooledBuffer::small();
                buf.extend_from_slice(&bytes);
                let data = buf.into_vec();
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
    pub(in crate::syscalls) fn get_scheduler_stats(&self, pid: Pid) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "scheduler".into(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        info!("PID {} requested scheduler statistics", pid);

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_scheduler_stats() {
            Some(stats) => match json::to_vec(&stats) {
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
    pub(in crate::syscalls) fn get_process_scheduler_stats(
        &self,
        pid: Pid,
        target_pid: Pid,
    ) -> SyscallResult {
        let request =
            PermissionRequest::new(pid, Resource::Process { pid: target_pid }, Action::Inspect);
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_process_stats(target_pid) {
            Some(stats) => match json::to_vec(&stats) {
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
    pub(in crate::syscalls) fn get_all_process_scheduler_stats(&self, pid: Pid) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "scheduler".into(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        let stats = process_manager.get_all_process_stats();
        match json::to_vec(&stats) {
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
    pub(in crate::syscalls) fn boost_priority(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        let request =
            PermissionRequest::new(pid, Resource::Process { pid: target_pid }, Action::Write);
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.boost_process_priority(target_pid) {
            Ok(new_priority) => {
                info!(
                    "PID {} successfully boosted priority of PID {} to {}",
                    pid, target_pid, new_priority
                );
                use crate::core::PooledBuffer;
                let bytes = new_priority.to_le_bytes();
                let mut buf = PooledBuffer::small();
                buf.extend_from_slice(&bytes);
                SyscallResult::success_with_data(buf.into_vec())
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

    /// Lower process priority (internal implementation)
    pub(in crate::syscalls) fn lower_priority(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        let request =
            PermissionRequest::new(pid, Resource::Process { pid: target_pid }, Action::Write);
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.lower_process_priority(target_pid) {
            Ok(new_priority) => {
                info!(
                    "PID {} successfully lowered priority of PID {} to {}",
                    pid, target_pid, new_priority
                );
                use crate::core::PooledBuffer;
                let bytes = new_priority.to_le_bytes();
                let mut buf = PooledBuffer::small();
                buf.extend_from_slice(&bytes);
                SyscallResult::success_with_data(buf.into_vec())
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
}

// Implement trait interfaces by delegating to internal methods
impl SchedulerControl for SyscallExecutorWithIpc {
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

impl SchedulerPolicy for SyscallExecutorWithIpc {
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

impl SchedulerStats for SyscallExecutorWithIpc {
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

impl PriorityControl for SyscallExecutorWithIpc {
    fn boost_priority(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        self.boost_priority(pid, target_pid)
    }

    fn lower_priority(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        self.lower_priority(pid, target_pid)
    }
}
