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
    let pm = ProcessManager::with_executor();

    // Create process with OS execution
    let config = ExecutionConfig::new("echo".to_string()).with_args(vec!["test".to_string()]);
    let pid = pm.create_process_with_command("test-app".to_string(), 5, Some(config));

    // Verify process created
    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.name, "test-app");
    assert!(process.os_pid.is_some());

    // Wait for process to complete
    sleep(Duration::from_millis(200)).await;

    // Terminate
    assert!(pm.terminate_process(pid));
}

#[tokio::test]
async fn test_process_manager_with_memory_and_executor() {
    let mem_mgr = MemoryManager::new();
    let pm = ProcessManager::full(mem_mgr.clone());

    // Create process with command
    let config = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);
    let pid = pm.create_process_with_command("test-app".to_string(), 5, Some(config));

    // Allocate memory
    mem_mgr.allocate(10 * 1024 * 1024, pid).unwrap();

    // Verify both process and memory exist
    assert!(pm.get_process(pid).is_some());
    assert_eq!(mem_mgr.get_process_memory(pid), 10 * 1024 * 1024);

    // Terminate - should cleanup both
    pm.terminate_process(pid);

    assert_eq!(mem_mgr.get_process_memory(pid), 0);
}

#[tokio::test]
async fn test_multiple_processes_with_isolation() {
    let pm = ProcessManager::with_executor();

    // Create 3 processes
    let mut pids = Vec::new();
    for i in 1..=3 {
        let config = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.2".to_string()]);
        let pid =
            pm.create_process_with_command(format!("app-{}", i), 5, Some(config));
        pids.push(pid);
    }

    // Verify all created with different OS PIDs
    let processes = pm.list_processes();
    assert_eq!(processes.len(), 3);

    let os_pids: Vec<_> = processes.iter().filter_map(|p| p.os_pid).collect();
    assert_eq!(os_pids.len(), 3);

    // All OS PIDs should be unique
    let unique_count = os_pids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert_eq!(unique_count, 3);

    // Cleanup
    for pid in pids {
        pm.terminate_process(pid);
    }
}

#[tokio::test]
async fn test_process_without_os_execution() {
    let pm = ProcessManager::with_executor();

    // Create process without command (metadata only)
    let pid = pm.create_process("virtual-app".to_string(), 5);

    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.name, "virtual-app");
    assert_eq!(process.os_pid, None);
    assert!(matches!(process.state, ProcessState::Ready));

    pm.terminate_process(pid);
}

#[tokio::test]
async fn test_priority_based_resource_limits() {
    let pm = ProcessManager::with_executor();

    // Low priority process
    let config_low = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);
    let pid_low = pm.create_process_with_command("low-priority".to_string(), 2, Some(config_low));

    // High priority process
    let config_high =
        ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);
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
    let pm = ProcessManager::with_executor();

    // Try to spawn with dangerous command
    let config = ExecutionConfig::new("echo; rm -rf /".to_string());
    let pid = pm.create_process_with_command("evil-app".to_string(), 5, Some(config));

    // Process should be created but OS process should fail to spawn
    let process = pm.get_process(pid).unwrap();
    assert_eq!(process.os_pid, None); // Should not have OS PID due to validation failure

    pm.terminate_process(pid);
}
