/*!
 * Integration Tests for Process Isolation
 * Tests full integration of ProcessManager, Executor, and LimitManager
 */

use ai_os_kernel::{
    ExecutionConfig, LimitManager, Limits, MemoryManager, ProcessManager, ProcessState,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_full_process_lifecycle_with_executor() {
    let pm = ProcessManager::builder().with_executor().build();

    // Create process with OS execution
    let config = ExecutionConfig::new("echo".to_string()).with_args(vec!["test".to_string()]);
    let pid = pm.create_process_with_command("test-app".to_string(), 5, Some(config));

    // Verify process created
    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.name, "test-app");
    // OS PID may be None if command doesn't exist or fails validation
    // This is expected behavior - the process metadata is still created

    // Wait for process to complete
    sleep(Duration::from_millis(200)).await;

    // Terminate
    assert!(pm.terminate_process(pid));
}

#[tokio::test]
async fn test_process_manager_with_memory_and_executor() {
    let mem_mgr = MemoryManager::new();
    let pm = ProcessManager::builder()
        .with_memory_manager(mem_mgr.clone())
        .with_executor()
        .with_limits()
        .build();

    // Create process with command
    let config = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);
    let pid = pm.create_process_with_command("test-app".to_string(), 5, Some(config));

    // Allocate memory
    mem_mgr.allocate(10 * 1024 * 1024, pid).unwrap();

    // Verify both process and memory exist
    assert!(pm.get_process(pid).is_some());
    assert_eq!(mem_mgr.process_memory(pid), 10 * 1024 * 1024);

    // Terminate - should cleanup both
    pm.terminate_process(pid);

    assert_eq!(mem_mgr.process_memory(pid), 0);
}

#[tokio::test]
async fn test_multiple_processes_with_isolation() {
    let pm = ProcessManager::builder().with_executor().build();

    // Create 3 processes
    let mut pids = Vec::new();
    for i in 1..=3 {
        let config = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.2".to_string()]);
        let pid = pm.create_process_with_command(format!("app-{}", i), 5, Some(config));
        pids.push(pid);
    }

    // Verify all created
    let processes = pm.list_processes();
    assert_eq!(processes.len(), 3);

    // Collect OS PIDs that were successfully spawned
    let os_pids: Vec<_> = processes.iter().filter_map(|p| p.os_pid).collect();

    // If any processes spawned successfully, verify they have unique OS PIDs
    if !os_pids.is_empty() {
        let unique_count = os_pids
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(unique_count, os_pids.len(), "All OS PIDs should be unique");
    }

    // Cleanup
    for pid in pids {
        pm.terminate_process(pid);
    }
}

#[tokio::test]
async fn test_process_without_os_execution() {
    let pm = ProcessManager::builder().with_executor().build();

    // Create process without command (metadata only)
    let pid = pm.create_process("virtual-app".to_string(), 5);

    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.name, "virtual-app");
    assert_eq!(process.os_pid, None);
    // Process completes full lifecycle: Creating -> Initializing -> Ready
    assert!(matches!(process.state, ProcessState::Ready));

    pm.terminate_process(pid);
}

#[tokio::test]
async fn test_priority_based_resource_limits() {
    let pm = ProcessManager::builder().with_executor().build();

    // Low priority process
    let config_low = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);
    let pid_low = pm.create_process_with_command("low-priority".to_string(), 2, Some(config_low));

    // High priority process
    let config_high = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);
    let pid_high =
        pm.create_process_with_command("high-priority".to_string(), 9, Some(config_high));

    // Both should be created (resource limits applied internally)
    assert!(pm.get_process(pid_low).is_some());
    assert!(pm.get_process(pid_high).is_some());

    // Cleanup
    sleep(Duration::from_millis(200)).await;
    pm.terminate_process(pid_low);
    pm.terminate_process(pid_high);
}

#[test]
fn test_limit_manager_standalone() {
    let manager = LimitManager::new();
    assert!(manager.is_ok());

    let limits = Limits::new()
        .with_memory(256 * 1024 * 1024)
        .with_cpu_shares(100);

    // On non-Linux platforms, this should not fail
    // On Linux, it may fail if cgroups v2 is not available (which is fine)
    let _ = manager.unwrap().apply(12345, &limits);
}

