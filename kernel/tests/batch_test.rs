/*!
 * Batch Execution Tests
 * Tests for batch syscall execution (parallel and sequential)
 */

use ai_os_kernel::api::batch::BatchExecutor;
use ai_os_kernel::security::traits::SandboxProvider;
use ai_os_kernel::security::{Capability, SandboxConfig, SandboxManager};
use ai_os_kernel::syscalls::{Syscall, SyscallExecutor, SyscallResult};
use std::fs;
use tempfile::TempDir;

fn setup_test_env() -> (SyscallExecutor, SandboxManager, TempDir, u32) {
    let sandbox_manager = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox_manager.clone());
    let temp_dir = TempDir::new().unwrap();
    let pid = 100;

    let mut config = SandboxConfig::standard(pid);
    let canonical_path = temp_dir.path().canonicalize().unwrap();
    config.allow_path(canonical_path);
    sandbox_manager.create_sandbox(config);

    (executor, sandbox_manager, temp_dir, pid)
}

#[tokio::test]
async fn test_batch_sequential_execution() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let batch_executor = BatchExecutor::new(executor);

    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    let requests = vec![
        (
            pid,
            Syscall::WriteFile {
                path: file1.clone(),
                data: b"data1".to_vec(),
            },
        ),
        (
            pid,
            Syscall::WriteFile {
                path: file2.clone(),
                data: b"data2".to_vec(),
            },
        ),
    ];

    let results = batch_executor.execute_batch(requests, false).await;

    assert_eq!(results.len(), 2);
    for result in results {
        match result {
            SyscallResult::Success { .. } => (),
            _ => panic!("Expected success"),
        }
    }

    // Verify files created
    assert!(file1.exists());
    assert!(file2.exists());
    assert_eq!(fs::read(&file1).unwrap(), b"data1");
    assert_eq!(fs::read(&file2).unwrap(), b"data2");
}

#[tokio::test]
async fn test_batch_parallel_execution() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let batch_executor = BatchExecutor::new(executor);

    // Create multiple syscalls
    let mut requests = vec![];
    for i in 0..10 {
        requests.push((
            pid,
            Syscall::WriteFile {
                path: temp_dir.path().join(format!("file{}.txt", i)),
                data: format!("data{}", i).into_bytes(),
            },
        ));
    }

    let results = batch_executor.execute_batch(requests, true).await;

    assert_eq!(results.len(), 10);

    // All should succeed
    let success_count = results
        .iter()
        .filter(|r| matches!(r, SyscallResult::Success { .. }))
        .count();
    assert_eq!(success_count, 10);
}

#[tokio::test]
async fn test_batch_mixed_results() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let batch_executor = BatchExecutor::new(executor);

    let valid_file = temp_dir.path().join("valid.txt");
    let invalid_path = "/invalid/path/file.txt";

    let requests = vec![
        (
            pid,
            Syscall::WriteFile {
                path: valid_file.clone(),
                data: b"valid".to_vec(),
            },
        ),
        (
            pid,
            Syscall::ReadFile {
                path: invalid_path.into(),
            },
        ),
        (pid, Syscall::GetCurrentTime),
    ];

    let results = batch_executor.execute_batch(requests, false).await;

    assert_eq!(results.len(), 3);

    // First should succeed
    match &results[0] {
        SyscallResult::Success { .. } => (),
        _ => panic!("Expected success for valid operation"),
    }

    // Second should fail
    match &results[1] {
        SyscallResult::PermissionDenied { .. } | SyscallResult::Error { .. } => (),
        _ => panic!("Expected failure for invalid path"),
    }

    // Third should succeed
    match &results[2] {
        SyscallResult::Success { .. } => (),
        _ => panic!("Expected success for system call"),
    }
}

#[tokio::test]
async fn test_batch_empty_requests() {
    let (executor, _, _, _) = setup_test_env();
    let batch_executor = BatchExecutor::new(executor);

    let results = batch_executor.execute_batch(vec![], false).await;
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_batch_large_batch() {
    let (executor, _, _, pid) = setup_test_env();
    let batch_executor = BatchExecutor::new(executor);

    // Create 100 system info requests
    let requests: Vec<_> = (0..100).map(|_| (pid, Syscall::GetCurrentTime)).collect();

    let start = std::time::Instant::now();
    let results = batch_executor.execute_batch(requests, true).await;
    let duration = start.elapsed();

    assert_eq!(results.len(), 100);

    let success_count = results
        .iter()
        .filter(|r| matches!(r, SyscallResult::Success { .. }))
        .count();

    assert_eq!(success_count, 100);

    // Parallel execution should be faster than 100 sequential syscalls
    // (though this is a weak assertion for a test)
    assert!(duration.as_millis() < 5000);
}

#[tokio::test]
async fn test_batch_read_write_sequence() {
    let (executor, _, temp_dir, pid) = setup_test_env();
    let batch_executor = BatchExecutor::new(executor);

    let file_path = temp_dir.path().join("test.txt");

    // Write then read
    let requests = vec![
        (
            pid,
            Syscall::WriteFile {
                path: file_path.clone(),
                data: b"test data".to_vec(),
            },
        ),
        (
            pid,
            Syscall::ReadFile {
                path: file_path.clone(),
            },
        ),
    ];

    let results = batch_executor.execute_batch(requests, false).await;

    assert_eq!(results.len(), 2);

    // Both should succeed
    match &results[0] {
        SyscallResult::Success { .. } => (),
        _ => panic!("Write should succeed"),
    }

    match &results[1] {
        SyscallResult::Success { data } => {
            assert_eq!(data.as_ref().unwrap(), b"test data");
        }
        _ => panic!("Read should succeed"),
    }
}
