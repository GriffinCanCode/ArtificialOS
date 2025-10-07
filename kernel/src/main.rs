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
use tracing::info;

use ai_os_kernel::{
    start_grpc_server, init_tracing, init_simd, IPCManager, LocalFS, MemFS, MemoryManager, MmapManager,
    MountManager, SchedulingPolicy as Policy, ProcessManager, SandboxManager, SyscallExecutor,
};
use std::sync::Arc;

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

    info!("Initializing process manager with memory management, IPC cleanup, and scheduler...");
    let process_manager = ProcessManager::builder()
        .with_memory_manager(memory_manager.clone())
        .with_ipc_manager(ipc_manager.clone())
        .with_scheduler(Policy::Fair)
        .build();

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
    vfs.mount("/storage", Arc::new(LocalFS::new(&storage_path)))
        .expect("Failed to mount /storage");

    // Mount in-memory filesystem at /tmp (100MB limit)
    info!("Mounting in-memory filesystem at /tmp (100MB limit)");
    vfs.mount("/tmp", Arc::new(MemFS::with_capacity(100 * 1024 * 1024)))
        .expect("Failed to mount /tmp");

    // Mount in-memory filesystem at /cache (50MB limit)
    info!("Mounting in-memory filesystem at /cache (50MB limit)");
    vfs.mount("/cache", Arc::new(MemFS::with_capacity(50 * 1024 * 1024)))
        .expect("Failed to mount /cache");

    info!("Initializing mmap manager with VFS support...");
    let mmap_manager = MmapManager::with_vfs(Arc::new(vfs.clone()));

    info!("Initializing syscall executor with IPC, VFS, and mmap support...");
    let syscall_executor = SyscallExecutor::with_ipc(
        sandbox_manager.clone(),
        ipc_manager.pipes().clone(),
        ipc_manager.shm().clone(),
    )
    .with_queues(ipc_manager.queues().clone())
    .with_vfs(vfs)
    .with_mmap(mmap_manager)
    .with_metrics(metrics_collector.clone());

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

    // Spawn gRPC server as a background task
    tokio::spawn(async move {
        if let Err(e) = start_grpc_server(
            grpc_addr,
            grpc_syscall_executor,
            grpc_process_manager,
            grpc_sandbox_manager,
        )
        .await
        {
            tracing::error!(error = %e, "gRPC server error");
        }
    });

    info!("gRPC server started");
    info!("Kernel is ready to receive syscalls from AI service");

    // Clone memory manager for monitoring loop
    let monitor_mem_mgr = memory_manager.clone();

    // Kernel main loop with memory monitoring
    loop {
        // Log kernel statistics periodically
        info!("Kernel running - press Ctrl+C to exit");

        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
