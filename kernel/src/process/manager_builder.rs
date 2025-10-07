/*!
 * Process Manager Builder
 * Builder pattern for ProcessManager construction
 */

use super::manager::ProcessManager;
use super::preemption::PreemptionController;
use super::scheduler::Scheduler;
use super::scheduler_task::SchedulerTask;
use super::types::SchedulingPolicy;
use crate::ipc::IPCManager;
use crate::memory::MemoryManager;
use crate::process::executor::ProcessExecutor;
use crate::security::LimitManager;
use ahash::RandomState;
use dashmap::DashMap;
use log::info;
use parking_lot::RwLock;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

/// Builder for ProcessManager
pub struct ProcessManagerBuilder {
    memory_manager: Option<MemoryManager>,
    enable_executor: bool,
    enable_limits: bool,
    ipc_manager: Option<IPCManager>,
    scheduler_policy: Option<SchedulingPolicy>,
    fd_manager: Option<crate::syscalls::fd::FdManager>,
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
            fd_manager: None,
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

    /// Add file descriptor manager for automatic FD cleanup
    pub fn with_fd_manager(mut self, fd_manager: crate::syscalls::fd::FdManager) -> Self {
        self.fd_manager = Some(fd_manager);
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
        if self.fd_manager.is_some() {
            features.push("FD-cleanup");
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
            fd_manager: self.fd_manager,
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

