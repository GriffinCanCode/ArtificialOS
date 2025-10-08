/*!
 * Process Manager Builder
 * Builder pattern for ProcessManager construction
 */

use super::manager::ProcessManager;
use crate::core::{ShardManager, WorkloadProfile};
use crate::ipc::IPCManager;
use crate::memory::MemoryManager;
use crate::monitoring::Collector;
use crate::process::core::types::SchedulingPolicy;
use crate::process::execution::{PreemptionController, ProcessExecutor};
use crate::process::lifecycle::LifecycleRegistry;
use crate::process::resources::ResourceOrchestrator;
use crate::process::scheduler::{Scheduler, SchedulerTask};
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
    fd_manager: Option<crate::syscalls::impls::fd::FdManager>,
    resource_orchestrator: ResourceOrchestrator,
    signal_manager: Option<Arc<crate::signals::SignalManagerImpl>>,
    collector: Option<Arc<Collector>>,
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
            resource_orchestrator: ResourceOrchestrator::new(),
            signal_manager: None,
            collector: None,
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
    pub fn with_fd_manager(mut self, fd_manager: crate::syscalls::impls::fd::FdManager) -> Self {
        self.fd_manager = Some(fd_manager);
        self
    }

    /// Add resource orchestrator for comprehensive cleanup
    pub fn with_resource_orchestrator(mut self, orchestrator: ResourceOrchestrator) -> Self {
        self.resource_orchestrator = orchestrator;
        self
    }

    /// Add signal manager for lifecycle initialization
    pub fn with_signal_manager(
        mut self,
        signal_manager: Arc<crate::signals::SignalManagerImpl>,
    ) -> Self {
        self.signal_manager = Some(signal_manager);
        self
    }

    /// Add observability collector for event streaming
    pub fn with_collector(mut self, collector: Arc<Collector>) -> Self {
        self.collector = Some(collector);
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

        // Auto-register resources if orchestrator is empty
        let orchestrator = if self.resource_orchestrator.resource_count() == 0 {
            let mut orch = self.resource_orchestrator;

            // Register memory if available
            if let Some(ref mem_mgr) = self.memory_manager {
                orch = orch.register(crate::process::resources::MemoryResource::new(
                    mem_mgr.clone(),
                ));
            }

            // Register IPC if available
            if let Some(ref ipc_mgr) = self.ipc_manager {
                orch = orch.register(crate::process::resources::IpcResource::new(ipc_mgr.clone()));
            }

            // Register FD if available
            if let Some(ref fd_mgr) = self.fd_manager {
                orch = orch.register(crate::process::resources::FdResource::new(fd_mgr.clone()));
            }

            orch
        } else {
            self.resource_orchestrator
        };

        let scheduler = self
            .scheduler_policy
            .map(|policy| Arc::new(RwLock::new(Scheduler::new(policy))));

        // Create preemption controller if both scheduler and executor are available
        let preemption = match (&scheduler, &executor) {
            (Some(sched), Some(exec)) => Some(Arc::new(PreemptionController::new(
                Arc::clone(sched),
                Arc::new(exec.clone()),
            ))),
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
        if orchestrator.resource_count() > 0 {
            features.push("unified-resource-cleanup");
        }

        // Build lifecycle registry if we have relevant managers
        // This coordinates initialization hooks across subsystems
        let lifecycle = if self.signal_manager.is_some()
            || self.ipc_manager.is_some()
            || self.fd_manager.is_some()
        {
            let mut registry = LifecycleRegistry::new();

            // Register signal manager if available
            if let Some(ref signal_mgr) = self.signal_manager {
                registry = registry.with_signal_manager(Arc::clone(signal_mgr));
            }

            // Register zero-copy IPC if we have IPC manager
            if let Some(ref ipc_mgr) = self.ipc_manager {
                if let Some(zerocopy) = ipc_mgr.zerocopy() {
                    registry = registry.with_zerocopy_ipc(Arc::new(zerocopy.clone()));
                }
            }

            // Register FD manager if available
            if let Some(ref fd_mgr) = self.fd_manager {
                registry = registry.with_fd_manager(Arc::new(fd_mgr.clone()));
            }

            features.push("lifecycle-hooks");
            Some(registry)
        } else {
            None
        };

        info!("Process manager initialized with: {}", features.join(", "));

        ProcessManager {
            // CPU-topology-aware shard counts for optimal concurrent performance
            processes: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                ShardManager::shards(WorkloadProfile::HighContention), // process table: heavy concurrent access
            )),
            next_pid: Arc::new(AtomicU32::new(1)),
            memory_manager: self.memory_manager,
            executor,
            limit_manager,
            ipc_manager: self.ipc_manager,
            scheduler,
            scheduler_task,
            preemption,
            fd_manager: self.fd_manager,
            resource_orchestrator: orchestrator,
            child_counts: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                ShardManager::shards(WorkloadProfile::MediumContention), // child tracking: moderate access
            )),
            lifecycle,
            collector: self.collector,
        }
    }
}

impl Default for ProcessManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
