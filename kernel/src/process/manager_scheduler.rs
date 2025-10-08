/*!
 * Process Manager Scheduler Integration
 * Methods for scheduler control and statistics
 */

use super::manager::ProcessManager;
use super::priority;
use super::scheduler_task::SchedulerTask;
use super::types::{ProcessStats, SchedulerStats, SchedulingPolicy};
use crate::core::types::{Pid, Priority};
use log::info;
use std::sync::Arc;

impl ProcessManager {
    /// Get scheduler statistics
    pub fn get_scheduler_stats(&self) -> Option<SchedulerStats> {
        self.scheduler.as_ref().map(|s| s.read().stats())
    }

    /// Get current scheduling policy
    pub fn get_scheduling_policy(&self) -> Option<SchedulingPolicy> {
        self.scheduler.as_ref().map(|s| s.read().policy())
    }

    /// Change scheduling policy (requires scheduler)
    pub fn set_scheduling_policy(&self, policy: SchedulingPolicy) -> bool {
        if let Some(ref scheduler) = self.scheduler {
            let scheduler = scheduler.read();
            scheduler.set_policy(policy);
            info!("Scheduling policy changed to {:?}", policy);
            true
        } else {
            false
        }
    }

    /// Get per-process CPU statistics (requires scheduler)
    pub fn get_process_stats(&self, pid: Pid) -> Option<ProcessStats> {
        self.scheduler
            .as_ref()
            .and_then(|s| s.read().process_stats(pid))
    }

    /// Get all process CPU statistics (requires scheduler)
    pub fn get_all_process_stats(&self) -> Vec<ProcessStats> {
        self.scheduler
            .as_ref()
            .map(|s| s.read().all_process_stats())
            .unwrap_or_default()
    }

    /// Schedule next process (requires scheduler)
    pub fn schedule_next(&self) -> Option<u32> {
        self.scheduler.as_ref().and_then(|s| s.read().schedule())
    }

    /// Yield current process (requires scheduler)
    pub fn yield_current(&self) -> Option<u32> {
        self.scheduler
            .as_ref()
            .and_then(|s| s.read().yield_process())
    }

    /// Get currently scheduled process (requires scheduler)
    pub fn current_scheduled(&self) -> Option<u32> {
        self.scheduler.as_ref().and_then(|s| s.read().current())
    }

    /// Set process priority
    pub fn set_process_priority(&self, pid: Pid, new_priority: Priority) -> bool {
        // Update process info
        let mut entry = match self.processes.get_mut(&pid) {
            Some(e) => e,
            None => return false,
        };

        let old_priority = entry.priority;
        let os_pid = entry.os_pid;
        entry.priority = new_priority;

        info!(
            "Updated priority for PID {}: {} -> {}",
            pid, old_priority, new_priority
        );

        drop(entry); // Release DashMap lock

        // Update scheduler if available (use efficient in-place update)
        if let Some(ref scheduler) = self.scheduler {
            let scheduler = scheduler.read();
            if scheduler.set_priority(pid, new_priority) {
                info!("Updated PID {} priority in scheduler", pid);
            }
        }

        // Update resource limits if executor and limit manager available
        if let Some(os_pid) = os_pid {
            if let (Some(ref limit_mgr), Some(_)) = (&self.limit_manager, &self.executor) {
                let new_limits = priority::priority_to_limits(new_priority);
                if let Err(e) = limit_mgr.apply(os_pid, &new_limits) {
                    log::warn!("Failed to update resource limits for PID {}: {}", pid, e);
                } else {
                    info!(
                        "Updated resource limits for PID {} (OS PID {})",
                        pid, os_pid
                    );
                }
            }
        }

        true
    }

    /// Boost process priority
    pub fn boost_process_priority(&self, pid: Pid) -> Result<Priority, String> {
        let current_priority = self
            .processes
            .get(&pid)
            .map(|r| r.value().priority)
            .ok_or_else(|| format!("Process {} not found", pid))?;

        let new_priority = crate::scheduler::apply_priority_op(
            current_priority,
            crate::scheduler::PriorityOp::Boost,
        )?;

        if self.set_process_priority(pid, new_priority) {
            Ok(new_priority)
        } else {
            Err("Failed to update priority".to_string())
        }
    }

    /// Lower process priority
    pub fn lower_process_priority(&self, pid: Pid) -> Result<Priority, String> {
        let current_priority = self
            .processes
            .get(&pid)
            .map(|r| r.value().priority)
            .ok_or_else(|| format!("Process {} not found", pid))?;

        let new_priority = crate::scheduler::apply_priority_op(
            current_priority,
            crate::scheduler::PriorityOp::Lower,
        )?;

        if self.set_process_priority(pid, new_priority) {
            Ok(new_priority)
        } else {
            Err("Failed to update priority".to_string())
        }
    }

    /// Set scheduler time quantum (requires scheduler)
    pub fn set_time_quantum(&self, quantum_micros: u64) -> Result<(), String> {
        let quantum = std::time::Duration::from_micros(quantum_micros);

        // Validate using scheduler's time quantum type
        crate::scheduler::TimeQuantum::new(quantum_micros)?;

        if let Some(ref scheduler) = self.scheduler {
            scheduler.read().set_quantum(quantum);

            // Update the scheduler task's interval dynamically
            if let Some(ref task) = self.scheduler_task {
                task.update_quantum(quantum_micros);
            }

            info!("Time quantum updated to {} microseconds", quantum_micros);
            Ok(())
        } else {
            Err("Scheduler not available".to_string())
        }
    }

    /// Get scheduler task for advanced control (pause/resume/trigger)
    pub fn scheduler_task(&self) -> Option<&Arc<SchedulerTask>> {
        self.scheduler_task.as_ref()
    }
}
