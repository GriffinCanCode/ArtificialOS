/*!
 * AI-OS Kernel - Main Entry Point
 *
 * Lightweight microkernel that provides:
 * - Process management
 * - Memory management
 * - IPC with AI service
 * - Hardware abstraction
 */

use log::info;
use std::error::Error;

use ai_os_kernel::{
    start_grpc_server, IPCManager, LocalFS, MemFS, MemoryManager, MountManager, Policy,
    ProcessManager, SandboxManager, SyscallExecutor,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("AgentOS Kernel starting...");
    info!("================================================");

    // Initialize kernel subsystems
    info!("Initializing memory manager...");
    let memory_manager = MemoryManager::new();

    info!("Initializing IPC system...");
    let ipc_manager = IPCManager::new();

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
    info!("Mounting local filesystem at /storage -> {}", storage_path);
    if let Err(e) = std::fs::create_dir_all(&storage_path) {
        log::warn!("Could not create storage directory: {}", e);
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

    info!("Initializing syscall executor with IPC and VFS support...");
    let syscall_executor = SyscallExecutor::with_ipc(
        sandbox_manager.clone(),
        ipc_manager.pipes().clone(),
        ipc_manager.shm().clone(),
    )
    .with_vfs(vfs);

    info!("Kernel initialization complete");
    info!("================================================");
    info!("Kernel entering main loop...");
    info!("Press Ctrl+C to exit");

    // Start gRPC server in parallel with main loop
    let grpc_addr = match "0.0.0.0:50051".parse() {
        Ok(addr) => addr,
        Err(e) => {
            log::error!("Failed to parse gRPC address: {}", e);
            return Err(format!("Failed to parse gRPC address: {}", e).into());
        }
    };
    let grpc_syscall_executor = syscall_executor.clone();
    let grpc_process_manager = process_manager.clone();
    let grpc_sandbox_manager = sandbox_manager.clone();

    info!("Starting gRPC server on {}", grpc_addr);

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
            log::error!("gRPC server error: {}", e);
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
