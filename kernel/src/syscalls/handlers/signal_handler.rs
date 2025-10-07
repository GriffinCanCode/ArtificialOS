/*!
 * Signal Syscall Handler
 * Handles signal-related syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};
use crate::syscalls::executor::SyscallExecutor;

/// Handler for signal syscalls
pub struct SignalHandler {
    executor: SyscallExecutor,
}

impl SignalHandler {
    pub fn new(executor: SyscallExecutor) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for SignalHandler {
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::SendSignal { target_pid, signal } => {
                Some(self.executor.send_signal(pid, *target_pid, *signal))
            }
            _ => None, // Not a signal syscall
        }
    }

    fn name(&self) -> &'static str {
        "signal_handler"
    }
}
