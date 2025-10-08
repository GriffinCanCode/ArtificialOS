/*!
 * Syscall Executor with TypeState Pattern
 * Eliminates runtime Option checks through compile-time state enforcement
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
use crate::syscalls::types::{Syscall, SyscallResult};

/// Global system start time for uptime tracking
pub static SYSTEM_START: OnceLock<Instant> = OnceLock::new();

// ============================================================================
// TypeState Pattern - Simplified Design
// ============================================================================
// The TypeState pattern is implemented by having separate types:
// - Basic executors can be created without IPC (not exposed publicly)
// - SyscallExecutorWithIpc is the full-featured public executor

// ============================================================================
// Manager Groupings (Avoid Option anti-pattern)
// ============================================================================

/// IPC managers bundled together (always present when IPC is enabled)
#[derive(Clone)]
pub struct IpcManagers {
    pub pipe_manager: crate::ipc::PipeManager,
    pub shm_manager: crate::ipc::ShmManager,
    pub queue_manager: Option<crate::ipc::QueueManager>, // Truly optional feature
    pub mmap_manager: Option<crate::ipc::MmapManager>,   // Truly optional feature
}

impl IpcManagers {
    /// Get reference to pipe manager
    pub fn pipe_manager(&self) -> &crate::ipc::PipeManager {
        &self.pipe_manager
    }

    /// Get reference to shared memory manager
    pub fn shm_manager(&self) -> &crate::ipc::ShmManager {
        &self.shm_manager
    }

    /// Get reference to queue manager (if enabled)
    pub fn queue_manager(&self) -> &Option<crate::ipc::QueueManager> {
        &self.queue_manager
    }

    /// Get reference to mmap manager (if enabled)
    pub fn mmap_manager(&self) -> &Option<crate::ipc::MmapManager> {
        &self.mmap_manager
    }
}

/// Optional feature managers (legitimately optional)
#[derive(Clone)]
pub struct OptionalManagers {
    pub process_manager: Option<crate::process::ProcessManagerImpl>,
    pub memory_manager: Option<crate::memory::MemoryManager>,
    pub signal_manager: Option<crate::signals::SignalManagerImpl>,
    pub vfs: Option<crate::vfs::MountManager>,
    pub metrics: Option<Arc<MetricsCollector>>,
    pub collector: Option<Arc<Collector>>,
}

impl Default for OptionalManagers {
    fn default() -> Self {
        Self {
            process_manager: None,
            memory_manager: None,
            signal_manager: None,
            vfs: None,
            metrics: None,
            collector: None,
        }
    }
}

// ============================================================================
// Specialized Executor with IPC
// ============================================================================

/// Executor with IPC managers (no Option checks needed!)
#[repr(C, align(64))]
pub struct SyscallExecutorWithIpc {
    // Core managers (always present)
    pub(super) sandbox_manager: SandboxManager,
    pub(super) permission_manager: PermissionManager,
    pub(super) fd_manager: crate::syscalls::impls::fd::FdManager,
    pub(super) socket_manager: crate::syscalls::impls::network::SocketManager,
    pub(super) timeout_executor: crate::syscalls::timeout::executor::TimeoutExecutor,
    pub(super) timeout_config: crate::syscalls::timeout::config::SyscallTimeoutConfig,

    // Handler registry
    handler_registry: SyscallHandlerRegistry,

    // IPC managers (GUARANTEED present, no Option!)
    pub(super) ipc: IpcManagers,

    // Optional managers
    pub(super) optional: OptionalManagers,
}

// ============================================================================
// Clone implementation
// ============================================================================

impl Clone for SyscallExecutorWithIpc {
    fn clone(&self) -> Self {
        Self {
            sandbox_manager: self.sandbox_manager.clone(),
            permission_manager: self.permission_manager.clone(),
            fd_manager: self.fd_manager.clone(),
            socket_manager: self.socket_manager.clone(),
            timeout_executor: self.timeout_executor.clone(),
            timeout_config: self.timeout_config.clone(),
            handler_registry: self.handler_registry.clone(),
            ipc: self.ipc.clone(),
            optional: self.optional.clone(),
        }
    }
}

// ============================================================================
// IPC-Enabled Executor - Construction and Builders
// ============================================================================

impl SyscallExecutorWithIpc {
    /// Create executor directly with IPC support (alternative constructor)
    pub fn with_ipc_direct(
        sandbox_manager: SandboxManager,
        pipe_manager: crate::ipc::PipeManager,
        shm_manager: crate::ipc::ShmManager,
    ) -> Self {
        // Initialize system start time
        SYSTEM_START.get_or_init(Instant::now);

        let permission_manager = PermissionManager::new(sandbox_manager.clone());
        info!("Syscall executor initialized with IPC support");

        let executor = Self {
            sandbox_manager,
            permission_manager,
            fd_manager: crate::syscalls::impls::fd::FdManager::new(),
            socket_manager: crate::syscalls::impls::network::SocketManager::new(),
            timeout_executor: crate::syscalls::timeout::executor::TimeoutExecutor::disabled(),
            timeout_config: crate::syscalls::timeout::config::SyscallTimeoutConfig::new(),
            handler_registry: SyscallHandlerRegistry::new(),
            ipc: IpcManagers {
                pipe_manager,
                shm_manager,
                queue_manager: None,
                mmap_manager: None,
            },
            optional: OptionalManagers::default(),
        };

        executor
    }

    /// Create executor with full features (legacy compatibility)
    pub fn with_full_features(
        sandbox_manager: SandboxManager,
        pipe_manager: crate::ipc::PipeManager,
        shm_manager: crate::ipc::ShmManager,
        process_manager: crate::process::ProcessManagerImpl,
        memory_manager: crate::memory::MemoryManager,
    ) -> Self {
        SYSTEM_START.get_or_init(Instant::now);

        let permission_manager = PermissionManager::new(sandbox_manager.clone());
        info!("Syscall executor initialized with full features");

        Self {
            sandbox_manager,
            permission_manager,
            fd_manager: crate::syscalls::impls::fd::FdManager::new(),
            socket_manager: crate::syscalls::impls::network::SocketManager::new(),
            timeout_executor: crate::syscalls::timeout::executor::TimeoutExecutor::disabled(),
            timeout_config: crate::syscalls::timeout::config::SyscallTimeoutConfig::default(),
            handler_registry: SyscallHandlerRegistry::new(),
            ipc: IpcManagers {
                pipe_manager,
                shm_manager,
                queue_manager: None,
                mmap_manager: None,
            },
            optional: OptionalManagers {
                process_manager: Some(process_manager),
                memory_manager: Some(memory_manager),
                signal_manager: None,
                vfs: None,
                metrics: None,
                collector: None,
            },
        }
    }

    /// Add queue manager support
    pub fn with_queues(mut self, queue_manager: crate::ipc::QueueManager) -> Self {
        self.ipc.queue_manager = Some(queue_manager);
        info!("Queue support enabled");
        self
    }

    /// Add mmap manager support
    pub fn with_mmap(mut self, mmap_manager: crate::ipc::MmapManager) -> Self {
        self.ipc.mmap_manager = Some(mmap_manager);
        info!("Mmap support enabled");
        self
    }

    /// Set VFS mount manager
    pub fn with_vfs(mut self, vfs: crate::vfs::MountManager) -> Self {
        self.optional.vfs = Some(vfs);
        info!("VFS enabled");
        self
    }

    /// Add signal manager support
    pub fn with_signals(mut self, signal_manager: crate::signals::SignalManagerImpl) -> Self {
        self.optional.signal_manager = Some(signal_manager);
        info!("Signal support enabled");
        self
    }

    /// Add metrics collector
    pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.optional.metrics = Some(metrics);
        self
    }

    /// Add observability collector
    pub fn with_collector(mut self, collector: Arc<Collector>) -> Self {
        self.optional.collector = Some(collector.clone());
        // Enable timeout executor with observability
        if self.timeout_config.enabled {
            use crate::monitoring::TimeoutObserver;
            let observer = Arc::new(TimeoutObserver::new(collector));
            self.timeout_executor =
                crate::syscalls::timeout::executor::TimeoutExecutor::new(Some(observer));
        }
        self
    }

    /// Set timeout configuration
    pub fn with_timeout_config(
        mut self,
        config: crate::syscalls::timeout::config::SyscallTimeoutConfig,
    ) -> Self {
        self.timeout_config = config;
        info!("Custom timeout configuration applied");
        self
    }

    /// Finalize executor with handler registry
    pub fn build(mut self) -> Self {
        self.handler_registry = Self::build_handler_registry(&self);
        info!("Syscall executor built with handler registry");
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
}

// ============================================================================
// Common Methods
// ============================================================================

impl SyscallExecutorWithIpc {
    /// Get reference to socket manager
    pub fn socket_manager(&self) -> &crate::syscalls::impls::network::SocketManager {
        &self.socket_manager
    }

    /// Get mutable reference to socket manager
    pub fn socket_manager_mut(&mut self) -> &mut crate::syscalls::impls::network::SocketManager {
        &mut self.socket_manager
    }

    /// Get reference to file descriptor manager
    pub fn fd_manager(&self) -> &crate::syscalls::impls::fd::FdManager {
        &self.fd_manager
    }

    /// Get reference to permission manager
    pub fn permission_manager(&self) -> &PermissionManager {
        &self.permission_manager
    }

    /// Get reference to sandbox manager
    pub fn sandbox_manager(&self) -> &SandboxManager {
        &self.sandbox_manager
    }

    /// Get reference to timeout executor
    pub fn timeout_executor(&self) -> &crate::syscalls::timeout::executor::TimeoutExecutor {
        &self.timeout_executor
    }

    /// Get timeout configuration
    pub fn timeout_config(&self) -> &crate::syscalls::timeout::config::SyscallTimeoutConfig {
        &self.timeout_config
    }

    /// Get reference to IPC managers
    pub fn ipc(&self) -> &IpcManagers {
        &self.ipc
    }

    /// Get reference to optional managers
    pub fn optional(&self) -> &OptionalManagers {
        &self.optional
    }

    /// Execute a system call with sandboxing
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
        if let Some(ref collector) = self.optional.collector {
            let duration_us = start.elapsed().as_micros() as u64;
            let success = matches!(result, SyscallResult::Success { .. });
            collector.syscall_exit(pid, syscall_name.to_string(), duration_us, success);
        }

        // Record result in span
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
