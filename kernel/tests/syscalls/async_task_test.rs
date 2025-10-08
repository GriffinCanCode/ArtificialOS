/*!
 * Async Task Management Tests
 * Tests for async syscall execution and task tracking
 */

use ai_os_kernel::api::execution::{AsyncTaskManager, TaskStatus};
use ai_os_kernel::security::traits::SandboxProvider;
use ai_os_kernel::security::{Capability, SandboxConfig, SandboxManager};
use ai_os_kernel::syscalls::{Syscall, SyscallExecutor, SyscallResult};
use std::time::Duration;
use tokio::time::sleep;

fn setup_executor() -> (SyscallExecutor, SandboxManager, u32) {
    let sandbox_manager = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox_manager.clone());
    let pid = 100;

    // Create sandbox with necessary capabilities for tests
    let mut config = SandboxConfig::minimal(pid);
    config.grant_capability(Capability::TimeAccess);
    config.grant_capability(Capability::SystemInfo);
    config.grant_capability(Capability::SpawnProcess);
    sandbox_manager.create_sandbox(config);

    (executor, sandbox_manager, pid)
}

#[tokio::test]
async fn test_async_task_immediate_completion() {
    let (executor, _, pid) = setup_executor();
    let manager = AsyncTaskManager::new(executor);

    // Submit fast syscall
    let task_id = manager.submit(pid, Syscall::GetCurrentTime);

    // Wait a bit for execution
    sleep(Duration::from_millis(100)).await;

    // Check status
    let status = manager.get_status(&task_id);
    assert!(status.is_some(), "Task should exist");

    match status.unwrap() {
        (TaskStatus::Completed(SyscallResult::Success { .. }), progress) => {
            assert_eq!(progress, 1.0);
        }
        (status, progress) => panic!("Expected completed status, got: {:?} with progress: {}", status, progress),
    }
}

#[tokio::test]
async fn test_async_task_cancellation() {
    let (executor, _, pid) = setup_executor();
    let manager = AsyncTaskManager::new(executor);

    // Submit long-running syscall
    let task_id = manager.submit(pid, Syscall::Sleep { duration_ms: 5000 });

    // Give it time to start
    sleep(Duration::from_millis(50)).await;

    // Cancel it
    let cancelled = manager.cancel(&task_id);
    assert!(cancelled, "Should successfully cancel");

    // Wait a bit
    sleep(Duration::from_millis(100)).await;

    // Verify cancelled status
    if let Some((status, _)) = manager.get_status(&task_id) {
        match status {
            TaskStatus::Cancelled => (),
            _ => panic!("Expected cancelled status, got: {:?}", status),
        }
    }
}

#[tokio::test]
async fn test_async_task_not_found() {
    let (executor, _, _) = setup_executor();
    let manager = AsyncTaskManager::new(executor);

    let status = manager.get_status("nonexistent-task-id");
    assert!(status.is_none());
}

#[tokio::test]
async fn test_async_task_multiple_concurrent() {
    let (executor, _, pid) = setup_executor();
    let manager = AsyncTaskManager::new(executor);

    // Submit multiple tasks
    let task1 = manager.submit(pid, Syscall::GetCurrentTime);
    let task2 = manager.submit(pid, Syscall::GetSystemInfo);
    let task3 = manager.submit(pid, Syscall::GetUptime);

    // Wait for completion
    sleep(Duration::from_millis(200)).await;

    // All should complete
    for task_id in &[task1, task2, task3] {
        let status = manager.get_status(task_id);
        assert!(status.is_some());
        match status.unwrap().0 {
            TaskStatus::Completed(_) => (),
            other => panic!("Expected completed, got: {:?}", other),
        }
    }
}

#[tokio::test]
async fn test_async_task_cleanup_respects_ttl() {
    let (executor, _, pid) = setup_executor();
    // Use short TTL for testing (1 second)
    let manager = AsyncTaskManager::with_ttl(executor, Duration::from_secs(1));

    // Submit and complete a task
    let task_id = manager.submit(pid, Syscall::GetCurrentTime);
    sleep(Duration::from_millis(100)).await;

    // Verify task exists and is completed
    let status = manager.get_status(&task_id);
    assert!(status.is_some());
    assert!(matches!(status.unwrap().0, TaskStatus::Completed(_)));

    // Cleanup immediately - task should still exist (not expired yet)
    let cleaned = manager.cleanup_completed();
    assert_eq!(cleaned, 0, "No tasks should be cleaned yet");
    assert!(manager.get_status(&task_id).is_some(), "Task should still exist");

    // Wait for TTL to expire
    sleep(Duration::from_secs(2)).await;

    // Now cleanup should remove expired task
    let cleaned = manager.cleanup_completed();
    assert_eq!(cleaned, 1, "One expired task should be cleaned");
    assert!(manager.get_status(&task_id).is_none(), "Task should be gone");
}

