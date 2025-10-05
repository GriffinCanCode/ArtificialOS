/*!
 * Process Executor Tests
 * Tests for OS-level process spawning and management
 */

use ai_os_kernel::executor::{ExecutionConfig, ProcessExecutor};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_spawn_simple_process() {
    let executor = ProcessExecutor::new();
    let config = ExecutionConfig::new("echo".to_string()).with_args(vec!["hello".to_string()]);

    let result = executor.spawn(1, "test-echo".to_string(), config);
    assert!(result.is_ok());

    let os_pid = result.unwrap();
    assert!(os_pid > 0);
    assert!(executor.is_running(1));

    // Wait for process to complete
    sleep(Duration::from_millis(100)).await;
    executor.cleanup();
}

#[tokio::test]
async fn test_spawn_multiple_processes() {
    let executor = ProcessExecutor::new();

    // Spawn 3 processes
    for i in 1..=3 {
        let config =
            ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);
        let result = executor.spawn(i, format!("test-sleep-{}", i), config);
        assert!(result.is_ok());
    }

    assert_eq!(executor.count(), 3);

    // Wait for processes to complete
    sleep(Duration::from_millis(200)).await;
    executor.cleanup();
}

#[tokio::test]
async fn test_kill_process() {
    let executor = ProcessExecutor::new();
    let config = ExecutionConfig::new("sleep".to_string()).with_args(vec!["10".to_string()]);

    executor.spawn(1, "test-sleep".to_string(), config).unwrap();
    assert!(executor.is_running(1));

    let result = executor.kill(1);
    assert!(result.is_ok());
    assert!(!executor.is_running(1));
}

#[tokio::test]
async fn test_get_os_pid() {
    let executor = ProcessExecutor::new();
    let config = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);

    let os_pid = executor.spawn(1, "test-sleep".to_string(), config).unwrap();
    assert_eq!(executor.get_os_pid(1), Some(os_pid));

    executor.kill(1).ok();
}

#[tokio::test]
async fn test_invalid_command() {
    let executor = ProcessExecutor::new();
    let config = ExecutionConfig::new("echo; rm -rf /".to_string());

    let result = executor.spawn(1, "test-evil".to_string(), config);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_command_with_env_vars() {
    let executor = ProcessExecutor::new();
    let config = ExecutionConfig::new("printenv".to_string())
        .with_args(vec!["TEST_VAR".to_string()])
        .with_env(vec![("TEST_VAR".to_string(), "test_value".to_string())]);

    let result = executor.spawn(1, "test-env".to_string(), config);
    assert!(result.is_ok());

    // Wait for process to complete
    sleep(Duration::from_millis(100)).await;
    executor.cleanup();
}

#[tokio::test]
async fn test_wait_for_completion() {
    let executor = ProcessExecutor::new();
    let config = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);

    executor.spawn(1, "test-sleep".to_string(), config).unwrap();

    let result = executor.wait(1);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // Exit code 0 for success
}

#[tokio::test]
async fn test_cleanup_zombie_processes() {
    let executor = ProcessExecutor::new();

    // Spawn short-lived process
    let config = ExecutionConfig::new("echo".to_string()).with_args(vec!["done".to_string()]);
    executor.spawn(1, "test-echo".to_string(), config).unwrap();

    // Wait for it to complete
    sleep(Duration::from_millis(200)).await;

    // Cleanup should remove completed process
    executor.cleanup();
    assert_eq!(executor.count(), 0);
}

#[tokio::test]
async fn test_process_isolation() {
    let executor = ProcessExecutor::new();

    // Spawn two independent processes
    let config1 = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.2".to_string()]);
    let os_pid1 = executor.spawn(1, "proc1".to_string(), config1).unwrap();

    let config2 = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.2".to_string()]);
    let os_pid2 = executor.spawn(2, "proc2".to_string(), config2).unwrap();

    // Verify different OS PIDs
    assert_ne!(os_pid1, os_pid2);
    assert_eq!(executor.count(), 2);

    // Kill one should not affect the other
    executor.kill(1).unwrap();
    assert!(!executor.is_running(1));
    assert!(executor.is_running(2));

    // Cleanup
    executor.kill(2).ok();
}
