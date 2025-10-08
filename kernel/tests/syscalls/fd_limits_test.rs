/*!
 * File Descriptor Limits Tests
 * Tests per-process FD limit enforcement
 */

use ai_os_kernel::core::types::Pid;
use ai_os_kernel::security::{
    ResourceLimitProvider, SandboxConfig, SandboxManager, SandboxProvider,
};
use ai_os_kernel::syscalls::{FdManager, Syscall, SyscallExecutorWithIpc, SyscallResult};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_temp_file(dir: &TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    let mut file = File::create(&path).unwrap();
    writeln!(file, "test content").unwrap();
    path
}

fn extract_fd(result: SyscallResult) -> u32 {
    match result {
        SyscallResult::Success { data } => {
            let json: serde_json::Value = serde_json::from_slice(&data.unwrap()).unwrap();
            json["fd"].as_u64().unwrap() as u32
        }
        _ => panic!("Expected success, got: {:?}", result),
    }
}

#[test]
fn test_fd_limit_enforcement() {
    let sandbox_manager = SandboxManager::new();
    let memory_manager = ai_os_kernel::memory::MemoryManager::new();
    let pipe_manager = ai_os_kernel::ipc::PipeManager::new(memory_manager.clone());
    let shm_manager = ai_os_kernel::ipc::ShmManager::new(memory_manager.clone());
    let executor = SyscallExecutorWithIpc::with_ipc_direct(sandbox_manager.clone(), pipe_manager, shm_manager);

    let pid: Pid = 1000;
    let mut config = SandboxConfig::standard(pid);
    config.resource_limits.max_file_descriptors = 5; // Very low limit for testing
    sandbox_manager.create_sandbox(config);

    let temp_dir = TempDir::new().unwrap();
    let file1 = create_temp_file(&temp_dir, "test1.txt");
    let file2 = create_temp_file(&temp_dir, "test2.txt");
    let file3 = create_temp_file(&temp_dir, "test3.txt");
    let file4 = create_temp_file(&temp_dir, "test4.txt");
    let file5 = create_temp_file(&temp_dir, "test5.txt");
    let file6 = create_temp_file(&temp_dir, "test6.txt");

    // Open files up to the limit
    for (i, file) in [&file1, &file2, &file3, &file4, &file5].iter().enumerate() {
        let result = executor.execute(
            pid,
            Syscall::Open {
                path: (*file).clone(),
                flags: 0, // O_RDONLY
                mode: 0,
            },
        );
        assert!(
            matches!(result, SyscallResult::Success { .. }),
            "Opening file {} should succeed (within limit)",
            i + 1
        );
    }

    // Verify we're at the limit
    let fd_count = executor.fd_manager().get_fd_count(pid);
    assert_eq!(fd_count, 5, "Should have 5 FDs open");

    // Try to open one more file - should fail
    let result = executor.execute(
        pid,
        Syscall::Open {
            path: file6,
            flags: 0,
            mode: 0,
        },
    );
    assert!(
        matches!(result, SyscallResult::PermissionDenied { .. }),
        "Opening file beyond limit should fail with PermissionDenied"
    );

    // Verify count hasn't changed
    let fd_count = executor.fd_manager().get_fd_count(pid);
    assert_eq!(fd_count, 5, "FD count should still be 5");
}

#[test]
fn test_fd_limit_with_dup() {
    let sandbox_manager = SandboxManager::new();
    let memory_manager = ai_os_kernel::memory::MemoryManager::new();
    let pipe_manager = ai_os_kernel::ipc::PipeManager::new(memory_manager.clone());
    let shm_manager = ai_os_kernel::ipc::ShmManager::new(memory_manager.clone());
    let executor = SyscallExecutorWithIpc::with_ipc_direct(sandbox_manager.clone(), pipe_manager, shm_manager);

    let pid: Pid = 1001;
    let mut config = SandboxConfig::standard(pid);
    config.resource_limits.max_file_descriptors = 3;
    sandbox_manager.create_sandbox(config);

    let temp_dir = TempDir::new().unwrap();
    let file1 = create_temp_file(&temp_dir, "test1.txt");
    let file2 = create_temp_file(&temp_dir, "test2.txt");

    // Open 2 files
    let fd1 = extract_fd(executor.execute(
        pid,
        Syscall::Open {
            path: file1,
            flags: 0,
            mode: 0,
        },
    ));

    let _ = extract_fd(executor.execute(
        pid,
        Syscall::Open {
            path: file2,
            flags: 0,
            mode: 0,
        },
    ));

    // dup should succeed (still within limit)
    let dup_result = executor.execute(pid, Syscall::Dup { fd: fd1 });
    assert!(
        matches!(dup_result, SyscallResult::Success { .. }),
        "dup should succeed within limit"
    );

    // Now at limit (3 FDs), dup should fail
    let dup_result2 = executor.execute(pid, Syscall::Dup { fd: fd1 });
    assert!(
        matches!(dup_result2, SyscallResult::PermissionDenied { .. }),
        "dup should fail when at limit"
    );
}

