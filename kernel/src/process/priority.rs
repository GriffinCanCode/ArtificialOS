/*!
 * Process Priority Management
 * Helpers for managing process priorities and limits
 */

use super::types::ProcessInfo;
use crate::core::types::{Pid, Priority};
use crate::process::Scheduler;
use crate::security::Limits;
use dashmap::DashMap;
use ahash::RandomState;
use log::info;
use parking_lot::RwLock;
use std::sync::Arc;

/// Convert priority to resource limits
pub(super) fn priority_to_limits(priority: Priority) -> Limits {
    // Higher priority = more resources
    let (memory_bytes, max_pids, max_open_files, cpu_shares) = match priority {
        p if p >= 90 => (
            512 * 1024 * 1024,  // 512MB
            1000,               // 1000 pids
            10000,              // 10000 files
            1024,               // High CPU share
        ),
        p if p >= 70 => (
            256 * 1024 * 1024,  // 256MB
            500,                // 500 pids
            5000,               // 5000 files
            512,                // Medium-high CPU share
        ),
        p if p >= 50 => (
            128 * 1024 * 1024,  // 128MB
            250,                // 250 pids
            2000,               // 2000 files
            256,                // Medium CPU share
        ),
        p if p >= 30 => (
            64 * 1024 * 1024,   // 64MB
            100,                // 100 pids
            1000,               // 1000 files
            128,                // Low-medium CPU share
        ),
        _ => (
            32 * 1024 * 1024,   // 32MB
            50,                 // 50 pids
            500,                // 500 files
            64,                 // Low CPU share
        ),
    };

    Limits {
        memory_bytes: Some(memory_bytes),
        cpu_shares: Some(cpu_shares),
        max_pids: Some(max_pids),
        max_open_files: Some(max_open_files),
    }
}

/// Set process priority in manager
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

    info!("Boosted PID {} priority: {} -> {}", pid, current_priority, new_priority);
    Ok(new_priority)
}

/// Lower process priority
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

    info!("Lowered PID {} priority: {} -> {}", pid, current_priority, new_priority);
    Ok(new_priority)
}
