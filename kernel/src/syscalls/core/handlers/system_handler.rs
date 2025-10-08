/*!
 * System Information Syscall Handler
 * Handles system information and environment syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for system information syscalls
pub struct SystemHandler {
    executor: SyscallExecutorWithIpc,
}

impl SystemHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for SystemHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::GetSystemInfo => Some(self.executor.get_system_info(pid)),
            Syscall::GetCurrentTime => Some(self.executor.get_current_time(pid)),
            Syscall::GetEnvironmentVar { ref key } => Some(self.executor.get_env_var(pid, key)),
            Syscall::SetEnvironmentVar { ref key, ref value } => {
                Some(self.executor.set_env_var(pid, key, value))
            }
            _ => None, // Not a system info syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "system_handler"
    }
}