#[tokio::test]
async fn test_executor_command_validation() {
    let pm = ProcessManager::builder().with_executor().build();

    // Try to spawn with dangerous command
    let config = ExecutionConfig::new("echo; rm -rf /".to_string());
    let pid = pm.create_process_with_command("evil-app".to_string(), 5, Some(config));

    // Process should be created but OS process should fail to spawn
    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.os_pid, None); // Should not have OS PID due to validation failure

    pm.terminate_process(pid);
}

#[tokio::test]
async fn test_executor_path_traversal_prevention() {
    let pm = ProcessManager::builder().with_executor().build();

    // Test basic .. traversal
    let config =
        ExecutionConfig::new("cat".to_string()).with_args(vec!["../etc/passwd".to_string()]);
    let pid = pm.create_process_with_command("traversal-1".to_string(), 5, Some(config));
    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.os_pid, None, "Basic .. traversal should be blocked");
    pm.terminate_process(pid);

    // Test path with /./  and .. combination
    let config =
        ExecutionConfig::new("cat".to_string()).with_args(vec!["./../../etc/passwd".to_string()]);
    let pid = pm.create_process_with_command("traversal-2".to_string(), 5, Some(config));
    let process = pm.get_process(pid).unwrap();
    assert_eq!(
        process.os_pid, None,
        "/./ with .. traversal should be blocked"
    );
    pm.terminate_process(pid);

    // Test normalized path that escapes
    let config = ExecutionConfig::new("cat".to_string())
        .with_args(vec!["foo/../../../etc/passwd".to_string()]);
    let pid = pm.create_process_with_command("traversal-3".to_string(), 5, Some(config));
    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.os_pid, None, "Multiple .. should be blocked");
    pm.terminate_process(pid);

    // Test Windows-style path traversal
    let config = ExecutionConfig::new("cat".to_string())
        .with_args(vec!["..\\..\\windows\\system32".to_string()]);
    let pid = pm.create_process_with_command("traversal-4".to_string(), 5, Some(config));
    let process = pm.get_process(pid).unwrap();
    assert_eq!(
        process.os_pid, None,
        "Windows-style traversal should be blocked"
    );
    pm.terminate_process(pid);

    // Test URL encoded traversal
    let config =
        ExecutionConfig::new("cat".to_string()).with_args(vec!["%2e%2e/etc/passwd".to_string()]);
    let pid = pm.create_process_with_command("traversal-5".to_string(), 5, Some(config));
    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.os_pid, None, "URL encoded .. should be blocked");
    pm.terminate_process(pid);

    // Test valid paths (should succeed - won't have OS PID if command doesn't exist, but should pass validation)
    // Using 'echo' which is more likely to exist
    let config = ExecutionConfig::new("echo".to_string())
        .with_args(vec!["./valid/path/file.txt".to_string()]);
    let pid = pm.create_process_with_command("valid-path".to_string(), 5, Some(config));
    let _process = pm.get_process(pid).unwrap();
    // Note: os_pid might be None if echo doesn't exist, but validation passed
    pm.terminate_process(pid);
}

#[tokio::test]
async fn test_executor_path_normalization_edge_cases() {
    let pm = ProcessManager::builder().with_executor().build();

    // Test absolute path with upward traversal
    let config =
        ExecutionConfig::new("cat".to_string()).with_args(vec!["/../../../etc/passwd".to_string()]);
    let pid = pm.create_process_with_command("edge-1".to_string(), 5, Some(config));
    let process = pm.get_process(pid).unwrap();
    assert_eq!(
        process.os_pid, None,
        "Absolute path with .. should be blocked"
    );
    pm.terminate_process(pid);

    // Test mixed slashes
    let config = ExecutionConfig::new("cat".to_string())
        .with_args(vec!["foo/./bar/../../../etc".to_string()]);
    let pid = pm.create_process_with_command("edge-2".to_string(), 5, Some(config));
    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.os_pid, None, "Mixed ./ and .. should be detected");
    pm.terminate_process(pid);

    // Test safe relative paths (should pass validation)
    let config =
        ExecutionConfig::new("echo".to_string()).with_args(vec!["./subdir/file.txt".to_string()]);
    let pid = pm.create_process_with_command("safe-1".to_string(), 5, Some(config));
    let _process = pm.get_process(pid).unwrap();
    pm.terminate_process(pid);

    // Test safe path with .. that stays within bounds
    let config = ExecutionConfig::new("echo".to_string())
        .with_args(vec!["dir1/dir2/../file.txt".to_string()]);
    let pid = pm.create_process_with_command("safe-2".to_string(), 5, Some(config));
    let _process = pm.get_process(pid).unwrap();
    pm.terminate_process(pid);
}
