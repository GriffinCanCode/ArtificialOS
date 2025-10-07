/*!
 * Process Management
 * Handles process creation, scheduling, and lifecycle
 */

use super::types::{ExecutionConfig, ProcessInfo, ProcessState, SchedulingPolicy};
use crate::core::types::{Pid, Priority};
use crate::ipc::IPCManager;
use crate::memory::MemoryManager;
use crate::process::executor::ProcessExecutor;
use crate::process::scheduler::Scheduler;
use crate::process::scheduler_task::SchedulerTask;
use crate::security::{LimitManager, Limits};
use dashmap::DashMap;
use log::info;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

// Type alias for backwards compatibility
pub type Process = ProcessInfo;

pub struct ProcessManager {
    processes: Arc<DashMap<Pid, ProcessInfo>>,
    next_pid: AtomicU32,
    memory_manager: Option<MemoryManager>,
    executor: Option<ProcessExecutor>,
    limit_manager: Option<LimitManager>,
    ipc_manager: Option<IPCManager>,
    scheduler: Option<Arc<RwLock<Scheduler>>>,
    scheduler_task: Option<Arc<SchedulerTask>>,
    // Track child processes per parent PID for limit enforcement
    child_counts: Arc<DashMap<Pid, u32>>,
}

/// Builder for ProcessManager
pub struct ProcessManagerBuilder {
    memory_manager: Option<MemoryManager>,
    enable_executor: bool,
    enable_limits: bool,
    ipc_manager: Option<IPCManager>,
    scheduler_policy: Option<SchedulingPolicy>,
}

impl ProcessManagerBuilder {
    /// Create a new ProcessManager builder
    pub fn new() -> Self {
        Self {
            memory_manager: None,
            enable_executor: false,
            enable_limits: false,
            ipc_manager: None,
            scheduler_policy: None,
        }
    }

    /// Add memory manager
    pub fn with_memory_manager(mut self, memory_manager: MemoryManager) -> Self {
        self.memory_manager = Some(memory_manager);
        self
    }

    /// Enable OS process execution
    pub fn with_executor(mut self) -> Self {
        self.enable_executor = true;
        self
    }

    /// Enable resource limits (requires executor)
    pub fn with_limits(mut self) -> Self {
        self.enable_limits = true;
        self
    }

    /// Add IPC manager for automatic cleanup
    pub fn with_ipc_manager(mut self, ipc_manager: IPCManager) -> Self {
        self.ipc_manager = Some(ipc_manager);
        self
    }

    /// Add scheduler with specified policy
    pub fn with_scheduler(mut self, policy: SchedulingPolicy) -> Self {
        self.scheduler_policy = Some(policy);
        self
    }

    /// Build the ProcessManager
    pub fn build(self) -> ProcessManager {
        let executor = if self.enable_executor {
            Some(ProcessExecutor::new())
        } else {
            None
        };

        let limit_manager = if self.enable_limits {
            LimitManager::new().ok()
        } else {
            None
        };

        let scheduler = self
            .scheduler_policy
            .map(|policy| Arc::new(RwLock::new(Scheduler::new(policy))));

        // Spawn autonomous scheduler task if scheduler is enabled
        let scheduler_task = scheduler.as_ref().map(|sched| {
            Arc::new(SchedulerTask::spawn(Arc::clone(sched)))
        });

        let mut features = Vec::new();
        if self.memory_manager.is_some() {
            features.push("memory");
        }
        if executor.is_some() {
            features.push("executor");
        }
        if limit_manager.is_some() {
            features.push("limits");
        }
        if self.ipc_manager.is_some() {
            features.push("IPC");
        }
        if scheduler.is_some() {
            features.push("scheduler");
        }
        if scheduler_task.is_some() {
            features.push("preemptive-scheduling");
        }

        info!("Process manager initialized with: {}", features.join(", "));

        ProcessManager {
            processes: Arc::new(DashMap::new()),
            next_pid: AtomicU32::new(1),
            memory_manager: self.memory_manager,
            executor,
            limit_manager,
            ipc_manager: self.ipc_manager,
            scheduler,
            scheduler_task,
            child_counts: Arc::new(DashMap::new()),
        }
    }
}

