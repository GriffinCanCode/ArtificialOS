/*!
 * Time Syscall Handler
 * Handles time-related syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for time syscalls
pub struct TimeHandler {
    executor: SyscallExecutorWithIpc,
}

impl TimeHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for TimeHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::Sleep { duration_ms } => Some(self.executor.sleep(pid, *duration_ms).into()),
            Syscall::GetUptime => Some(self.executor.get_uptime(pid).into()),
            _ => None, // Not a time syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "time_handler"
    }
}
