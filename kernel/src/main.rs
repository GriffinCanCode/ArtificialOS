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
use std::path::PathBuf;

use ai_os_kernel::{
    start_grpc_server, IPCManager, MemoryManager, ProcessManager, SandboxConfig, SandboxManager,
    Syscall, SyscallExecutor,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚀 AI-OS Kernel starting...");
    info!("================================================");

    // Initialize kernel subsystems
    info!("Initializing memory manager...");
    let memory_manager = MemoryManager::new();

    info!("Initializing process manager with memory management...");
    let process_manager = ProcessManager::with_memory_manager(memory_manager.clone());

    info!("Initializing IPC system...");
    let _ipc_manager = IPCManager::new();

    info!("Initializing sandbox manager...");
    let sandbox_manager = SandboxManager::new();

    info!("Initializing syscall executor...");
    let syscall_executor = SyscallExecutor::new(sandbox_manager.clone());

    info!("✅ Kernel initialization complete");
    info!("================================================");

    // Demo: Create a test process with sandboxing
    demo_sandboxed_execution(&process_manager, &sandbox_manager, &syscall_executor);

    // Demo: Memory management with OOM handling
    demo_memory_management(&memory_manager);

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

    info!("✅ gRPC server started");
    info!("Kernel is ready to receive syscalls from AI service");

    // Clone memory manager for monitoring loop
    let monitor_mem_mgr = memory_manager.clone();

    // Kernel main loop with memory monitoring
    loop {
        // Log memory statistics periodically
        let stats = monitor_mem_mgr.get_detailed_stats();
        info!(
            "Memory: {:.1}% used ({} MB / {} MB), {} blocks allocated",
            stats.usage_percentage,
            stats.used_memory / (1024 * 1024),
            stats.total_memory / (1024 * 1024),
            stats.allocated_blocks
        );

        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}

/// Demonstration of sandboxed execution
fn demo_sandboxed_execution(
    process_manager: &ProcessManager,
    sandbox_manager: &SandboxManager,
    syscall_executor: &SyscallExecutor,
) {
    info!("Running sandboxed execution demo...");
    info!("-----------------------------------");

    // Create a test process
    let pid = process_manager.create_process("test-app".to_string(), 5);

    // Create a standard sandbox for it
    let mut sandbox_config = SandboxConfig::standard(pid);
    sandbox_config.allow_path(PathBuf::from("/tmp"));
    sandbox_manager.create_sandbox(sandbox_config);

    // Test 1: Allowed file read (should succeed)
    info!("\n[Test 1] Attempting allowed file operation...");
    let result = syscall_executor.execute(
        pid,
        Syscall::FileExists {
            path: PathBuf::from("/tmp/test.txt"),
        },
    );
    info!("Result: {:?}", result);

    // Test 2: Blocked file read (should fail)
    info!("\n[Test 2] Attempting blocked file operation...");
    let result = syscall_executor.execute(
        pid,
        Syscall::ReadFile {
            path: PathBuf::from("/etc/passwd"),
        },
    );
    info!("Result: {:?}", result);

    // Test 3: Missing capability (should fail)
    info!("\n[Test 3] Attempting operation without capability...");
    let result = syscall_executor.execute(
        pid,
        Syscall::SpawnProcess {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
        },
    );
    info!("Result: {:?}", result);

    // Test 4: System info (should succeed)
    info!("\n[Test 4] Attempting allowed system info...");
    let result = syscall_executor.execute(pid, Syscall::GetSystemInfo);
    info!("Result: {:?}", result);

    info!("-----------------------------------");
    info!("Sandboxed execution demo complete!");
    info!("");
}

/// Demonstration of memory management with OOM handling
fn demo_memory_management(memory_manager: &MemoryManager) {
    info!("Running memory management demo...");
    info!("-----------------------------------");

    // Test 1: Normal allocation
    info!("\n[Test 1] Normal memory allocation...");
    let pid = 100;
    match memory_manager.allocate(1024 * 1024, pid) {
        Ok(addr) => info!("✅ Allocated 1 MB at address 0x{:x}", addr),
        Err(e) => info!("❌ Allocation failed: {}", e),
    }

    // Test 2: Large allocation triggering memory pressure warning
    info!("\n[Test 2] Large allocation (should trigger warning)...");
    let pid = 101;
    let large_size = 900 * 1024 * 1024; // 900 MB
    match memory_manager.allocate(large_size, pid) {
        Ok(addr) => info!(
            "✅ Allocated {} MB at address 0x{:x}",
            large_size / (1024 * 1024),
            addr
        ),
        Err(e) => info!("❌ Allocation failed: {}", e),
    }

    // Test 3: OOM scenario
    info!("\n[Test 3] OOM scenario (should fail gracefully)...");
    let pid = 102;
    let oom_size = 200 * 1024 * 1024; // 200 MB (should exceed 1GB total)
    match memory_manager.allocate(oom_size, pid) {
        Ok(addr) => info!(
            "✅ Allocated {} MB at address 0x{:x}",
            oom_size / (1024 * 1024),
            addr
        ),
        Err(e) => info!("✅ Gracefully handled OOM: {}", e),
    }

    // Test 4: Process memory cleanup
    info!("\n[Test 4] Cleanup after process termination...");
    let freed = memory_manager.free_process_memory(101);
    info!("✅ Freed {} MB from PID 101", freed / (1024 * 1024));

    // Test 5: Retry allocation after cleanup
    info!("\n[Test 5] Retry allocation after cleanup...");
    let pid = 103;
    match memory_manager.allocate(100 * 1024 * 1024, pid) {
        Ok(addr) => info!(
            "✅ Successfully allocated 100 MB at address 0x{:x} after cleanup",
            addr
        ),
        Err(e) => info!("❌ Allocation failed: {}", e),
    }

    // Show final statistics
    let stats = memory_manager.get_detailed_stats();
    info!("\n[Final Statistics]");
    info!("  Total Memory: {} MB", stats.total_memory / (1024 * 1024));
    info!(
        "  Used Memory: {} MB ({:.1}%)",
        stats.used_memory / (1024 * 1024),
        stats.usage_percentage
    );
    info!("  Available: {} MB", stats.available_memory / (1024 * 1024));
    info!("  Allocated Blocks: {}", stats.allocated_blocks);

    info!("-----------------------------------");
    info!("Memory management demo complete!");
    info!("");
}
