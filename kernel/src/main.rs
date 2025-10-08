/*!
 * AI-OS Kernel - Main Entry Point
 *
 * Lightweight microkernel that provides:
 * - Process management
 * - Memory management
 * - IPC with AI service
 * - Hardware abstraction
 */

use std::error::Error;
use tokio::signal;
use tokio::sync::broadcast;
use tracing::info;

use ai_os_kernel::process::resources::{
    FdResource, IpcResource, MappingResource, MemoryResource, ResourceOrchestrator, RingResource,
    SignalResource, SocketResource, TaskResource,
};
use ai_os_kernel::{
    init_simd, init_tracing, AsyncTaskManager, IPCManager, IoUringExecutor, IoUringManager,
    LocalFS, MemFS, MemoryManager, MmapManager, MountManager, ProcessManager, SandboxManager,
    SchedulingPolicy as Policy, SignalManagerImpl, SyscallExecutorWithIpc, ZeroCopyIpc,
};
use std::sync::Arc;

/// Wait for shutdown signals (SIGTERM/SIGINT)
async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        match signal::ctrl_c().await {
            Ok(()) => (),
            Err(e) => {
                tracing::error!(error = %e, "Failed to install Ctrl+C handler");
            }
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to install SIGTERM handler");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        },
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown...");
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize structured tracing
    init_tracing();

    info!("AgentOS Kernel starting...");
    info!("================================================");

    // Detect SIMD capabilities
    info!("Detecting SIMD capabilities...");
    let simd_caps = init_simd();
    info!(
        "SIMD ready: AVX-512={}, AVX2={}, SSE2={}, NEON={}, max_vector={}B",
        simd_caps.has_avx512_full(),
        simd_caps.avx2,
        simd_caps.sse2,
        simd_caps.neon,
        simd_caps.max_vector_bytes()
    );

    // Initialize monitoring
    info!("Initializing performance monitoring...");
    let metrics_collector = std::sync::Arc::new(ai_os_kernel::MetricsCollector::new());
    info!("Metrics collector initialized");

    // Initialize kernel subsystems
    info!("Initializing memory manager...");
    let memory_manager = MemoryManager::new();

    info!("Initializing IPC system with memory management...");
    let ipc_manager = IPCManager::new(memory_manager.clone());

    info!("Initializing sandbox manager...");
    let sandbox_manager = SandboxManager::new();

    info!("Initializing VFS with default mount points...");
    let vfs = MountManager::new();

    // Mount local filesystem at /storage for persistent data
    let storage_path =
        std::env::var("KERNEL_STORAGE_PATH").unwrap_or_else(|_| "/tmp/ai-os-storage".to_string());
    info!(storage_path = %storage_path, "Mounting local filesystem at /storage");
    if let Err(e) = std::fs::create_dir_all(&storage_path) {
        tracing::warn!(error = %e, "Could not create storage directory");
    }
    if let Err(e) = vfs.mount("/storage", Arc::new(LocalFS::new(&storage_path))) {
        tracing::error!(error = %e, "Failed to mount /storage");
        return Err("Failed to mount /storage filesystem".into());
    }

    // Mount in-memory filesystem at /tmp (100MB limit)
    info!("Mounting in-memory filesystem at /tmp (100MB limit)");
    if let Err(e) = vfs.mount(
        "/tmp",
        Arc::new(MemFS::with_capacity(
            ai_os_kernel::core::limits::TMP_FILESYSTEM_CAPACITY,
        )),
    ) {
        tracing::error!(error = %e, "Failed to mount /tmp");
        return Err("Failed to mount /tmp filesystem".into());
    }

    // Mount in-memory filesystem at /cache (50MB limit)
    info!("Mounting in-memory filesystem at /cache (50MB limit)");
    if let Err(e) = vfs.mount(
        "/cache",
        Arc::new(MemFS::with_capacity(
            ai_os_kernel::core::limits::CACHE_FILESYSTEM_CAPACITY,
        )),
    ) {
        tracing::error!(error = %e, "Failed to mount /cache");
        return Err("Failed to mount /cache filesystem".into());
    }

    info!("Initializing mmap manager with VFS support...");
    let mmap_manager = MmapManager::with_vfs(Arc::new(vfs.clone()));

    info!("Initializing syscall executor with IPC, VFS, and mmap support...");
    let syscall_executor = SyscallExecutorWithIpc::with_ipc_direct(
        sandbox_manager.clone(),
        ipc_manager.pipes().clone(),
        ipc_manager.shm().clone(),
    )
    .with_queues(ipc_manager.queues().clone())
    .with_vfs(vfs)
    .with_mmap(mmap_manager.clone())
    .with_metrics(metrics_collector.clone())
    .build(); // Finalize with handler registry

    // Initialize managers needed for comprehensive resource cleanup
    info!("Initializing resource managers for comprehensive cleanup...");
    let signal_manager = SignalManagerImpl::new();
    let zerocopy_ipc = ZeroCopyIpc::new(memory_manager.clone());
    let iouring_executor = Arc::new(IoUringExecutor::new(syscall_executor.clone()));
    let iouring_manager = IoUringManager::new(iouring_executor);
    let async_task_manager = AsyncTaskManager::new(syscall_executor.clone());

    // Build comprehensive resource cleanup orchestrator
    // Resources are registered in dependency order (LIFO cleanup - first registered = last cleaned)
    info!("Building unified resource cleanup orchestrator...");
    let resource_orchestrator = ResourceOrchestrator::new()
        .register(MemoryResource::new(memory_manager.clone()))       // Freed last
        .register(MappingResource::new(mmap_manager))                // Depends on memory
        .register(IpcResource::new(ipc_manager.clone()))             // Message queues, pipes, shm
        .register(TaskResource::new(async_task_manager))             // Async tasks
        .register(
            RingResource::new()
                .with_zerocopy(zerocopy_ipc)
                .with_iouring(iouring_manager),
        )
        .register(SignalResource::new(signal_manager))               // Signal handlers
        .register(SocketResource::new(
            syscall_executor.socket_manager().clone(),
        ))                                                            // Network sockets
        .register(FdResource::new(syscall_executor.fd_manager().clone())); // Freed first

    // Validate comprehensive coverage
    resource_orchestrator.validate_coverage(&[
        "memory",
        "ipc",
        "mappings",
        "async_tasks",
        "rings",
        "signals",
        "sockets",
        "file_descriptors",
    ]);

    info!(
        "Resource orchestrator initialized with {} types: {:?}",
        resource_orchestrator.resource_count(),
        resource_orchestrator.registered_types()
    );

    // Build process manager with comprehensive cleanup
    info!("Initializing process manager with memory, IPC, scheduler, and comprehensive cleanup...");
    let process_manager = ProcessManager::builder()
        .with_memory_manager(memory_manager.clone())
        .with_ipc_manager(ipc_manager.clone())
        .with_scheduler(Policy::Fair)
        .with_resource_orchestrator(resource_orchestrator)
        .build();

    info!("Kernel initialization complete");
    info!("================================================");
    info!("Kernel entering main loop...");
    info!("Press Ctrl+C to exit");

    // Start gRPC server in parallel with main loop
    let grpc_addr = match "0.0.0.0:50051".parse() {
        Ok(addr) => addr,
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse gRPC address");
            return Err(format!("Failed to parse gRPC address: {}", e).into());
        }
    };
    let grpc_syscall_executor = syscall_executor.clone();
    let grpc_process_manager = process_manager.clone();
    let grpc_sandbox_manager = sandbox_manager.clone();

    info!(addr = %grpc_addr, "Starting gRPC server");

    // Create shutdown broadcast channel for coordinated graceful shutdown
    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let mut grpc_shutdown_rx = shutdown_tx.subscribe();
    let mut monitor_shutdown_rx = shutdown_tx.subscribe();

    // Spawn gRPC server with graceful shutdown support
    let grpc_handle = tokio::spawn(async move {
        use std::time::Duration;
        use tonic::transport::Server;

        let service_impl = ai_os_kernel::KernelServiceImpl::new(
            grpc_syscall_executor,
            grpc_process_manager,
            grpc_sandbox_manager,
        );

        let service = ai_os_kernel::kernel_proto::kernel_service_server::KernelServiceServer::new(
            service_impl,
        );

        // Graceful shutdown future
        let shutdown_future = async move {
            let _ = grpc_shutdown_rx.recv().await;
            info!("gRPC server received shutdown signal");
        };

        match Server::builder()
            .timeout(Duration::from_secs(30))
            .http2_keepalive_interval(Some(Duration::from_secs(60)))
            .http2_keepalive_timeout(Some(Duration::from_secs(20)))
            .http2_adaptive_window(Some(true))
            .tcp_nodelay(true)
            .add_service(service)
            .serve_with_shutdown(grpc_addr, shutdown_future)
            .await
        {
            Ok(_) => info!("gRPC server shut down gracefully"),
            Err(e) => tracing::error!(error = %e, "gRPC server error"),
        }
    });

    info!("gRPC server started");
    info!("Kernel is ready to receive syscalls from AI service");

    // Clone managers for shutdown
    let shutdown_process_manager = process_manager.clone();

    // Spawn monitoring task with graceful shutdown
    let monitor_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = monitor_shutdown_rx.recv() => {
                    info!("Monitoring task received shutdown signal");
                    break;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                    info!("Kernel running - press Ctrl+C to exit");
                }
            }
        }
    });

    // Wait for shutdown signal
    wait_for_shutdown_signal().await;

    // Begin graceful shutdown sequence
    info!("================================================");
    info!("Starting graceful shutdown...");
    info!("================================================");

    // 1. Broadcast shutdown signal to all tasks
    info!("Broadcasting shutdown signal to background tasks...");
    let _ = shutdown_tx.send(());

    // 2. Allow brief drain period for in-flight requests to complete
    info!("Draining in-flight requests (2 second grace period)...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 3. Terminate all processes with resource cleanup (with timeout per process)
    info!("Terminating all processes with resource cleanup...");
    let processes = shutdown_process_manager.list_processes();
    let process_count = processes.len();

    if process_count > 0 {
        let mut termination_tasks = Vec::new();

        for process in processes {
            let pm = shutdown_process_manager.clone();
            let task = tokio::spawn(async move {
                let pid = process.pid;
                let name = process.name.clone();

                // Give each process 5 seconds to terminate
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(5),
                    tokio::task::spawn_blocking(move || pm.terminate_process(pid)),
                )
                .await
                {
                    Ok(Ok(true)) => {
                        info!("Terminated process {} (PID: {})", name, pid);
                        true
                    }
                    Ok(Ok(false)) => {
                        tracing::warn!("Process {} (PID: {}) was already terminated", name, pid);
                        false
                    }
                    Ok(Err(e)) => {
                        tracing::error!(error = ?e, "Error in spawn_blocking for PID {}", pid);
                        false
                    }
                    Err(_) => {
                        tracing::warn!(
                            "Process {} (PID: {}) termination timed out after 5s",
                            name,
                            pid
                        );
                        false
                    }
                }
            });
            termination_tasks.push(task);
        }

        // Wait for all terminations to complete (parallel)
        let results = futures::future::join_all(termination_tasks).await;
        let successful = results
            .into_iter()
            .filter(|r| r.is_ok() && *r.as_ref().expect("filtered by is_ok()"))
            .count();
        info!(
            "Successfully terminated {}/{} processes",
            successful, process_count
        );
    } else {
        info!("No processes to terminate");
    }

    // 4. IPC resources already cleaned by process termination
    info!("IPC resources cleaned up automatically during process termination");

    // 5. Wait for background tasks to complete (with timeout)
    info!("Waiting for background tasks to complete...");
    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(10), async {
        let _ = tokio::join!(grpc_handle, monitor_handle);
    })
    .await;
    info!("Background tasks completed");

    info!("================================================");
    info!("Graceful shutdown complete. Goodbye!");
    info!("================================================");

    Ok(())
}
