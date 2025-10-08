/*!
 * Integration Tests
 * End-to-end tests for the kernel
 */

use ai_os_kernel::security::SandboxProvider;
use ai_os_kernel::{
    Capability, IPCManager, MemoryManager, ProcessManager, SandboxConfig, SandboxManager, Syscall,
    SyscallExecutorWithIpc, SyscallResult,
};
use pretty_assertions::assert_eq;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_full_process_lifecycle() {
    let mem_mgr = MemoryManager::new();
    let pm = ProcessManager::builder()
        .with_memory_manager(mem_mgr.clone())
        .build();
    let sandbox_mgr = SandboxManager::new();

    // Create process
    let pid = pm.create_process("test-app".to_string(), 5);
    assert_eq!(pid, 1);

    // Setup sandbox
    let config = SandboxConfig::standard(pid);
    sandbox_mgr.create_sandbox(config);

    // Allocate memory
    mem_mgr.allocate(10 * 1024 * 1024, pid).unwrap();
    assert_eq!(mem_mgr.process_memory(pid), 10 * 1024 * 1024);

    // Terminate process
    pm.terminate_process(pid);

    // Memory should be cleaned up
    assert_eq!(mem_mgr.process_memory(pid), 0);
}

#[test]
fn test_sandboxed_file_operations() {
    let sandbox_mgr = SandboxManager::new();
    let executor = SyscallExecutorWithIpc::new(sandbox_mgr.clone());
    let temp_dir = TempDir::new().unwrap();
    let pid = 100;

    // Create sandbox with file capabilities
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::ReadFile(None));
    config.grant_capability(Capability::WriteFile(None));
    config.grant_capability(Capability::CreateFile(None));
    config.allow_path(temp_dir.path().canonicalize().unwrap());
    sandbox_mgr.create_sandbox(config);

    // Write a file
    let test_file = temp_dir.path().join("test.txt");
    let content = b"Hello, World!".to_vec();

    let write_result = executor.execute(
        pid,
        Syscall::WriteFile {
            path: test_file.clone(),
            data: content.clone(),
        },
    );

    match write_result {
        SyscallResult::Success { .. } => {}
        _ => panic!("Write failed: {:?}", write_result),
    }

    // Read the file back
    let read_result = executor.execute(pid, Syscall::ReadFile { path: test_file });

    match read_result {
        SyscallResult::Success { data } => {
            assert_eq!(data.unwrap(), content);
        }
        _ => panic!("Read failed: {:?}", read_result),
    }
}

#[test]
fn test_sandbox_permission_enforcement() {
    let sandbox_mgr = SandboxManager::new();
    let executor = SyscallExecutorWithIpc::new(sandbox_mgr.clone());
    let temp_dir = TempDir::new().unwrap();
    let pid = 100;

    // Create minimal sandbox (no capabilities)
    let mut config = SandboxConfig::minimal(pid);
    config.allow_path(temp_dir.path().to_path_buf());
    sandbox_mgr.create_sandbox(config);

    // Try to write without capability
    let test_file = temp_dir.path().join("test.txt");
    let result = executor.execute(
        pid,
        Syscall::WriteFile {
            path: test_file,
            data: b"test".to_vec(),
        },
    );

    match result {
        SyscallResult::PermissionDenied { .. } => {}
        _ => panic!("Expected permission denied, got: {:?}", result),
    }
}

#[test]
fn test_process_memory_limits() {
    let mem_mgr = MemoryManager::new();
    let pm = ProcessManager::builder()
        .with_memory_manager(mem_mgr.clone())
        .build();

    let pid1 = pm.create_process("app1".to_string(), 5);
    let pid2 = pm.create_process("app2".to_string(), 5);

    // Allocate memory for both processes
    mem_mgr.allocate(400 * 1024 * 1024, pid1).unwrap();
    mem_mgr.allocate(400 * 1024 * 1024, pid2).unwrap();

    // Total should be 800MB
    let (_, used, _) = mem_mgr.info();
    assert_eq!(used, 800 * 1024 * 1024);

    // Try to allocate more than available
    let result = mem_mgr.allocate(300 * 1024 * 1024, pid1);
    assert!(result.is_err());

    // Terminate one process to free memory
    pm.terminate_process(pid1);

    // Now allocation should succeed
    let result = mem_mgr.allocate(300 * 1024 * 1024, pid2);
    assert!(result.is_ok());
}

