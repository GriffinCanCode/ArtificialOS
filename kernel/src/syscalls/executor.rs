/*!
 * Syscall Executor
 * Central executor for all syscalls with sandboxing
 */

use crate::core::types::Pid;
use crate::monitoring::{span_syscall, Collector, MetricsCollector};
use crate::permissions::PermissionManager;
use crate::security::SandboxManager;
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use tracing::{error, info};

use super::handler::SyscallHandlerRegistry;
use super::handlers::*;
use super::types::{Syscall, SyscallResult};

/// Global system start time for uptime tracking
pub static SYSTEM_START: OnceLock<Instant> = OnceLock::new();

/// System call executor
///
/// # Performance
/// - Cache-line aligned for optimal performance in hot syscall paths
#[repr(C, align(64))]
#[derive(Clone)]
pub struct SyscallExecutor {
    pub(super) sandbox_manager: SandboxManager,
    pub(super) permission_manager: PermissionManager,
    pub(super) pipe_manager: Option<crate::ipc::PipeManager>,
    pub(super) shm_manager: Option<crate::ipc::ShmManager>,
    pub(super) queue_manager: Option<crate::ipc::QueueManager>,
    pub(super) mmap_manager: Option<crate::ipc::MmapManager>,
    pub(super) process_manager: Option<crate::process::ProcessManagerImpl>,
    pub(super) memory_manager: Option<crate::memory::MemoryManager>,
    pub(super) signal_manager: Option<crate::signals::SignalManagerImpl>,
    pub(super) vfs: Option<crate::vfs::MountManager>,
    pub(super) metrics: Option<Arc<MetricsCollector>>,
    pub(super) collector: Option<Arc<Collector>>,
    pub(super) fd_manager: super::fd::FdManager,
    pub(super) socket_manager: super::network::SocketManager,
    handler_registry: SyscallHandlerRegistry,
    pub(super) timeout_pipe_ops: Option<Arc<crate::ipc::TimeoutPipeOps>>,
    pub(super) timeout_queue_ops: Option<Arc<crate::ipc::TimeoutQueueOps>>,
    pub(super) timeout_config: super::timeout_config::SyscallTimeoutConfig,
}

impl SyscallExecutor {
    pub fn new(sandbox_manager: SandboxManager) -> Self {
        // Initialize system start time
        SYSTEM_START.get_or_init(Instant::now);

        let permission_manager = PermissionManager::new(sandbox_manager.clone());
        info!("Syscall executor initialized with centralized permissions");

        let mut executor = Self {
            sandbox_manager,
            permission_manager,
            pipe_manager: None,
            shm_manager: None,
            queue_manager: None,
            mmap_manager: None,
            process_manager: None,
            memory_manager: None,
            signal_manager: None,
            vfs: None,
            metrics: None,
            collector: None,
            fd_manager: super::fd::FdManager::new(),
            socket_manager: super::network::SocketManager::new(),
            handler_registry: SyscallHandlerRegistry::new(),
            timeout_pipe_ops: None,
            timeout_queue_ops: None,
            timeout_config: super::timeout_config::SyscallTimeoutConfig::new(),
        };

        executor.handler_registry = Self::build_handler_registry(&executor);
        executor
    }

    pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_collector(mut self, collector: Arc<Collector>) -> Self {
        self.collector = Some(collector);
        self
    }

    /// Get reference to socket manager (for resource cleanup orchestrator)
    pub fn socket_manager(&self) -> &super::network::SocketManager {
        &self.socket_manager
    }

    pub fn with_ipc(
        sandbox_manager: SandboxManager,
        pipe_manager: crate::ipc::PipeManager,
        shm_manager: crate::ipc::ShmManager,
    ) -> Self {
        // Initialize system start time
        SYSTEM_START.get_or_init(Instant::now);

        let permission_manager = PermissionManager::new(sandbox_manager.clone());
        info!("Syscall executor initialized with IPC support and centralized permissions");

        // Create timeout-aware operations
        let pipe_manager_arc = Arc::new(pipe_manager);
        let timeout_pipe_ops = Arc::new(crate::ipc::TimeoutPipeOps::new(pipe_manager_arc.clone()));

        let mut executor = Self {
            sandbox_manager,
            permission_manager,
            pipe_manager: Some((*pipe_manager_arc).clone()),
            shm_manager: Some(shm_manager),
            queue_manager: None,
            mmap_manager: None,
            process_manager: None,
            memory_manager: None,
            signal_manager: None,
            vfs: None,
            metrics: None,
            collector: None,
            fd_manager: super::fd::FdManager::new(),
            socket_manager: super::network::SocketManager::new(),
            handler_registry: SyscallHandlerRegistry::new(),
            timeout_pipe_ops: Some(timeout_pipe_ops),
            timeout_queue_ops: None,
            timeout_config: super::timeout_config::SyscallTimeoutConfig::new(),
        };

        executor.handler_registry = Self::build_handler_registry(&executor);
        info!("Timeout support enabled for IPC operations");
        executor
    }

    pub fn with_queues(mut self, queue_manager: crate::ipc::QueueManager) -> Self {
        let queue_manager_arc = Arc::new(queue_manager);
        let timeout_queue_ops = Arc::new(crate::ipc::TimeoutQueueOps::new(queue_manager_arc.clone()));

        self.queue_manager = Some((*queue_manager_arc).clone());
        self.timeout_queue_ops = Some(timeout_queue_ops);
        info!("Queue support enabled for syscall executor with timeout support");
        self.handler_registry = Self::build_handler_registry(&self);
        self
    }

