/*!
 * Async Syscall Handler Tests
 * Tests for async syscall handling with Tokio
 */

use ai_os_kernel::syscalls::handlers::{AsyncSyscallHandler, AsyncSyscallHandlerRegistry};
use ai_os_kernel::syscalls::types::{Syscall, SyscallResult};
use ai_os_kernel::core::types::Pid;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

struct TestAsyncHandler;

impl AsyncSyscallHandler for TestAsyncHandler {
    fn handle_async(&self, _pid: Pid, syscall: &Syscall)
        -> Pin<Box<dyn Future<Output = Option<SyscallResult>> + Send + '_>> {
        Box::pin(async move {
            match syscall {
                Syscall::NetworkRequest { .. } => {
                    // Simulate async I/O operation
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    Some(SyscallResult::success(Some(b"response".to_vec())))
                }
                Syscall::GetProcessList => {
                    // Simulate async database query
                    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                    Some(SyscallResult::success(Some(b"[]".to_vec())))
                }
                _ => None,
            }
        })
    }

    fn name(&self) -> &'static str {
        "test_async_handler"
    }
}

struct AnotherAsyncHandler;

impl AsyncSyscallHandler for AnotherAsyncHandler {
    fn handle_async(&self, _pid: Pid, syscall: &Syscall)
        -> Pin<Box<dyn Future<Output = Option<SyscallResult>> + Send + '_>> {
        Box::pin(async move {
            match syscall {
                Syscall::ReadFile { .. } => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                    Some(SyscallResult::success(Some(b"file contents".to_vec())))
                }
                _ => None,
            }
        })
    }

    fn name(&self) -> &'static str {
        "another_async_handler"
    }
}

#[tokio::test]
async fn test_async_handler_network_request() {
    let handler = TestAsyncHandler;
    let syscall = Syscall::NetworkRequest {
        url: "http://example.com".to_string(),
    };

    let result = handler.handle_async(1, &syscall).await;

    assert!(result.is_some());
    match result.unwrap() {
        SyscallResult::Success { data } => {
            assert_eq!(data.unwrap(), b"response");
        }
        _ => panic!("Expected success"),
    }
}

#[tokio::test]
async fn test_async_handler_process_list() {
    let handler = TestAsyncHandler;
    let syscall = Syscall::GetProcessList;

    let result = handler.handle_async(1, &syscall).await;

    assert!(result.is_some());
    match result.unwrap() {
        SyscallResult::Success { .. } => {}
        _ => panic!("Expected success"),
    }
}

#[tokio::test]
async fn test_async_handler_returns_none() {
    let handler = TestAsyncHandler;
    let syscall = Syscall::ReadFile { path: "/test".into() };

    let result = handler.handle_async(1, &syscall).await;

    assert!(result.is_none());
}

#[tokio::test]
async fn test_async_handler_registry() {
    let registry = AsyncSyscallHandlerRegistry::new()
        .register(Arc::new(TestAsyncHandler));

    assert_eq!(registry.handler_count(), 1);
}

#[tokio::test]
async fn test_async_handler_registry_dispatch() {
    let registry = AsyncSyscallHandlerRegistry::new()
        .register(Arc::new(TestAsyncHandler));

    let syscall = Syscall::NetworkRequest {
        url: "http://example.com".to_string(),
    };

    let result = registry.dispatch(1, &syscall).await;

    assert!(result.is_some());
}

#[tokio::test]
async fn test_async_handler_registry_multiple_handlers() {
    let registry = AsyncSyscallHandlerRegistry::new()
        .register(Arc::new(TestAsyncHandler))
        .register(Arc::new(AnotherAsyncHandler));

    assert_eq!(registry.handler_count(), 2);

    // Test first handler
    let syscall1 = Syscall::NetworkRequest {
        url: "http://example.com".to_string(),
    };
    let result1 = registry.dispatch(1, &syscall1).await;
    assert!(result1.is_some());

    // Test second handler
    let syscall2 = Syscall::ReadFile { path: "/test".into() };
    let result2 = registry.dispatch(1, &syscall2).await;
    assert!(result2.is_some());
}

#[tokio::test]
async fn test_async_handler_registry_no_match() {
    let registry = AsyncSyscallHandlerRegistry::new()
        .register(Arc::new(TestAsyncHandler));

    let syscall = Syscall::KillProcess { target_pid: 123 };

    let result = registry.dispatch(1, &syscall).await;

    assert!(result.is_none());
}

#[tokio::test]
async fn test_async_handler_concurrent_requests() {
    let registry = Arc::new(
        AsyncSyscallHandlerRegistry::new()
            .register(Arc::new(TestAsyncHandler))
    );

    // Spawn multiple concurrent requests
    let mut tasks = vec![];

    for i in 0..10 {
        let reg = registry.clone();
        let task = tokio::spawn(async move {
            let syscall = Syscall::NetworkRequest {
                url: format!("http://example.com/{}", i),
            };
            reg.dispatch(i as Pid, &syscall).await
        });
        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        let result = task.await.unwrap();
        assert!(result.is_some());
    }
}

#[tokio::test]
async fn test_async_handler_timeout() {
    struct SlowHandler;

    impl AsyncSyscallHandler for SlowHandler {
        fn handle_async(&self, _pid: Pid, syscall: &Syscall)
            -> Pin<Box<dyn Future<Output = Option<SyscallResult>> + Send + '_>> {
            Box::pin(async move {
                match syscall {
                    Syscall::NetworkRequest { .. } => {
                        // Very slow operation
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        Some(SyscallResult::success(None))
                    }
                    _ => None,
                }
            })
        }

        fn name(&self) -> &'static str {
            "slow_handler"
        }
    }

    let registry = AsyncSyscallHandlerRegistry::new()
        .register(Arc::new(SlowHandler));

    let syscall = Syscall::NetworkRequest {
        url: "http://example.com".to_string(),
    };

    // Use timeout to avoid hanging
    let result = tokio::time::timeout(
        tokio::time::Duration::from_millis(100),
        registry.dispatch(1, &syscall)
    ).await;

    // Should timeout
    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_handler_error_handling() {
    struct ErrorHandler;

    impl AsyncSyscallHandler for ErrorHandler {
        fn handle_async(&self, _pid: Pid, syscall: &Syscall)
            -> Pin<Box<dyn Future<Output = Option<SyscallResult>> + Send + '_>> {
            Box::pin(async move {
                match syscall {
                    Syscall::NetworkRequest { .. } => {
                        Some(SyscallResult::error("Network error".to_string()))
                    }
                    _ => None,
                }
            })
        }

        fn name(&self) -> &'static str {
            "error_handler"
        }
    }

    let registry = AsyncSyscallHandlerRegistry::new()
        .register(Arc::new(ErrorHandler));

    let syscall = Syscall::NetworkRequest {
        url: "http://example.com".to_string(),
    };

    let result = registry.dispatch(1, &syscall).await;

    assert!(result.is_some());
    match result.unwrap() {
        SyscallResult::Error { message } => {
            assert_eq!(message, "Network error");
        }
        _ => panic!("Expected error"),
    }
}

