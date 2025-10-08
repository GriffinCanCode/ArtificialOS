/*!
 * Syscall Executor
 * Central executor for all syscalls with sandboxing
 */

use crate::core::types::Pid;
use crate::monitoring::{MetricsCollector, span_syscall};
use crate::permissions::PermissionManager;
use crate::security::SandboxManager;
use tracing::{error, info};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

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
    pub(super) fd_manager: super::fd::FdManager,
    pub(super) socket_manager: super::network::SocketManager,
    handler_registry: SyscallHandlerRegistry,
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
            fd_manager: super::fd::FdManager::new(),
            socket_manager: super::network::SocketManager::new(),
            handler_registry: SyscallHandlerRegistry::new(),
        };

        executor.handler_registry = Self::build_handler_registry(&executor);
        executor
    }

    pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.metrics = Some(metrics);
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

        let mut executor = Self {
            sandbox_manager,
            permission_manager,
            pipe_manager: Some(pipe_manager),
            shm_manager: Some(shm_manager),
            queue_manager: None,
            mmap_manager: None,
            process_manager: None,
            memory_manager: None,
            signal_manager: None,
            vfs: None,
            metrics: None,
            fd_manager: super::fd::FdManager::new(),
            socket_manager: super::network::SocketManager::new(),
            handler_registry: SyscallHandlerRegistry::new(),
        };

        executor.handler_registry = Self::build_handler_registry(&executor);
        executor
    }

    pub fn with_queues(mut self, queue_manager: crate::ipc::QueueManager) -> Self {
        self.queue_manager = Some(queue_manager);
        info!("Queue support enabled for syscall executor");
        self.handler_registry = Self::build_handler_registry(&self);
        self
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
            fd_manager: super::fd::FdManager::new(),
            socket_manager: super::network::SocketManager::new(),
            handler_registry: SyscallHandlerRegistry::new(),
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

        // Dispatch to appropriate handler via registry
        let result = self.handler_registry
            .dispatch(pid, &syscall)
            .unwrap_or_else(|| {
                error!("No handler found for syscall: {:?}", syscall);
                SyscallResult::error(format!("Unhandled syscall: {}", syscall_name))
            });

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
