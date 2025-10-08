/*!
 * Signal Syscall Handler
 * Handles signal-related syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for signal syscalls
pub struct SignalHandler {
    executor: SyscallExecutorWithIpc,
}

impl SignalHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
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
            Syscall::BlockSignal { signal } => Some(self.executor.block_signal(pid, *signal).into()),
            Syscall::UnblockSignal { signal } => Some(self.executor.unblock_signal(pid, *signal).into()),
            Syscall::GetPendingSignals => Some(self.executor.get_pending_signals(pid).into()),
            Syscall::GetSignalStats => Some(self.executor.get_signal_stats(pid).into()),
            Syscall::WaitForSignal {
                signals,
                timeout_ms,
            } => Some(self.executor.wait_for_signal(pid, signals, *timeout_ms).into()),
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
