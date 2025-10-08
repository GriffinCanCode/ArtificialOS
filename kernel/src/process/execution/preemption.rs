/*!
 * Preemptive Scheduling Integration
 *
 * Bridges the scheduler with OS-level process control to enable true
 * preemptive multitasking using SIGSTOP/SIGCONT signals.
 */

use super::ProcessExecutor;
use crate::core::types::Pid;
use crate::process::scheduler::Scheduler;
use log::{debug, info, warn};
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(unix)]
use nix::sys::signal::{kill, Signal as UnixSignal};
#[cfg(unix)]
use nix::unistd::Pid as NixPid;

/// Preemption controller that integrates scheduler decisions with OS-level process control
pub struct PreemptionController {
    scheduler: Arc<RwLock<Scheduler>>,
    executor: Arc<ProcessExecutor>,
    last_scheduled: Arc<RwLock<Option<Pid>>>,
}

impl PreemptionController {
    /// Create a new preemption controller
    pub fn new(scheduler: Arc<RwLock<Scheduler>>, executor: Arc<ProcessExecutor>) -> Self {
        info!("Preemption controller initialized - true preemptive multitasking enabled");
        Self {
            scheduler,
            executor,
            last_scheduled: Arc::new(RwLock::new(None).into()),
        }
    }

    /// Perform a scheduling decision with OS-level preemption
    ///
    /// This method:
    /// 1. Calls the scheduler to make a decision
    /// 2. If a different process is selected, pauses the old one (SIGSTOP)
    /// 3. Resumes the new process (SIGCONT)
    pub fn schedule(&self) -> Option<Pid> {
        let next_pid = self.scheduler.read().schedule()?;
        let last_pid = *self.last_scheduled.read();

        // If we're switching to a different process, perform OS-level context switch
        if Some(next_pid) != last_pid {
            self.perform_context_switch(last_pid, next_pid);
        }

        Some(next_pid)
    }

    /// Perform OS-level context switch using signals
    fn perform_context_switch(&self, old_pid: Option<Pid>, new_pid: Pid) {
        // Pause the old process if it exists and has an OS process
        if let Some(old) = old_pid {
            if let Some(os_pid) = self.executor.get_os_pid(old) {
                if self.pause_process(os_pid) {
                    debug!("Preempted PID {} (OS PID {})", old, os_pid);
                }
            }
        }

        // Resume the new process if it has an OS process
        if let Some(os_pid) = self.executor.get_os_pid(new_pid) {
            if self.resume_process(os_pid) {
                debug!("Resumed PID {} (OS PID {})", new_pid, os_pid);
            }
        }

        // Update tracking
        *self.last_scheduled.write() = Some(new_pid);
    }

    /// Pause an OS process using SIGSTOP
    #[cfg(unix)]
    fn pause_process(&self, os_pid: u32) -> bool {
        match kill(NixPid::from_raw(os_pid as i32), UnixSignal::SIGSTOP) {
            Ok(_) => true,
            Err(e) => {
                warn!("Failed to pause OS PID {}: {}", os_pid, e);
                false
            }
        }
    }

    /// Resume an OS process using SIGCONT
    #[cfg(unix)]
    fn resume_process(&self, os_pid: u32) -> bool {
        match kill(NixPid::from_raw(os_pid as i32), UnixSignal::SIGCONT) {
            Ok(_) => true,
            Err(e) => {
                warn!("Failed to resume OS PID {}: {}", os_pid, e);
                false
            }
        }
    }

    /// Non-Unix stubs
    #[cfg(not(unix))]
    fn pause_process(&self, os_pid: u32) -> bool {
        warn!(
            "Process preemption not supported on this platform (OS PID {})",
            os_pid
        );
        false
    }

    #[cfg(not(unix))]
    fn resume_process(&self, os_pid: u32) -> bool {
        warn!(
            "Process resumption not supported on this platform (OS PID {})",
            os_pid
        );
        false
    }

    /// Yield the current process (voluntary context switch)
    pub fn yield_current(&self) -> Option<Pid> {
        let next_pid = self.scheduler.read().yield_process()?;
        let last_pid = *self.last_scheduled.read();

        if Some(next_pid) != last_pid {
            self.perform_context_switch(last_pid, next_pid);
        }

        Some(next_pid)
    }

    /// Get currently scheduled process
    pub fn current(&self) -> Option<Pid> {
        *self.last_scheduled.read()
    }

    /// Clean up when a process is removed
    pub fn cleanup_process(&self, pid: Pid) {
        let mut last = self.last_scheduled.write();
        if *last == Some(pid) {
            *last = None;
        }
    }
}

impl Clone for PreemptionController {
    fn clone(&self) -> Self {
        Self {
            scheduler: Arc::clone(&self.scheduler),
            executor: Arc::clone(&self.executor),
            last_scheduled: Arc::clone(&self.last_scheduled),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::core::types::SchedulingPolicy;

    #[test]
    fn test_preemption_controller_creation() {
        let scheduler = Arc::new(RwLock::new(Scheduler::new(SchedulingPolicy::Fair).into()));
        let executor = Arc::new(ProcessExecutor::new());

        let controller = PreemptionController::new(scheduler, executor);
        assert!(controller.current().is_none());
    }

    #[test]
    fn test_schedule_with_no_processes() {
        let scheduler = Arc::new(RwLock::new(Scheduler::new(SchedulingPolicy::RoundRobin).into()));
        let executor = Arc::new(ProcessExecutor::new());

        let controller = PreemptionController::new(scheduler, executor);
        assert_eq!(controller.schedule(), None);
    }

    #[test]
    fn test_cleanup_process() {
        let scheduler = Arc::new(RwLock::new(Scheduler::new(SchedulingPolicy::Priority).into()));
        let executor = Arc::new(ProcessExecutor::new());

        let controller = PreemptionController::new(scheduler.clone(), executor);

        // Simulate having scheduled PID 1
        *controller.last_scheduled.write() = Some(1);
        assert_eq!(controller.current(), Some(1));

        // Cleanup should clear it
        controller.cleanup_process(1);
        assert_eq!(controller.current(), None);
    }
}