    /// Set timeout configuration
    pub fn with_timeout_config(mut self, config: super::timeout_config::SyscallTimeoutConfig) -> Self {
        self.timeout_config = config;
        info!("Custom timeout configuration applied");
        self
    }

    /// Get timeout configuration
    pub fn timeout_config(&self) -> &super::timeout_config::SyscallTimeoutConfig {
        &self.timeout_config
    }

    /// Add signal manager support
    pub fn with_signals(mut self, signal_manager: crate::signals::SignalManagerImpl) -> Self {
        self.signal_manager = Some(signal_manager);
        info!("Signal support enabled for syscall executor");
        self.handler_registry = Self::build_handler_registry(&self);
        self
    }

    pub fn with_full_features(
        sandbox_manager: SandboxManager,
        pipe_manager: crate::ipc::PipeManager,
        shm_manager: crate::ipc::ShmManager,
        process_manager: crate::process::ProcessManagerImpl,
        memory_manager: crate::memory::MemoryManager,
    ) -> Self {
        // Initialize system start time
        SYSTEM_START.get_or_init(Instant::now);

        let permission_manager = PermissionManager::new(sandbox_manager.clone());
        info!("Syscall executor initialized with full features and centralized permissions");

        let mut executor = Self {
            sandbox_manager,
            permission_manager,
            pipe_manager: Some(pipe_manager),
            shm_manager: Some(shm_manager),
            queue_manager: None,
            mmap_manager: None,
            process_manager: Some(process_manager),
            memory_manager: Some(memory_manager),
            signal_manager: None,
            vfs: None,
            metrics: None,
            collector: None,
            fd_manager: super::fd::FdManager::new(),
            socket_manager: super::network::SocketManager::new(),
            handler_registry: SyscallHandlerRegistry::new(),
            timeout_pipe_ops: None,
            timeout_queue_ops: None,
            timeout_config: super::timeout_config::SyscallTimeoutConfig::default(),
        };

        executor.handler_registry = Self::build_handler_registry(&executor);
        executor
    }

    /// Set VFS mount manager
    pub fn with_vfs(mut self, vfs: crate::vfs::MountManager) -> Self {
        self.vfs = Some(vfs);
        info!("VFS enabled for syscall executor");
        self.handler_registry = Self::build_handler_registry(&self);
        self
    }

    /// Add mmap manager support (requires VFS)
    pub fn with_mmap(mut self, mmap_manager: crate::ipc::MmapManager) -> Self {
        self.mmap_manager = Some(mmap_manager);
        info!("Mmap support enabled for syscall executor");
        self.handler_registry = Self::build_handler_registry(&self);
        self
    }

    /// Build the handler registry with all syscall handlers
    fn build_handler_registry(executor: &Self) -> SyscallHandlerRegistry {
        SyscallHandlerRegistry::new()
            .register(Arc::new(FileSystemHandler::new(executor.clone())))
            .register(Arc::new(ProcessHandler::new(executor.clone())))
            .register(Arc::new(SystemHandler::new(executor.clone())))
            .register(Arc::new(IpcHandler::new(executor.clone())))
            .register(Arc::new(MmapHandler::new(executor.clone())))
            .register(Arc::new(SchedulerHandler::new(executor.clone())))
            .register(Arc::new(TimeHandler::new(executor.clone())))
            .register(Arc::new(MemoryHandler::new(executor.clone())))
            .register(Arc::new(SignalHandler::new(executor.clone())))
            .register(Arc::new(NetworkHandler::new(executor.clone())))
            .register(Arc::new(FileDescriptorHandler::new(executor.clone())))
    }

    /// Get a reference to the file descriptor manager
    pub fn fd_manager(&self) -> &super::fd::FdManager {
        &self.fd_manager
    }

    /// Execute a system call with sandboxing
    /// Uses handler registry for low cognitive complexity dispatch
    pub fn execute(&self, pid: Pid, syscall: Syscall) -> SyscallResult {
        // Create a rich structured span for this syscall
        let syscall_name = syscall.name();
        let span = span_syscall(syscall_name, pid);
        let _guard = span.enter();

        info!(
            pid = pid,
            syscall = syscall_name,
            trace_id = %span.trace_id(),
            "Executing syscall"
        );

        // Record syscall details
        span.record_debug("syscall_details", &syscall);

        // Track timing for observability
        let start = Instant::now();

        // Dispatch to appropriate handler via registry
        let result = self
            .handler_registry
            .dispatch(pid, &syscall)
            .unwrap_or_else(|| {
                error!("No handler found for syscall: {:?}", syscall);
                SyscallResult::error(format!("Unhandled syscall: {}", syscall_name))
            });

        // Emit observability event
        if let Some(ref collector) = self.collector {
            let duration_us = start.elapsed().as_micros() as u64;
            let success = matches!(result, SyscallResult::Success { .. });
            collector.syscall_exit(pid, syscall_name.to_string(), duration_us, success);
        }

        // Record result in span for structured tracing
        match &result {
            SyscallResult::Success { data } => {
                span.record_result(true);
                if let Some(d) = data {
                    span.record("data_size", d.len());
                }
            }
            SyscallResult::Error { message } => {
                span.record_error(message);
            }
            SyscallResult::PermissionDenied { reason } => {
                span.record_error(&format!("Permission denied: {}", reason));
            }
        }

        result
    }
}
