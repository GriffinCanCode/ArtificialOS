/*!
 * Process Management
 * Handles process creation, scheduling, and lifecycle
 */

use super::cleanup;
use super::preemption::PreemptionController;
use super::priority;
use super::types::{ExecutionConfig, ProcessInfo, ProcessState, SchedulingPolicy};
use crate::core::types::{Pid, Priority};
use crate::ipc::IPCManager;
use crate::memory::MemoryManager;
use crate::process::executor::ProcessExecutor;
use crate::process::scheduler::Scheduler;
use crate::process::scheduler_task::SchedulerTask;
use crate::security::LimitManager;
use ahash::RandomState;
use dashmap::DashMap;
use log::info;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// Type alias for backwards compatibility
pub type Process = ProcessInfo;

pub struct ProcessManager {
    processes: Arc<DashMap<Pid, ProcessInfo, RandomState>>,
    next_pid: AtomicU32,
    memory_manager: Option<MemoryManager>,
    executor: Option<ProcessExecutor>,
    limit_manager: Option<LimitManager>,
    ipc_manager: Option<IPCManager>,
    scheduler: Option<Arc<RwLock<Scheduler>>>,
    scheduler_task: Option<Arc<SchedulerTask>>,
    preemption: Option<Arc<PreemptionController>>,
    // Track child processes per parent PID for limit enforcement
    child_counts: Arc<DashMap<Pid, u32, RandomState>>,
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

        // Create preemption controller if both scheduler and executor are available
        let preemption = match (&scheduler, &executor) {
            (Some(sched), Some(exec)) => {
                Some(Arc::new(PreemptionController::new(
                    Arc::clone(sched),
                    Arc::new(exec.clone()),
                )))
            }
            _ => None,
        };

        // Spawn autonomous scheduler task if scheduler is enabled
        let scheduler_task = scheduler.as_ref().map(|sched| {
            Arc::new(SchedulerTask::spawn_with_preemption(
                Arc::clone(sched),
                preemption.clone(),
            ))
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
        if preemption.is_some() {
            features.push("OS-preemption");
        }
        if scheduler_task.is_some() {
            features.push("autonomous-scheduling");
        }

        info!("Process manager initialized with: {}", features.join(", "));

        ProcessManager {
            // Use 128 shards for processes - high contention from concurrent process operations
            processes: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(0, RandomState::new(), 128)),
            next_pid: AtomicU32::new(1),
            memory_manager: self.memory_manager,
            executor,
            limit_manager,
            ipc_manager: self.ipc_manager,
            scheduler,
            scheduler_task,
            preemption,
            // Use 64 shards for child_counts (moderate contention)
            child_counts: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(0, RandomState::new(), 64)),
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
            // Use 128 shards for processes - high contention from concurrent process operations
            processes: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(0, RandomState::new(), 128)),
            next_pid: AtomicU32::new(1),
            memory_manager: None,
            executor: None,
            limit_manager: None,
            ipc_manager: None,
            scheduler: None,
            scheduler_task: None,
            preemption: None,
            // Use 64 shards for child_counts (moderate contention)
            child_counts: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(0, RandomState::new(), 64)),
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
        let os_pid = if let Some(mut cfg) = config {
            if let Some(ref executor) = self.executor {
                // Calculate resource limits based on priority BEFORE spawning
                // This ensures limits are applied atomically during spawn
                let limits = priority::priority_to_limits(priority);

                // Add limits to config so they're applied in pre-exec hook
                cfg = cfg.with_limits(limits.clone());

                match executor.spawn(pid, name.clone(), cfg) {
                    Ok(os_pid) => {
                        info!("Spawned OS process {} for PID {} with resource limits", os_pid, pid);

                        // Apply cgroup limits as a fallback/supplement
                        // Cgroups provide additional controls (CPU shares) not available via rlimits
                        // This happens AFTER spawn but the critical limits are already enforced
                        if let Some(ref limit_mgr) = self.limit_manager {
                            if let Err(e) = limit_mgr.apply(os_pid, &limits) {
                                log::warn!("Failed to apply cgroup limits (non-critical): {}", e);
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

    /// Get process by PID
    ///
    /// # Performance
    /// Hot path - frequently called for process lookups
    #[inline]
    #[must_use]
    pub fn get_process(&self, pid: Pid) -> Option<ProcessInfo> {
        self.processes.get(&pid).map(|r| r.value().clone())
    }

    pub fn terminate_process(&self, pid: Pid) -> bool {
        if let Some((_, process)) = self.processes.remove(&pid) {
            info!("Terminating process: PID {}", pid);

            cleanup::cleanup_os_process(&process, pid, &self.executor, &self.limit_manager);
            cleanup::cleanup_memory(pid, &self.memory_manager);
            cleanup::cleanup_ipc(pid, &self.ipc_manager);
            cleanup::cleanup_scheduler(pid, &self.scheduler);
            cleanup::cleanup_preemption(pid, &self.preemption);

            true
        } else {
            false
        }
    }

    pub fn list_processes(&self) -> Vec<ProcessInfo> {
        self.processes.iter().map(|r| r.value().clone()).collect()
    }

    /// Get memory manager reference
    ///
    /// # Performance
    /// Hot path - frequently accessed for memory operations
    #[inline(always)]
    #[must_use]
    pub fn memory_manager(&self) -> Option<&MemoryManager> {
        self.memory_manager.as_ref()
    }

    /// Get executor reference
    ///
    /// # Performance
    /// Hot path - frequently accessed for process execution
    #[inline(always)]
    #[must_use]
    pub fn executor(&self) -> Option<&ProcessExecutor> {
        self.executor.as_ref()
    }

    /// Check if process has OS execution
    ///
    /// # Performance
    /// Hot path - frequently checked during process operations
    #[inline]
    #[must_use]
    pub fn has_os_process(&self, pid: Pid) -> bool {
        self.processes
            .get(&pid)
            .and_then(|r| r.value().os_pid)
            .is_some()
    }

    /// Get child process count for a PID
    ///
    /// # Performance
    /// Hot path - frequently checked for resource limits
    #[inline]
    #[must_use]
    pub fn get_child_count(&self, pid: Pid) -> u32 {
        self.child_counts.get(&pid).map(|r| *r.value()).unwrap_or(0)
    }

    /// Increment child count for a PID
    fn increment_child_count(&self, pid: Pid) {
        // Use alter() for atomic increment
        self.child_counts.alter(&pid, |_, count| count + 1);
    }

    /// Decrement child count for a PID
    fn decrement_child_count(&self, pid: Pid) {
        // Use alter() for atomic decrement
        self.child_counts
            .alter(&pid, |_, count| count.saturating_sub(1));
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
                let new_limits = priority::priority_to_limits(new_priority);
                if let Err(e) = limit_mgr.apply(os_pid, &new_limits) {
                    log::warn!("Failed to update resource limits for PID {}: {}", pid, e);
                } else {
                    info!(
                        "Updated resource limits for PID {} (OS PID {})",
                        pid, os_pid
                    );
                }
            }
        }

        true
    }

    /// Boost process priority
    pub fn boost_process_priority(&self, pid: Pid) -> Result<Priority, String> {
        let current_priority = self
            .processes
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
        let current_priority = self
            .processes
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
            preemption: self.preemption.clone(),
            child_counts: Arc::clone(&self.child_counts),
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