#[tokio::test]
async fn test_async_task_error_handling() {
    let (executor, sandbox_manager, _) = setup_executor();
    let pid = 999; // No sandbox configured
    let manager = AsyncTaskManager::new(executor);

    // Submit syscall that should fail
    let task_id = manager.submit(
        pid,
        Syscall::ReadFile {
            path: "/nonexistent/file.txt".into(),
        },
    );

    sleep(Duration::from_millis(200)).await;

    // Should complete with error
    let status = manager.get_status(&task_id);
    assert!(status.is_some());
    match status.unwrap().0 {
        TaskStatus::Completed(SyscallResult::PermissionDenied { .. })
        | TaskStatus::Completed(SyscallResult::Error { .. }) => (),
        other => panic!("Expected error result, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_async_task_background_cleanup() {
    let (executor, _, pid) = setup_executor();
    // Use very short TTL and cleanup interval for testing
    let manager = AsyncTaskManager::with_ttl(executor, Duration::from_millis(500));

    // Submit multiple tasks
    let task1 = manager.submit(pid, Syscall::GetCurrentTime);
    let task2 = manager.submit(pid, Syscall::GetSystemInfo);
    let task3 = manager.submit(pid, Syscall::GetUptime);

    // Wait for completion
    sleep(Duration::from_millis(200)).await;

    // All tasks should exist
    assert!(manager.get_status(&task1).is_some());
    assert!(manager.get_status(&task2).is_some());
    assert!(manager.get_status(&task3).is_some());

    // Wait for TTL + some margin for background cleanup to run
    // Background cleanup runs every 5 minutes by default, but manual cleanup works immediately
    sleep(Duration::from_millis(600)).await;

    // Manual cleanup to verify TTL expiration
    let cleaned = manager.cleanup_completed();
    assert!(cleaned >= 3, "Should clean at least 3 expired tasks");
}

#[tokio::test]
async fn test_async_task_immediate_cleanup() {
    let (executor, _, pid) = setup_executor();
    let manager = AsyncTaskManager::new(executor);

    // Submit and complete tasks
    let task1 = manager.submit(pid, Syscall::GetCurrentTime);
    let task2 = manager.submit(pid, Syscall::GetSystemInfo);
    sleep(Duration::from_millis(200)).await;

    // Verify both tasks completed
    assert!(matches!(
        manager.get_status(&task1).unwrap().0,
        TaskStatus::Completed(_)
    ));
    assert!(matches!(
        manager.get_status(&task2).unwrap().0,
        TaskStatus::Completed(_)
    ));

    // Immediate cleanup (ignores TTL)
    let cleaned = manager.cleanup_completed_immediate();
    assert_eq!(cleaned, 2, "Should immediately clean 2 completed tasks");

    // Tasks should be gone
    assert!(manager.get_status(&task1).is_none());
    assert!(manager.get_status(&task2).is_none());
}

#[tokio::test]
async fn test_async_task_stats() {
    let (executor, _, pid) = setup_executor();
    let manager = AsyncTaskManager::new(executor);

    // Initially no tasks
    let stats = manager.task_stats();
    assert_eq!(stats.total, 0);

    // Submit fast and slow tasks
    let _fast1 = manager.submit(pid, Syscall::GetCurrentTime);
    let _fast2 = manager.submit(pid, Syscall::GetSystemInfo);
    let slow = manager.submit(pid, Syscall::Sleep { duration_ms: 2000 });

    // Give fast tasks time to complete
    sleep(Duration::from_millis(200)).await;

    let stats = manager.task_stats();
    assert_eq!(stats.total, 3, "Should have 3 tasks total");
    assert_eq!(stats.completed, 2, "Should have 2 completed tasks");
    assert_eq!(stats.running, 1, "Should have 1 running task");

    // Cancel the slow task
    manager.cancel(&slow);
    sleep(Duration::from_millis(100)).await;

    let stats = manager.task_stats();
    assert_eq!(stats.cancelled, 1, "Should have 1 cancelled task");
}

#[tokio::test]
async fn test_async_task_process_cleanup_sets_timestamp() {
    let (executor, _, pid) = setup_executor();
    let manager = AsyncTaskManager::with_ttl(executor, Duration::from_secs(1));

    // Submit a long-running task
    let _task = manager.submit(pid, Syscall::Sleep { duration_ms: 5000 });
    sleep(Duration::from_millis(100)).await;

    // Verify task exists
    assert_eq!(manager.task_count(), 1);

    // Cleanup process tasks (simulates process termination)
    let cleaned = manager.cleanup_process_tasks(pid);
    assert_eq!(cleaned, 1, "Should clean 1 task for terminated process");

    // Task should be immediately removed
    assert_eq!(manager.task_count(), 0);
}

#[tokio::test]
async fn test_async_task_cancelled_task_expires() {
    let (executor, _, pid) = setup_executor();
    let manager = AsyncTaskManager::with_ttl(executor, Duration::from_millis(500));

    // Submit and cancel a task
    let task_id = manager.submit(pid, Syscall::Sleep { duration_ms: 5000 });
    sleep(Duration::from_millis(50)).await;
    manager.cancel(&task_id);
    sleep(Duration::from_millis(100)).await;

    // Verify cancelled
    assert!(matches!(
        manager.get_status(&task_id).unwrap().0,
        TaskStatus::Cancelled
    ));

    // Wait for TTL to expire
    sleep(Duration::from_millis(600)).await;

    // Cleanup should remove cancelled task
    let cleaned = manager.cleanup_completed();
    assert_eq!(cleaned, 1, "Should clean 1 expired cancelled task");
    assert!(manager.get_status(&task_id).is_none());
}
