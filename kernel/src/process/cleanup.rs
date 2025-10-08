/*!
 * Process Cleanup Logic
 * Core cleanup operations for OS processes and scheduling
 *
 * Note: Resource cleanup (memory, IPC, FDs, etc.) is handled by ResourceOrchestrator
 */

use super::executor::ProcessExecutor;
use super::preemption::PreemptionController;
use super::scheduler::Scheduler;
use super::types::ProcessInfo;
use crate::core::types::Pid;
use crate::security::LimitManager;
use log::warn;
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

/// Remove from scheduler
pub(super) fn cleanup_scheduler(pid: Pid, scheduler: &Option<Arc<RwLock<Scheduler>>>) {
    if let Some(ref sched) = scheduler {
        sched.read().remove(pid);
    }
}

/// Notify preemption controller
pub(super) fn cleanup_preemption(pid: Pid, preemption: &Option<Arc<PreemptionController>>) {
    if let Some(ref preempt) = preemption {
        preempt.cleanup_process(pid);
    }
}