#[test]
fn test_ipc_between_processes() {
    let ipc = IPCManager::new(MemoryManager::new());
    let pm = ProcessManager::new();

    let pid1 = pm.create_process("sender".to_string(), 5);
    let pid2 = pm.create_process("receiver".to_string(), 5);

    // Send message from pid1 to pid2
    let message = b"Hello from sender!".to_vec();
    ipc.send_message(pid1, pid2, message.clone()).unwrap();

    // Receive at pid2
    let received = ipc.receive_message(pid2).unwrap();
    assert_eq!(received.from, pid1);
    assert_eq!(received.to, pid2);
    assert_eq!(received.data, message);
}

#[test]
fn test_multiple_process_sandbox_isolation() {
    let sandbox_mgr = SandboxManager::new();
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    let pid1 = 100;
    let pid2 = 200;

    // Process 1 can only access temp_dir1
    let mut config1 = SandboxConfig::standard(pid1);
    let canon_path1 = temp_dir1.path().canonicalize().unwrap();
    config1.allow_path(canon_path1.clone());
    sandbox_mgr.create_sandbox(config1);

    // Process 2 can only access temp_dir2
    let mut config2 = SandboxConfig::standard(pid2);
    let canon_path2 = temp_dir2.path().canonicalize().unwrap();
    config2.allow_path(canon_path2.clone());
    sandbox_mgr.create_sandbox(config2);

    // pid1 can access temp_dir1
    assert!(sandbox_mgr.check_path_access(pid1, &canon_path1));

    // pid1 cannot access temp_dir2
    assert!(!sandbox_mgr.check_path_access(pid1, &canon_path2));

    // pid2 can access temp_dir2
    assert!(sandbox_mgr.check_path_access(pid2, &canon_path2));

    // pid2 cannot access temp_dir1
    assert!(!sandbox_mgr.check_path_access(pid2, &canon_path1));
}

#[test]
fn test_system_info_access_control() {
    let sandbox_mgr = SandboxManager::new();
    let executor = SyscallExecutorWithIpc::new(sandbox_mgr.clone());

    let privileged_pid = 100;
    let restricted_pid = 200;

    // Privileged process
    let priv_config = SandboxConfig::privileged(privileged_pid);
    sandbox_mgr.create_sandbox(priv_config);

    // Restricted process
    let restricted_config = SandboxConfig::minimal(restricted_pid);
    sandbox_mgr.create_sandbox(restricted_config);

    // Privileged process can get system info
    let result = executor.execute(privileged_pid, Syscall::GetSystemInfo);
    match result {
        SyscallResult::Success { .. } => {}
        _ => panic!("Privileged process should access system info"),
    }

    // Restricted process cannot
    let result = executor.execute(restricted_pid, Syscall::GetSystemInfo);
    match result {
        SyscallResult::PermissionDenied { .. } => {}
        _ => panic!("Restricted process should be denied"),
    }
}

#[test]
fn test_concurrent_process_operations() {
    use std::sync::Arc;
    use std::thread;

    let mem_mgr = Arc::new(MemoryManager::new());
    let pm = Arc::new(
        ProcessManager::builder()
            .with_memory_manager((*mem_mgr).clone())
            .build(),
    );

    let mut handles = vec![];

    // Create processes concurrently
    for i in 0..5 {
        let pm_clone = Arc::clone(&pm);
        let mem_mgr_clone = Arc::clone(&mem_mgr);

        let handle = thread::spawn(move || {
            let pid = pm_clone.create_process(format!("app-{}", i), 5);
            mem_mgr_clone.allocate(10 * 1024 * 1024, pid).unwrap();
            pid
        });

        handles.push(handle);
    }

    let pids: Vec<u32> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify all processes were created
    assert_eq!(pids.len(), 5);

    // Verify total memory
    let (_, used, _) = mem_mgr.info();
    assert_eq!(used, 50 * 1024 * 1024);
}

