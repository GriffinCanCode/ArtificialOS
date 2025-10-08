/*!
 * Scheduler Syscall Handler
 * Handles scheduler-related syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for scheduler syscalls
pub struct SchedulerHandler {
    executor: SyscallExecutorWithIpc,
}

impl SchedulerHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for SchedulerHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::ScheduleNext => Some(self.executor.schedule_next(pid)),
            Syscall::YieldProcess => Some(self.executor.yield_process(pid)),
            Syscall::GetCurrentScheduled => Some(self.executor.get_current_scheduled(pid)),
            Syscall::GetSchedulerStats => Some(self.executor.get_scheduler_stats(pid)),
            Syscall::SetSchedulingPolicy { ref policy } => {
                Some(self.executor.set_scheduling_policy(pid, policy))
            }
            Syscall::GetSchedulingPolicy => Some(self.executor.get_scheduling_policy(pid)),
            Syscall::SetTimeQuantum { quantum_micros } => {
                Some(self.executor.set_time_quantum(pid, *quantum_micros))
            }
            Syscall::GetTimeQuantum => Some(self.executor.get_time_quantum(pid)),
            Syscall::GetProcessSchedulerStats { target_pid } => {
                Some(self.executor.get_process_scheduler_stats(pid, *target_pid))
            }
            Syscall::GetAllProcessSchedulerStats => {
                Some(self.executor.get_all_process_scheduler_stats(pid))
            }
            Syscall::BoostPriority { target_pid } => {
                Some(self.executor.boost_priority(pid, *target_pid))
            }
            Syscall::LowerPriority { target_pid } => {
                Some(self.executor.lower_priority(pid, *target_pid))
            }
            _ => None, // Not a scheduler syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "scheduler_handler"
    }
}
