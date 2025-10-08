/*!
 * Process Syscall Handler
 * Handles all process management related syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::executor::SyscallExecutorWithIpc;
use crate::syscalls::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for process management syscalls
pub struct ProcessHandler {
    executor: SyscallExecutorWithIpc,
}

impl ProcessHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for ProcessHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::SpawnProcess {
                ref command,
                ref args,
            } => Some(self.executor.spawn_process(pid, command, args)),
            Syscall::KillProcess { target_pid } => {
                Some(self.executor.kill_process(pid, *target_pid))
            }
            Syscall::GetProcessInfo { target_pid } => {
                Some(self.executor.get_process_info(pid, *target_pid))
            }
            Syscall::GetProcessList => Some(self.executor.get_process_list(pid)),
            Syscall::SetProcessPriority {
                target_pid,
                priority,
            } => Some(
                self.executor
                    .set_process_priority(pid, *target_pid, *priority),
            ),
            Syscall::GetProcessState { target_pid } => {
                Some(self.executor.get_process_state(pid, *target_pid))
            }
            Syscall::GetProcessStats { target_pid } => {
                Some(self.executor.get_process_stats_call(pid, *target_pid))
            }
            Syscall::WaitProcess {
                target_pid,
                timeout_ms,
            } => Some(self.executor.wait_process(pid, *target_pid, *timeout_ms)),
            _ => None, // Not a process syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "process_handler"
    }
}
