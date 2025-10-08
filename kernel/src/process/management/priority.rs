/*!
 * Process Priority Management
 * Helpers for managing process priorities and limits
 */

use crate::core::types::{Pid, Priority};
use crate::process::core::types::ProcessInfo;
use crate::process::scheduler::Scheduler;
use crate::security::Limits;
use ahash::RandomState;
use dashmap::DashMap;
use log::info;
use parking_lot::RwLock;
use std::sync::Arc;

/// Convert priority to resource limits
pub(crate) fn priority_to_limits(priority: Priority) -> Limits {
    // Higher priority = more resources
    use crate::core::limits::*;
    let (memory_bytes, max_pids, max_open_files, cpu_shares) = match priority {
        p if p >= 90 => (
            HIGH_PRIORITY_MEMORY,
            1000,  // 1000 pids
            10000, // 10000 files
            HIGH_PRIORITY_CPU_SHARES,
        ),
        p if p >= 70 => (
            NORMAL_PRIORITY_MEMORY,
            500,  // 500 pids
            5000, // 5000 files
            NORMAL_PRIORITY_CPU_SHARES,
        ),
        p if p >= 50 => (
            LOW_PRIORITY_MEMORY,
            250,  // 250 pids
            2000, // 2000 files
            LOW_PRIORITY_CPU_SHARES,
        ),
        p if p >= 30 => (
            BACKGROUND_PRIORITY_MEMORY,
            BACKGROUND_PRIORITY_MAX_PIDS as usize,
            1000, // 1000 files
            BACKGROUND_PRIORITY_CPU_SHARES,
        ),
        _ => (
            IDLE_PRIORITY_MEMORY,
            50,  // 50 pids
            500, // 500 files
            64,  // Low CPU share
        ),
    };

    Limits {
        memory_bytes: Some(memory_bytes as u64),
        cpu_shares: Some(cpu_shares as u32),
        max_pids: Some(max_pids as u32),
        max_open_files: Some(max_open_files),
    }
}

/// Set process priority in manager
#[allow(dead_code)]
pub(super) fn set_process_priority(
    processes: &Arc<DashMap<Pid, ProcessInfo, RandomState>>,
    scheduler: &Option<Arc<RwLock<Scheduler>>>,
    pid: Pid,
    new_priority: Priority,
) -> bool {
    if new_priority > 100 {
        return false;
    }

    let mut updated = false;

    // Update in process table
    if let Some(mut proc) = processes.get_mut(&pid) {
        let old_priority = proc.priority;
        proc.priority = new_priority;
        updated = true;

        info!(
            "Updated PID {} priority: {} -> {}",
            pid, old_priority, new_priority
        );
    }

    // Update in scheduler if present
    if updated {
        if let Some(scheduler) = scheduler {
            scheduler.write().set_priority(pid, new_priority);
        }
    }

    updated
}

/// Boost process priority
#[allow(dead_code)]
pub(super) fn boost_process_priority(
    processes: &Arc<DashMap<Pid, ProcessInfo, RandomState>>,
    scheduler: &Option<Arc<RwLock<Scheduler>>>,
    pid: Pid,
) -> Result<Priority, String> {
    let current_priority = processes
        .get(&pid)
        .map(|proc| proc.priority)
        .ok_or_else(|| format!("Process {} not found", pid))?;

    let new_priority = (current_priority + 10).min(100);

    if new_priority == current_priority {
        return Err(format!(
            "Process {} already at maximum priority ({})",
            pid, current_priority
        ));
    }

    if !set_process_priority(processes, scheduler, pid, new_priority) {
        return Err(format!("Failed to update priority for process {}", pid));
    }

    info!(
        "Boosted PID {} priority: {} -> {}",
        pid, current_priority, new_priority
    );
    Ok(new_priority)
}

/// Lower process priority
#[allow(dead_code)]
pub(super) fn lower_process_priority(
    processes: &Arc<DashMap<Pid, ProcessInfo, RandomState>>,
    scheduler: &Option<Arc<RwLock<Scheduler>>>,
    pid: Pid,
) -> Result<Priority, String> {
    let current_priority = processes
        .get(&pid)
        .map(|proc| proc.priority)
        .ok_or_else(|| format!("Process {} not found", pid))?;

    let new_priority = current_priority.saturating_sub(10);

    if new_priority == current_priority {
        return Err(format!(
            "Process {} already at minimum priority ({})",
            pid, current_priority
        ));
    }

    if !set_process_priority(processes, scheduler, pid, new_priority) {
        return Err(format!("Failed to update priority for process {}", pid));
    }

    info!(
        "Lowered PID {} priority: {} -> {}",
        pid, current_priority, new_priority
    );
    Ok(new_priority)
}