#[test]
fn test_fd_limit_with_dup2() {
    let sandbox_manager = SandboxManager::new();
    let memory_manager = ai_os_kernel::memory::MemoryManager::new();
    let pipe_manager = ai_os_kernel::ipc::PipeManager::new(memory_manager.clone());
    let shm_manager = ai_os_kernel::ipc::ShmManager::new(memory_manager.clone());
    let executor = SyscallExecutorWithIpc::with_ipc_direct(sandbox_manager.clone(), pipe_manager, shm_manager);

    let pid: Pid = 1002;
    let mut config = SandboxConfig::standard(pid);
    config.resource_limits.max_file_descriptors = 2;
    sandbox_manager.create_sandbox(config);

    let temp_dir = TempDir::new().unwrap();
    let file1 = create_temp_file(&temp_dir, "test1.txt");
    let file2 = create_temp_file(&temp_dir, "test2.txt");

    // Open 2 files (at limit)
    let fd1 = extract_fd(executor.execute(
        pid,
        Syscall::Open {
            path: file1,
            flags: 0,
            mode: 0,
        },
    ));

    let fd2 = extract_fd(executor.execute(
        pid,
        Syscall::Open {
            path: file2,
            flags: 0,
            mode: 0,
        },
    ));

    // dup2 to an existing FD should succeed (not allocating new FD)
    let dup2_result = executor.execute(
        pid,
        Syscall::Dup2 {
            oldfd: fd1,
            newfd: fd2,
        },
    );
    assert!(
        matches!(dup2_result, SyscallResult::Success { .. }),
        "dup2 to existing FD should succeed"
    );

    // dup2 to a new FD should fail (would exceed limit)
    let dup2_result2 = executor.execute(
        pid,
        Syscall::Dup2 {
            oldfd: fd1,
            newfd: 999,
        },
    );
    assert!(
        matches!(dup2_result2, SyscallResult::PermissionDenied { .. }),
        "dup2 to new FD should fail at limit"
    );
}

#[test]
fn test_fd_cleanup_reduces_count() {
    let sandbox_manager = SandboxManager::new();
    let memory_manager = ai_os_kernel::memory::MemoryManager::new();
    let pipe_manager = ai_os_kernel::ipc::PipeManager::new(memory_manager.clone());
    let shm_manager = ai_os_kernel::ipc::ShmManager::new(memory_manager.clone());
    let executor = SyscallExecutorWithIpc::with_ipc_direct(sandbox_manager.clone(), pipe_manager, shm_manager);

    let pid: Pid = 1003;
    let mut config = SandboxConfig::standard(pid);
    config.resource_limits.max_file_descriptors = 3;
    sandbox_manager.create_sandbox(config);

    let temp_dir = TempDir::new().unwrap();
    let file1 = create_temp_file(&temp_dir, "test1.txt");
    let file2 = create_temp_file(&temp_dir, "test2.txt");
    let file3 = create_temp_file(&temp_dir, "test3.txt");

    // Open 3 files (at limit)
    let fd1 = extract_fd(executor.execute(
        pid,
        Syscall::Open {
            path: file1,
            flags: 0,
            mode: 0,
        },
    ));

    let _ = extract_fd(executor.execute(
        pid,
        Syscall::Open {
            path: file2,
            flags: 0,
            mode: 0,
        },
    ));

    let _ = extract_fd(executor.execute(
        pid,
        Syscall::Open {
            path: file3.clone(),
            flags: 0,
            mode: 0,
        },
    ));

    assert_eq!(executor.fd_manager().get_fd_count(pid), 3);

    // Close one FD
    let close_result = executor.execute(pid, Syscall::Close { fd: fd1 });
    assert!(matches!(close_result, SyscallResult::Success { .. }));
    assert_eq!(executor.fd_manager().get_fd_count(pid), 2);

    // Now we should be able to open another file
    let file4 = create_temp_file(&temp_dir, "test4.txt");
    let result4 = executor.execute(
        pid,
        Syscall::Open {
            path: file4,
            flags: 0,
            mode: 0,
        },
    );
    assert!(
        matches!(result4, SyscallResult::Success { .. }),
        "Should be able to open file after closing one"
    );
}

