/*!
 * Signal Syscall Handler
 * Handles signal-related syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::executor::SyscallExecutor;
use crate::syscalls::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for signal syscalls
pub struct SignalHandler {
    executor: SyscallExecutor,
}

impl SignalHandler {
    #[inline]
    pub fn new(executor: SyscallExecutor) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for SignalHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::SendSignal { target_pid, signal } => {
                Some(self.executor.send_signal(pid, *target_pid, *signal))
            }
            Syscall::RegisterSignalHandler { signal, handler_id } => Some(
                self.executor
                    .register_signal_handler(pid, *signal, *handler_id),
            ),
            Syscall::BlockSignal { signal } => Some(self.executor.block_signal(pid, *signal)),
            Syscall::UnblockSignal { signal } => Some(self.executor.unblock_signal(pid, *signal)),
            Syscall::GetPendingSignals => Some(self.executor.get_pending_signals(pid)),
            Syscall::GetSignalStats => Some(self.executor.get_signal_stats(pid)),
            Syscall::WaitForSignal {
                signals,
                timeout_ms,
            } => Some(self.executor.wait_for_signal(pid, signals, *timeout_ms)),
            Syscall::GetSignalState { target_pid } => {
                Some(self.executor.get_signal_state(pid, *target_pid))
            }
            _ => None, // Not a signal syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "signal_handler"
    }
}