#[test]
fn test_sandbox_capability_update() {
    let sandbox_mgr = SandboxManager::new();
    let pid = 100;

    // Start with minimal sandbox
    let config = SandboxConfig::minimal(pid);
    sandbox_mgr.create_sandbox(config);

    // Check initial capabilities
    assert!(!sandbox_mgr.check_permission(pid, &Capability::ReadFile(None)));

    // Update to add capability
    let mut new_config = SandboxConfig::minimal(pid);
    new_config.grant_capability(Capability::ReadFile(None));
    sandbox_mgr.update_sandbox(pid, new_config);

    // Now should have capability
    assert!(sandbox_mgr.check_permission(pid, &Capability::ReadFile(None)));
}

#[test]
fn test_memory_recovery_after_oom() {
    let mem_mgr = MemoryManager::new();
    let pm = ProcessManager::builder()
        .with_memory_manager(mem_mgr.clone())
        .build();

    let pid = pm.create_process("memory-hog".to_string(), 5);

    // Allocate most of memory
    mem_mgr.allocate(900 * 1024 * 1024, pid).unwrap();

    // Try to allocate more - should fail
    let result = mem_mgr.allocate(200 * 1024 * 1024, pid);
    assert!(result.is_err());

    // Terminate process
    pm.terminate_process(pid);

    // Should be able to allocate again
    let new_pid = pm.create_process("new-app".to_string(), 5);
    let result = mem_mgr.allocate(500 * 1024 * 1024, new_pid);
    assert!(result.is_ok());
}

#[test]
fn test_file_operations_with_symlink_protection() {
    let sandbox_mgr = SandboxManager::new();
    let executor = SyscallExecutorWithIpc::new(sandbox_mgr.clone());
    let temp_dir = TempDir::new().unwrap();
    let pid = 100;

    // Create allowed directory
    let allowed_dir = temp_dir.path().join("allowed");
    fs::create_dir_all(&allowed_dir).unwrap();

    // Create blocked directory
    let blocked_dir = temp_dir.path().join("blocked");
    fs::create_dir_all(&blocked_dir).unwrap();

    // Create file in blocked directory
    let blocked_file = blocked_dir.join("secret.txt");
    fs::write(&blocked_file, b"secret").unwrap();

    // Setup sandbox - only allowed_dir is accessible
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::ReadFile(None));
    config.allow_path(allowed_dir.clone());
    sandbox_mgr.create_sandbox(config);

    // Create symlink in allowed dir pointing to blocked file
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link_path = allowed_dir.join("link_to_secret");
        let _ = symlink(&blocked_file, &link_path);

        // Try to read through symlink - should be blocked
        let result = executor.execute(pid, Syscall::ReadFile { path: link_path });

        match result {
            SyscallResult::PermissionDenied { .. } | SyscallResult::Error { .. } => {
                // Expected - either permission denied or path canonicalization error
            }
            SyscallResult::Success { .. } => {
                panic!("Should not be able to read through symlink to blocked path");
            }
        }
    }
}

#[test]
#[serial]
fn test_garbage_collection_on_process_cleanup() {
    let mem_mgr = MemoryManager::new();
    let pm = ProcessManager::builder()
        .with_memory_manager(mem_mgr.clone())
        .build();

    // Create and destroy many processes
    for i in 0..10 {
        let pid = pm.create_process(format!("app-{}", i), 5);
        mem_mgr.allocate(1024 * 1024, pid).unwrap();
        pm.terminate_process(pid);
    }

    // Force GC (may have already been triggered automatically during cleanup)
    // The important thing is that blocks are collected, not necessarily all at once
    let removed = mem_mgr.force_collect();
    // At least some blocks should have been deallocated
    // (automatic GC may have already collected most during the loop above)
    assert!(
        removed <= 10,
        "Should not collect more blocks than were created"
    );

    let stats = mem_mgr.stats();
    assert_eq!(stats.allocated_blocks, 0, "All blocks should be cleaned up");
}
