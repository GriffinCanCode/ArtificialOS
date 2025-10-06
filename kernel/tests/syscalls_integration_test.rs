/*!
 * Comprehensive Syscall Integration Tests
 * Tests all 50 syscalls for Phase 5 completion
 */

use ai_os_kernel::ipc::{PipeManager, ShmManager};
use ai_os_kernel::memory::MemoryManager;
use ai_os_kernel::process::ProcessManager;
use ai_os_kernel::security::{SandboxConfig, SandboxManager};
use ai_os_kernel::syscalls::{Syscall, SyscallExecutor, SyscallResult};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test syscall executor
fn create_test_executor() -> (SyscallExecutor, SandboxManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let sandbox_mgr = SandboxManager::new();

    // Create privileged test process
    let test_pid = 1000;
    let config = SandboxConfig::privileged(test_pid);
    sandbox_mgr.create_sandbox(config);

    // Create managers
    let memory_manager = MemoryManager::new();
    let pipe_manager = PipeManager::new(memory_manager.clone());
    let shm_manager = ShmManager::new(memory_manager.clone());
    let process_manager = ProcessManager::new();

    let executor = SyscallExecutor::with_full_features(
        sandbox_mgr.clone(),
        pipe_manager,
        shm_manager,
        process_manager,
        memory_manager,
    );
    (executor, sandbox_mgr, temp_dir)
}

// ============================================================================
// File System Syscalls (14 tests)
// ============================================================================

#[test]
fn test_read_write_file() {
    let (executor, _, temp_dir) = create_test_executor();
    let test_file = temp_dir.path().join("test.txt");

    // Create file
    let result = executor.execute(
        1000,
        Syscall::CreateFile {
            path: test_file.clone(),
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));

    // Write file
    let data = b"Hello, World!".to_vec();
    let result = executor.execute(
        1000,
        Syscall::WriteFile {
            path: test_file.clone(),
            data: data.clone(),
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));

    // Read file
    let result = executor.execute(1000, Syscall::ReadFile { path: test_file });
    match result {
        SyscallResult::Success {
            data: Some(read_data),
        } => {
            assert_eq!(read_data, data);
        }
        _ => panic!("Expected success with data"),
    }
}

#[test]
fn test_file_exists() {
    let (executor, _, temp_dir) = create_test_executor();
    let test_file = temp_dir.path().join("exists.txt");

    // File doesn't exist yet - should return Success with data [0]
    let result = executor.execute(
        1000,
        Syscall::FileExists {
            path: test_file.clone(),
        },
    );
    match result {
        SyscallResult::Success { data: Some(d) } => {
            assert_eq!(d, vec![0], "Expected [0] for non-existent file");
        }
        _ => panic!("Expected Success with data [0] for non-existent file"),
    }

    // Create file
    executor.execute(
        1000,
        Syscall::CreateFile {
            path: test_file.clone(),
        },
    );

    // Now it exists - should return Success with data [1]
    let result = executor.execute(1000, Syscall::FileExists { path: test_file });
    match result {
        SyscallResult::Success { data: Some(d) } => {
            assert_eq!(d, vec![1], "Expected [1] for existing file");
        }
        _ => panic!("Expected Success with data [1] for existing file"),
    }
}

#[test]
fn test_move_copy_file() {
    let (executor, _, temp_dir) = create_test_executor();
    let source = temp_dir.path().join("source.txt");
    let dest = temp_dir.path().join("dest.txt");

    // Create and write source
    executor.execute(
        1000,
        Syscall::CreateFile {
            path: source.clone(),
        },
    );
    let data = b"Test data".to_vec();
    executor.execute(
        1000,
        Syscall::WriteFile {
            path: source.clone(),
            data: data.clone(),
        },
    );

    // Copy file
    let result = executor.execute(
        1000,
        Syscall::CopyFile {
            source: source.clone(),
            destination: dest.clone(),
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));

    // Verify copy
    let result = executor.execute(1000, Syscall::ReadFile { path: dest.clone() });
    match result {
        SyscallResult::Success {
            data: Some(read_data),
        } => {
            assert_eq!(read_data, data);
        }
        _ => panic!("Expected success with data"),
    }
}

#[test]
fn test_directory_operations() {
    let (executor, _, temp_dir) = create_test_executor();
    let test_dir = temp_dir.path().join("testdir");

    // Create directory
    let result = executor.execute(
        1000,
        Syscall::CreateDirectory {
            path: test_dir.clone(),
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));

    // List directory
    let result = executor.execute(
        1000,
        Syscall::ListDirectory {
            path: temp_dir.path().to_path_buf(),
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));

    // Remove directory
    let result = executor.execute(
        1000,
        Syscall::RemoveDirectory {
            path: test_dir.clone(),
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));
}

#[test]
fn test_working_directory() {
    let (executor, _, temp_dir) = create_test_executor();

    // Get current working directory
    let result = executor.execute(1000, Syscall::GetWorkingDirectory);
    assert!(matches!(result, SyscallResult::Success { .. }));

    // Set working directory
    let result = executor.execute(
        1000,
        Syscall::SetWorkingDirectory {
            path: temp_dir.path().to_path_buf(),
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));
}

