/*!
 * Async Task Management Tests
 * Tests for async syscall execution and task tracking
 */

use ai_os_kernel::api::async_task::{AsyncTaskManager, TaskStatus};
use ai_os_kernel::security::SandboxManager;
use ai_os_kernel::syscalls::{Syscall, SyscallExecutor, SyscallResult};
use std::time::Duration;
use tokio::time::sleep;

fn setup_executor() -> (SyscallExecutor, SandboxManager, u32) {
    let sandbox_manager = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox_manager.clone());
    let pid = 100;
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
    assert!(status.is_some());

    match status.unwrap() {
        (TaskStatus::Completed(SyscallResult::Success { .. }), progress) => {
            assert_eq!(progress, 1.0);
        }
        _ => panic!("Expected completed status"),
    }
}

#[tokio::test]
async fn test_async_task_cancellation() {
    let (executor, _, pid) = setup_executor();
    let manager = AsyncTaskManager::new(executor);

    // Submit long-running syscall
    let task_id = manager.submit(
        pid,
        Syscall::Sleep {
            duration_ms: 5000,
        },
    );

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
async fn test_async_task_cleanup() {
    let (executor, _, pid) = setup_executor();
    let manager = AsyncTaskManager::new(executor);

    // Submit and complete a task
    let task_id = manager.submit(pid, Syscall::GetCurrentTime);
    sleep(Duration::from_millis(100)).await;

    // Cleanup completed tasks
    manager.cleanup_completed();

    // Task should still be there initially
    let status = manager.get_status(&task_id);
    assert!(status.is_some());
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
