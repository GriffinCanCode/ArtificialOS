/*!
 * Process Cleanup Logic
 * Handles resource cleanup when processes terminate
 */

use super::executor::ProcessExecutor;
use super::preemption::PreemptionController;
use super::scheduler::Scheduler;
use super::types::ProcessInfo;
use crate::core::types::Pid;
use crate::ipc::IPCManager;
use crate::memory::MemoryManager;
use crate::security::LimitManager;
use log::{info, warn};
use parking_lot::RwLock;
use std::sync::Arc;

/// Cleanup OS process resources
pub(super) fn cleanup_os_process(
    process: &ProcessInfo,
    pid: Pid,
    executor: &Option<ProcessExecutor>,
    limit_manager: &Option<LimitManager>,
) {
    if let Some(os_pid) = process.os_pid {
        if let Some(ref exec) = executor {
            if let Err(e) = exec.kill(pid) {
                warn!("Failed to kill OS process: {}", e);
            }

            // Remove resource limits
            if let Some(ref limit_mgr) = limit_manager {
                if let Err(e) = limit_mgr.remove(os_pid) {
                    warn!("Failed to remove limits: {}", e);
                }
            }
        }
    }
}

/// Cleanup memory resources
pub(super) fn cleanup_memory(
    pid: Pid,
    memory_manager: &Option<MemoryManager>,
) {
    if let Some(ref mem_mgr) = memory_manager {
        let freed = mem_mgr.free_process_memory(pid);
        if freed > 0 {
            info!("Freed {} bytes from terminated PID {}", freed, pid);
        }
    }
}

/// Cleanup IPC resources
pub(super) fn cleanup_ipc(
    pid: Pid,
    ipc_manager: &Option<IPCManager>,
) {
    if let Some(ref ipc_mgr) = ipc_manager {
        let cleaned = ipc_mgr.clear_process_queue(pid);
        if cleaned > 0 {
            info!(
                "Cleaned up {} IPC resources for terminated PID {}",
                cleaned, pid
            );
        }
    }
}

/// Remove from scheduler
pub(super) fn cleanup_scheduler(
    pid: Pid,
    scheduler: &Option<Arc<RwLock<Scheduler>>>,
) {
    if let Some(ref sched) = scheduler {
        sched.read().remove(pid);
    }
}

/// Notify preemption controller
pub(super) fn cleanup_preemption(
    pid: Pid,
    preemption: &Option<Arc<PreemptionController>>,
) {
    if let Some(ref preempt) = preemption {
        preempt.cleanup_process(pid);
    }
}
