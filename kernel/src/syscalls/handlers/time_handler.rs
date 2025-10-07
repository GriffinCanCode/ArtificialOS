/*!
 * Time Syscall Handler
 * Handles time-related syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};
use crate::syscalls::executor::SyscallExecutor;

/// Handler for time syscalls
pub struct TimeHandler {
    executor: SyscallExecutor,
}

impl TimeHandler {
    #[inline]
    pub fn new(executor: SyscallExecutor) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for TimeHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::Sleep { duration_ms } => {
                Some(self.executor.sleep(pid, *duration_ms))
            }
            Syscall::GetUptime => {
                Some(self.executor.get_uptime(pid))
            }
            _ => None, // Not a time syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "time_handler"
    }
}
