/*!
 * Process Management
 * Handles process creation, scheduling, and lifecycle
 */

use super::cleanup;
use super::preemption::PreemptionController;
use super::priority;
use super::resources::ResourceOrchestrator;
use super::scheduler::Scheduler;
use super::scheduler_task::SchedulerTask;
use super::types::{ExecutionConfig, ProcessInfo, ProcessState};
use crate::core::types::{Pid, Priority};
use crate::ipc::IPCManager;
use crate::memory::MemoryManager;
use crate::process::executor::ProcessExecutor;
use crate::security::LimitManager;
use ahash::RandomState;
use dashmap::DashMap;
use log::info;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// Type alias for backwards compatibility
pub type Process = ProcessInfo;

/// Process manager
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic PID counter (extremely hot path)
#[repr(C, align(64))]
pub struct ProcessManager {
    pub(super) processes: Arc<DashMap<Pid, ProcessInfo, RandomState>>,
    pub(super) next_pid: AtomicU32,
    pub(super) memory_manager: Option<MemoryManager>,
    pub(super) executor: Option<ProcessExecutor>,
    pub(super) limit_manager: Option<LimitManager>,
    pub(super) ipc_manager: Option<IPCManager>,
    pub(super) scheduler: Option<Arc<RwLock<Scheduler>>>,
    pub(super) scheduler_task: Option<Arc<SchedulerTask>>,
    pub(super) preemption: Option<Arc<PreemptionController>>,
    pub(super) fd_manager: Option<crate::syscalls::fd::FdManager>,
    // Comprehensive resource cleanup orchestrator
    pub(super) resource_orchestrator: Option<ResourceOrchestrator>,
    // Track child processes per parent PID for limit enforcement
    pub(super) child_counts: Arc<DashMap<Pid, u32, RandomState>>,
}

impl ProcessManager {
    /// Create a basic ProcessManager with no features
    pub fn new() -> Self {
        info!("Process manager initialized (basic)");
        Self {
            // Use 128 shards for processes - high contention from concurrent process operations
            processes: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                128,
            )),
            next_pid: AtomicU32::new(1),
            memory_manager: None,
            executor: None,
            limit_manager: None,
            ipc_manager: None,
            scheduler: None,
            scheduler_task: None,
            preemption: None,
            fd_manager: None,
            resource_orchestrator: None,
            // Use 64 shards for child_counts (moderate contention)
            child_counts: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                64,
            )),
        }
    }

    /// Create a builder for constructing a ProcessManager
    pub fn builder() -> super::manager_builder::ProcessManagerBuilder {
        super::manager_builder::ProcessManagerBuilder::new()
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
                        info!(
                            "Spawned OS process {} for PID {} with resource limits",
                            os_pid, pid
                        );

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

    /// Terminate process by PID
    pub fn terminate_process(&self, pid: Pid) -> bool {
        if let Some((_, process)) = self.processes.remove(&pid) {
            info!("Terminating process: PID {}", pid);

            // Core cleanup (OS process, memory, IPC, scheduler)
            cleanup::cleanup_os_process(&process, pid, &self.executor, &self.limit_manager);
            cleanup::cleanup_memory(pid, &self.memory_manager);
            cleanup::cleanup_ipc(pid, &self.ipc_manager);
            cleanup::cleanup_scheduler(pid, &self.scheduler);
            cleanup::cleanup_preemption(pid, &self.preemption);
            cleanup::cleanup_file_descriptors(pid, &self.fd_manager);

            // Comprehensive resource cleanup (sockets, signals, rings, tasks, mappings)
            cleanup::cleanup_comprehensive(pid, &self.resource_orchestrator);

            true
        } else {
            false
        }
    }

    /// List all processes
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
    #[allow(dead_code)]
    pub(super) fn increment_child_count(&self, pid: Pid) {
        // Use alter() for atomic increment
        self.child_counts.alter(&pid, |_, count| count + 1);
    }

    /// Decrement child count for a PID
    #[allow(dead_code)]
    pub(super) fn decrement_child_count(&self, pid: Pid) {
        // Use alter() for atomic decrement
        self.child_counts
            .alter(&pid, |_, count| count.saturating_sub(1));
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
            fd_manager: self.fd_manager.clone(),
            resource_orchestrator: None, // Resource orchestrator is not Clone
            child_counts: Arc::clone(&self.child_counts),
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
