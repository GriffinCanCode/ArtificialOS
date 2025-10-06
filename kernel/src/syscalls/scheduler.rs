/*!

 * Scheduler Syscalls
 * CPU scheduling operations
 */

use crate::core::types::Pid;

use log::info;

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

impl SyscallExecutor {
    pub(super) fn schedule_next(&self, _pid: Pid) -> SyscallResult {
        info!("Schedule next syscall requested");
        // Note: Scheduler operations would require ProcessManager access
        // This is a placeholder that returns success
        // In a full implementation, this would call ProcessManager::schedule_next()
        SyscallResult::success()
    }

    pub(super) fn yield_process(&self, pid: Pid) -> SyscallResult {
        info!("Process {} yielding CPU", pid);
        // Note: This would call ProcessManager::yield_current()
        SyscallResult::success()
    }

    pub(super) fn get_current_scheduled(&self, _pid: Pid) -> SyscallResult {
        info!("Get current scheduled process requested");
        // Note: This would call ProcessManager::current_scheduled()
        SyscallResult::success()
    }

    pub(super) fn get_scheduler_stats(&self, pid: Pid) -> SyscallResult {
        info!("PID {} requested scheduler statistics", pid);
        // Note: This would call Scheduler::stats() via ProcessManager
        // Placeholder implementation
        SyscallResult::success()
    }
}