impl Default for ProcessManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    /// Create a basic ProcessManager with no features
    pub fn new() -> Self {
        info!("Process manager initialized (basic)");
        Self {
            processes: Arc::new(DashMap::new()),
            next_pid: AtomicU32::new(1),
            memory_manager: None,
            executor: None,
            limit_manager: None,
            ipc_manager: None,
            scheduler: None,
            scheduler_task: None,
            child_counts: Arc::new(DashMap::new()),
        }
    }

    /// Create a builder for constructing a ProcessManager
    pub fn builder() -> ProcessManagerBuilder {
        ProcessManagerBuilder::new()
    }

    /// Create a process (metadata only, no OS process)
    pub fn create_process(&self, name: String, priority: Priority) -> u32 {
        self.create_process_with_command(name, priority, None)
    }

    /// Create a process with optional OS execution
    pub fn create_process_with_command(
        &self,
        name: String,
        priority: Priority,
        config: Option<ExecutionConfig>,
    ) -> u32 {
        // Allocate PID atomically
        let pid = self.next_pid.fetch_add(1, Ordering::SeqCst);

        // Spawn OS process if command provided and executor available
        let os_pid = if let Some(cfg) = config {
            if let Some(ref executor) = self.executor {
                match executor.spawn(pid, name.clone(), cfg) {
                    Ok(os_pid) => {
                        info!("Spawned OS process {} for PID {}", os_pid, pid);

                        // Apply resource limits based on priority
                        if let Some(ref limit_mgr) = self.limit_manager {
                            let limits = self.priority_to_limits(priority);
                            if let Err(e) = limit_mgr.apply(os_pid, &limits) {
                                log::warn!("Failed to apply limits: {}", e);
                            }
                        }

                        Some(os_pid)
                    }
                    Err(e) => {
                        log::error!("Failed to spawn OS process: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let process = ProcessInfo {
            pid,
            name: name.clone(),
            state: ProcessState::Ready,
            priority,
            os_pid,
        };

        self.processes.insert(pid, process);
        info!(
            "Created process: {} (PID: {}, OS PID: {:?})",
            name, pid, os_pid
        );

        // Add to scheduler if available
        if let Some(ref scheduler) = self.scheduler {
            scheduler.read().add(pid, priority);
        }

        pid
    }

    /// Convert priority to resource limits
    fn priority_to_limits(&self, priority: Priority) -> Limits {
        match priority {
            0..=3 => Limits::new()
                .with_memory(128 * 1024 * 1024) // 128 MB
                .with_cpu_shares(50)
                .with_max_pids(5),
            4..=7 => Limits::default(), // 512 MB, 100 shares
            _ => Limits::new()
                .with_memory(2 * 1024 * 1024 * 1024) // 2 GB
                .with_cpu_shares(200)
                .with_max_pids(50),
        }
    }

    pub fn get_process(&self, pid: Pid) -> Option<ProcessInfo> {
        self.processes.get(&pid).map(|r| r.value().clone())
    }

    pub fn terminate_process(&self, pid: Pid) -> bool {
        if let Some((_, process)) = self.processes.remove(&pid) {
            info!("Terminating process: PID {}", pid);

            // Kill OS process if it exists
            if let Some(os_pid) = process.os_pid {
                if let Some(ref executor) = self.executor {
                    if let Err(e) = executor.kill(pid) {
                        log::warn!("Failed to kill OS process: {}", e);
                    }

                    // Remove resource limits
                    if let Some(ref limit_mgr) = self.limit_manager {
                        if let Err(e) = limit_mgr.remove(os_pid) {
                            log::warn!("Failed to remove limits: {}", e);
                        }
                    }
                }
            }

            // Clean up memory if memory manager is available
            if let Some(ref mem_mgr) = self.memory_manager {
                let freed = mem_mgr.free_process_memory(pid);
                if freed > 0 {
                    info!("Freed {} bytes from terminated PID {}", freed, pid);
                }
            }

            // Clean up IPC resources if IPC manager is available
            if let Some(ref ipc_mgr) = self.ipc_manager {
                let cleaned = ipc_mgr.clear_process_queue(pid);
                if cleaned > 0 {
                    info!(
                        "Cleaned up {} IPC resources for terminated PID {}",
                        cleaned, pid
                    );
                }
            }

            // Remove from scheduler if available
            if let Some(ref scheduler) = self.scheduler {
                scheduler.read().remove(pid);
            }

            true
        } else {
            false
        }
    }

    pub fn list_processes(&self) -> Vec<ProcessInfo> {
        self.processes.iter().map(|r| r.value().clone()).collect()
    }

    /// Get memory manager reference
    pub fn memory_manager(&self) -> Option<&MemoryManager> {
        self.memory_manager.as_ref()
    }

    /// Get executor reference
    pub fn executor(&self) -> Option<&ProcessExecutor> {
        self.executor.as_ref()
    }

    /// Check if process has OS execution
    pub fn has_os_process(&self, pid: Pid) -> bool {
        self.processes
            .get(&pid)
            .and_then(|r| r.value().os_pid)
            .is_some()
    }

    /// Get child process count for a PID
    pub fn get_child_count(&self, pid: Pid) -> u32 {
        self.child_counts.get(&pid).map(|r| *r.value()).unwrap_or(0)
    }

    /// Increment child count for a PID
    fn increment_child_count(&self, pid: Pid) {
        self.child_counts.entry(pid).and_modify(|v| *v += 1).or_insert(1);
    }

    /// Decrement child count for a PID
    fn decrement_child_count(&self, pid: Pid) {
        if let Some(mut entry) = self.child_counts.get_mut(&pid) {
            *entry = entry.saturating_sub(1);
            if *entry == 0 {
                drop(entry);
                self.child_counts.remove(&pid);
            }
        }
    }

    /// Get scheduler statistics
    pub fn get_scheduler_stats(&self) -> Option<super::types::SchedulerStats> {
        self.scheduler.as_ref().map(|s| s.read().stats())
    }

    /// Get current scheduling policy
    pub fn get_scheduling_policy(&self) -> Option<SchedulingPolicy> {
        self.scheduler.as_ref().map(|s| s.read().policy())
    }

    /// Change scheduling policy (requires scheduler)
    pub fn set_scheduling_policy(&self, policy: SchedulingPolicy) -> bool {
        if let Some(ref scheduler) = self.scheduler {
            let scheduler = scheduler.read();
            scheduler.set_policy(policy);
            info!("Scheduling policy changed to {:?}", policy);
            true
        } else {
            false
        }
    }

    /// Get per-process CPU statistics (requires scheduler)
    pub fn get_process_stats(&self, pid: Pid) -> Option<super::types::ProcessStats> {
        self.scheduler
            .as_ref()
            .and_then(|s| s.read().process_stats(pid))
    }

    /// Get all process CPU statistics (requires scheduler)
    pub fn get_all_process_stats(&self) -> Vec<super::types::ProcessStats> {
        self.scheduler
            .as_ref()
            .map(|s| s.read().all_process_stats())
            .unwrap_or_default()
    }

    /// Schedule next process (requires scheduler)
    pub fn schedule_next(&self) -> Option<u32> {
        self.scheduler.as_ref().and_then(|s| s.read().schedule())
    }

    /// Yield current process (requires scheduler)
    pub fn yield_current(&self) -> Option<u32> {
        self.scheduler
            .as_ref()
            .and_then(|s| s.read().yield_process())
    }

    /// Get currently scheduled process (requires scheduler)
    pub fn current_scheduled(&self) -> Option<u32> {
        self.scheduler.as_ref().and_then(|s| s.read().current())
    }

    /// Set process priority
    pub fn set_process_priority(&self, pid: Pid, new_priority: Priority) -> bool {
        // Update process info
        let mut entry = match self.processes.get_mut(&pid) {
            Some(e) => e,
            None => return false,
        };

        let old_priority = entry.priority;
        let os_pid = entry.os_pid;
        entry.priority = new_priority;

        info!(
            "Updated priority for PID {}: {} -> {}",
            pid, old_priority, new_priority
        );

        drop(entry); // Release DashMap lock

        // Update scheduler if available (use efficient in-place update)
        if let Some(ref scheduler) = self.scheduler {
            let scheduler = scheduler.read();
            if scheduler.set_priority(pid, new_priority) {
                info!("Updated PID {} priority in scheduler", pid);
            }
        }

        // Update resource limits if executor and limit manager available
        if let Some(os_pid) = os_pid {
            if let (Some(ref limit_mgr), Some(_)) = (&self.limit_manager, &self.executor) {
                let new_limits = self.priority_to_limits(new_priority);
                if let Err(e) = limit_mgr.apply(os_pid, &new_limits) {
                    log::warn!("Failed to update resource limits for PID {}: {}", pid, e);
                } else {
                    info!("Updated resource limits for PID {} (OS PID {})", pid, os_pid);
                }
            }
        }

        true
    }

    /// Boost process priority
    pub fn boost_process_priority(&self, pid: Pid) -> Result<Priority, String> {
        let current_priority = self.processes
            .get(&pid)
            .map(|r| r.value().priority)
            .ok_or_else(|| format!("Process {} not found", pid))?;

        let new_priority = crate::scheduler::apply_priority_op(
            current_priority,
            crate::scheduler::PriorityOp::Boost,
        )?;

        if self.set_process_priority(pid, new_priority) {
            Ok(new_priority)
        } else {
            Err("Failed to update priority".to_string())
        }
    }

    /// Lower process priority
    pub fn lower_process_priority(&self, pid: Pid) -> Result<Priority, String> {
        let current_priority = self.processes
            .get(&pid)
            .map(|r| r.value().priority)
            .ok_or_else(|| format!("Process {} not found", pid))?;

        let new_priority = crate::scheduler::apply_priority_op(
            current_priority,
            crate::scheduler::PriorityOp::Lower,
        )?;

        if self.set_process_priority(pid, new_priority) {
            Ok(new_priority)
        } else {
            Err("Failed to update priority".to_string())
        }
    }

    /// Set scheduler time quantum (requires scheduler)
    pub fn set_time_quantum(&self, quantum_micros: u64) -> Result<(), String> {
        let quantum = std::time::Duration::from_micros(quantum_micros);

        // Validate using scheduler's time quantum type
        crate::scheduler::TimeQuantum::new(quantum_micros)?;

        if let Some(ref scheduler) = self.scheduler {
            scheduler.read().set_quantum(quantum);

            // Update the scheduler task's interval dynamically
            if let Some(ref task) = self.scheduler_task {
                task.update_quantum(quantum_micros);
            }

            info!("Time quantum updated to {} microseconds", quantum_micros);
            Ok(())
        } else {
            Err("Scheduler not available".to_string())
        }
    }

    /// Get scheduler task for advanced control (pause/resume/trigger)
    pub fn scheduler_task(&self) -> Option<&Arc<SchedulerTask>> {
        self.scheduler_task.as_ref()
    }
}

impl Clone for ProcessManager {
    fn clone(&self) -> Self {
        Self {
            processes: Arc::clone(&self.processes),
            next_pid: AtomicU32::new(self.next_pid.load(Ordering::SeqCst)),
            memory_manager: self.memory_manager.clone(),
            executor: self.executor.clone(),
            limit_manager: None, // Limit manager is not Clone, create new if needed
            ipc_manager: self.ipc_manager.clone(),
            scheduler: self.scheduler.clone(),
            scheduler_task: self.scheduler_task.clone(),
            child_counts: Arc::clone(&self.child_counts),
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