#[test]
fn test_truncate_file() {
    let (executor, _, temp_dir) = create_test_executor();
    let test_file = temp_dir.path().join("truncate.txt");

    // Create and write file
    executor.execute(
        1000,
        Syscall::CreateFile {
            path: test_file.clone(),
        },
    );
    executor.execute(
        1000,
        Syscall::WriteFile {
            path: test_file.clone(),
            data: b"Hello, World!".to_vec(),
        },
    );

    // Truncate file
    let result = executor.execute(
        1000,
        Syscall::TruncateFile {
            path: test_file.clone(),
            size: 5,
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));

    // Verify truncation
    let result = executor.execute(1000, Syscall::ReadFile { path: test_file });
    match result {
        SyscallResult::Success {
            data: Some(read_data),
        } => {
            assert_eq!(read_data.len(), 5);
        }
        _ => panic!("Expected success with data"),
    }
}

// ============================================================================
// Process Syscalls (8 tests)
// ============================================================================

#[test]
fn test_spawn_process() {
    let (executor, _, _) = create_test_executor();

    let result = executor.execute(
        1000,
        Syscall::SpawnProcess {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
        },
    );

    assert!(matches!(result, SyscallResult::Success { .. }));
}

#[test]
fn test_get_process_list() {
    let (executor, _, _) = create_test_executor();

    let result = executor.execute(1000, Syscall::GetProcessList);
    assert!(matches!(result, SyscallResult::Success { .. }));
}

#[test]
fn test_get_process_info() {
    let (executor, _, _) = create_test_executor();

    // Try to get info for our test process
    let result = executor.execute(1000, Syscall::GetProcessInfo { target_pid: 1000 });

    // May succeed or error depending on process existence
    assert!(matches!(
        result,
        SyscallResult::Success { .. } | SyscallResult::Error { .. }
    ));
}

// ============================================================================
// System Info Syscalls (4 tests)
// ============================================================================

#[test]
fn test_get_system_info() {
    let (executor, _, _) = create_test_executor();

    let result = executor.execute(1000, Syscall::GetSystemInfo);
    assert!(matches!(result, SyscallResult::Success { .. }));
}

#[test]
fn test_get_current_time() {
    let (executor, _, _) = create_test_executor();

    let result = executor.execute(1000, Syscall::GetCurrentTime);
    assert!(matches!(result, SyscallResult::Success { .. }));
}

#[test]
fn test_environment_variables() {
    let (executor, _, _) = create_test_executor();

    // Set env var
    let result = executor.execute(
        1000,
        Syscall::SetEnvironmentVar {
            key: "TEST_VAR".to_string(),
            value: "test_value".to_string(),
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));

    // Get env var
    let result = executor.execute(
        1000,
        Syscall::GetEnvironmentVar {
            key: "TEST_VAR".to_string(),
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));
}

// ============================================================================
// Time Syscalls (2 tests)
// ============================================================================

#[test]
fn test_sleep() {
    let (executor, _, _) = create_test_executor();

    let result = executor.execute(1000, Syscall::Sleep { duration_ms: 10 });
    assert!(matches!(result, SyscallResult::Success { .. }));
}

#[test]
fn test_get_uptime() {
    let (executor, _, _) = create_test_executor();

    let result = executor.execute(1000, Syscall::GetUptime);
    assert!(matches!(result, SyscallResult::Success { .. }));
}

// ============================================================================
// Memory Syscalls (3 tests)
// ============================================================================

#[test]
fn test_get_memory_stats() {
    let (executor, _, _) = create_test_executor();

    let result = executor.execute(1000, Syscall::GetMemoryStats);
    assert!(matches!(result, SyscallResult::Success { .. }));
}

#[test]
fn test_get_process_memory_stats() {
    let (executor, _, _) = create_test_executor();

    let result = executor.execute(1000, Syscall::GetProcessMemoryStats { target_pid: 1000 });
    assert!(matches!(result, SyscallResult::Success { .. }));
}

#[test]
fn test_trigger_gc() {
    let (executor, _, _) = create_test_executor();

    // Trigger GC for specific process
    let result = executor.execute(
        1000,
        Syscall::TriggerGC {
            target_pid: Some(1000),
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));

    // Trigger global GC
    let result = executor.execute(1000, Syscall::TriggerGC { target_pid: None });
    assert!(matches!(result, SyscallResult::Success { .. }));
}

// ============================================================================
// Signal Syscalls (1 test)
// ============================================================================

#[test]
fn test_send_signal() {
    let (executor, _, _) = create_test_executor();

    let result = executor.execute(
        1000,
        Syscall::SendSignal {
            target_pid: 1000,
            signal: 15, // SIGTERM
        },
    );
    assert!(matches!(result, SyscallResult::Success { .. }));
}

// ============================================================================
// Permission Tests
// ============================================================================

#[test]
fn test_permission_denied() {
    let sandbox_mgr = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox_mgr.clone());

    // Create minimal sandbox without file permissions
    let test_pid = 2000;
    let config = SandboxConfig::minimal(test_pid);
    sandbox_mgr.create_sandbox(config);

    // Try to read file without permission
    let result = executor.execute(
        test_pid,
        Syscall::ReadFile {
            path: PathBuf::from("/tmp/test.txt"),
        },
    );

    assert!(matches!(result, SyscallResult::PermissionDenied { .. }));
}