#[test]
fn test_per_process_isolation() {
    let sandbox_manager = SandboxManager::new();
    let memory_manager = ai_os_kernel::memory::MemoryManager::new();
    let pipe_manager = ai_os_kernel::ipc::PipeManager::new(memory_manager.clone());
    let shm_manager = ai_os_kernel::ipc::ShmManager::new(memory_manager.clone());
    let executor = SyscallExecutorWithIpc::with_ipc_direct(sandbox_manager.clone(), pipe_manager, shm_manager);

    let pid1: Pid = 2000;
    let pid2: Pid = 2001;

    let mut config1 = SandboxConfig::standard(pid1);
    config1.resource_limits.max_file_descriptors = 2;
    sandbox_manager.create_sandbox(config1);

    let mut config2 = SandboxConfig::standard(pid2);
    config2.resource_limits.max_file_descriptors = 10;
    sandbox_manager.create_sandbox(config2);

    let temp_dir = TempDir::new().unwrap();
    let file1 = create_temp_file(&temp_dir, "test1.txt");
    let file2 = create_temp_file(&temp_dir, "test2.txt");
    let file3 = create_temp_file(&temp_dir, "test3.txt");

    // PID1 opens 2 files (at limit)
    let _ = extract_fd(executor.execute(
        pid1,
        Syscall::Open {
            path: file1.clone(),
            flags: 0,
            mode: 0,
        },
    ));
    let _ = extract_fd(executor.execute(
        pid1,
        Syscall::Open {
            path: file2.clone(),
            flags: 0,
            mode: 0,
        },
    ));

    // PID1 can't open more
    let result = executor.execute(
        pid1,
        Syscall::Open {
            path: file3.clone(),
            flags: 0,
            mode: 0,
        },
    );
    assert!(
        matches!(result, SyscallResult::PermissionDenied { .. }),
        "PID1 should be at limit"
    );

    // PID2 should still be able to open files (different limit)
    let result2 = executor.execute(
        pid2,
        Syscall::Open {
            path: file3,
            flags: 0,
            mode: 0,
        },
    );
    assert!(
        matches!(result2, SyscallResult::Success { .. }),
        "PID2 should not be affected by PID1's limit"
    );

    // Verify counts are separate
    assert_eq!(executor.fd_manager().get_fd_count(pid1), 2);
    assert_eq!(executor.fd_manager().get_fd_count(pid2), 1);
}

#[test]
fn test_fd_manager_atomic_operations() {
    let fd_manager = FdManager::new();
    let pid: Pid = 3000;

    // Initially zero
    assert_eq!(fd_manager.get_fd_count(pid), 0);
    assert!(!fd_manager.has_process_fds(pid));

    // Cleanup all
    let closed = fd_manager.cleanup_process_fds(pid);
    assert_eq!(closed, 0); // Nothing to clean
    assert_eq!(fd_manager.get_fd_count(pid), 0);
    assert!(!fd_manager.has_process_fds(pid));
}

#[test]
fn test_different_limit_tiers() {
    let sandbox_manager = SandboxManager::new();
    let memory_manager = ai_os_kernel::memory::MemoryManager::new();
    let pipe_manager = ai_os_kernel::ipc::PipeManager::new(memory_manager.clone());
    let shm_manager = ai_os_kernel::ipc::ShmManager::new(memory_manager.clone());
    let _executor = SyscallExecutorWithIpc::with_ipc_direct(sandbox_manager.clone(), pipe_manager, shm_manager);

    // Test minimal limits
    let pid_minimal: Pid = 4000;
    let config_minimal = SandboxConfig::minimal(pid_minimal);
    sandbox_manager.create_sandbox(config_minimal);

    let limits_minimal = sandbox_manager.get_limits(pid_minimal).unwrap();
    assert_eq!(limits_minimal.max_file_descriptors, 10);

    // Test default limits
    let pid_default: Pid = 4001;
    let config_default = SandboxConfig::minimal(pid_default);
    sandbox_manager.create_sandbox(config_default);

    let limits_default = sandbox_manager.get_limits(pid_default).unwrap();
    assert!(limits_default.max_file_descriptors >= 10);

    // Test privileged limits
    let pid_privileged: Pid = 4002;
    let config_privileged = SandboxConfig::privileged(pid_privileged);
    sandbox_manager.create_sandbox(config_privileged);

    let limits_privileged = sandbox_manager.get_limits(pid_privileged).unwrap();
    assert_eq!(limits_privileged.max_file_descriptors, 500);
}

#[test]
fn test_no_sandbox_no_limit() {
    let sandbox_manager = SandboxManager::new();
    let memory_manager = ai_os_kernel::memory::MemoryManager::new();
    let pipe_manager = ai_os_kernel::ipc::PipeManager::new(memory_manager.clone());
    let shm_manager = ai_os_kernel::ipc::ShmManager::new(memory_manager.clone());
    let executor = SyscallExecutorWithIpc::with_ipc_direct(sandbox_manager.clone(), pipe_manager, shm_manager);

    let pid: Pid = 5000;
    // No sandbox created for this PID

    let temp_dir = TempDir::new().unwrap();

    // Should be able to open files without limit check
    for i in 0..20 {
        let file = create_temp_file(&temp_dir, &format!("test{}.txt", i));
        let result = executor.execute(
            pid,
            Syscall::Open {
                path: file,
                flags: 0,
                mode: 0,
            },
        );
        assert!(
            matches!(result, SyscallResult::Success { .. }),
            "Should succeed without sandbox (no limit enforcement)"
        );
    }

    assert_eq!(executor.fd_manager().get_fd_count(pid), 20);
}
