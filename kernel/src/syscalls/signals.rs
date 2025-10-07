/*!
 * Signal Syscalls
 * Syscall interface for signal operations
 */

use crate::core::json;
use crate::core::types::Pid;
use crate::security::Capability;
use crate::signals::{
    Signal, SignalAction, SignalDelivery, SignalHandlerRegistry, SignalMasking, SignalQueue,
};
use log::{error, info};

use super::executor::SyscallExecutor;
use super::traits::SignalSyscalls;
use super::types::SyscallResult;

impl SyscallExecutor {
    /// Send signal to a process
    pub(super) fn send_signal(&self, pid: Pid, target_pid: Pid, signal: u32) -> SyscallResult {
        // Check permission
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::KillProcess)
        {
            return SyscallResult::permission_denied("Missing KillProcess capability");
        }

        // Get signal manager
        let signal_manager = match &self.signal_manager {
            Some(mgr) => mgr,
            None => {
                error!("Signal manager not available");
                return SyscallResult::error("Signal manager not available");
            }
        };

        // Convert signal number to Signal enum
        let signal_enum = match Signal::from_number(signal) {
            Ok(sig) => sig,
            Err(e) => {
                error!("Invalid signal number {}: {}", signal, e);
                return SyscallResult::error(format!("Invalid signal: {}", e));
            }
        };

        // Send signal
        match signal_manager.send(pid, target_pid, signal_enum) {
            Ok(()) => {
                info!(
                    "PID {} sent signal {:?} ({}) to PID {}",
                    pid, signal_enum, signal, target_pid
                );
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to send signal: {}", e);
                SyscallResult::error(format!("Signal send failed: {}", e))
            }
        }
    }

    /// Register signal handler
    fn register_signal_handler(&self, pid: Pid, signal: u32, handler_id: u64) -> SyscallResult {
        let signal_manager = match &self.signal_manager {
            Some(mgr) => mgr,
            None => return SyscallResult::error("Signal manager not available"),
        };

        let signal_enum = match Signal::from_number(signal) {
            Ok(sig) => sig,
            Err(e) => return SyscallResult::error(format!("Invalid signal: {}", e)),
        };

        match signal_manager.register_handler(pid, signal_enum, SignalAction::Handler(handler_id)) {
            Ok(()) => {
                info!(
                    "PID {} registered handler {} for signal {:?}",
                    pid, handler_id, signal_enum
                );
                SyscallResult::success()
            }
            Err(e) => SyscallResult::error(format!("Failed to register handler: {}", e)),
        }
    }

    /// Block signal
    fn block_signal(&self, pid: Pid, signal: u32) -> SyscallResult {
        let signal_manager = match &self.signal_manager {
            Some(mgr) => mgr,
            None => return SyscallResult::error("Signal manager not available"),
        };

        let signal_enum = match Signal::from_number(signal) {
            Ok(sig) => sig,
            Err(e) => return SyscallResult::error(format!("Invalid signal: {}", e)),
        };

        match signal_manager.block_signal(pid, signal_enum) {
            Ok(()) => {
                info!("PID {} blocked signal {:?}", pid, signal_enum);
                SyscallResult::success()
            }
            Err(e) => SyscallResult::error(format!("Failed to block signal: {}", e)),
        }
    }

    /// Unblock signal
    fn unblock_signal(&self, pid: Pid, signal: u32) -> SyscallResult {
        let signal_manager = match &self.signal_manager {
            Some(mgr) => mgr,
            None => return SyscallResult::error("Signal manager not available"),
        };

        let signal_enum = match Signal::from_number(signal) {
            Ok(sig) => sig,
            Err(e) => return SyscallResult::error(format!("Invalid signal: {}", e)),
        };

        match signal_manager.unblock_signal(pid, signal_enum) {
            Ok(()) => {
                info!("PID {} unblocked signal {:?}", pid, signal_enum);
                SyscallResult::success()
            }
            Err(e) => SyscallResult::error(format!("Failed to unblock signal: {}", e)),
        }
    }

    /// Get pending signals
    fn get_pending_signals(&self, pid: Pid) -> SyscallResult {
        let signal_manager = match &self.signal_manager {
            Some(mgr) => mgr,
            None => return SyscallResult::error("Signal manager not available"),
        };

        let pending = signal_manager.pending_signals(pid);
        let signal_numbers: Vec<u32> = pending.iter().map(|s| s.number()).collect();

        match json::to_vec(&signal_numbers) {
            Ok(data) => SyscallResult::success_with_data(data),
            Err(e) => {
                error!("Failed to serialize pending signals: {}", e);
                SyscallResult::error("Failed to serialize pending signals")
            }
        }
    }
}

// Implement SignalSyscalls trait
impl SignalSyscalls for SyscallExecutor {
    fn send_signal(&self, pid: Pid, target_pid: Pid, signal: u32) -> SyscallResult {
        self.send_signal(pid, target_pid, signal)
    }

    fn register_signal_handler(&self, pid: Pid, signal: u32, handler_id: u64) -> SyscallResult {
        self.register_signal_handler(pid, signal, handler_id)
    }

    fn block_signal(&self, pid: Pid, signal: u32) -> SyscallResult {
        self.block_signal(pid, signal)
    }

    fn unblock_signal(&self, pid: Pid, signal: u32) -> SyscallResult {
        self.unblock_signal(pid, signal)
    }

    fn get_pending_signals(&self, pid: Pid) -> SyscallResult {
        self.get_pending_signals(pid)
    }
}
