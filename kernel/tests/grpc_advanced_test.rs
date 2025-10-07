/*!
 * gRPC Advanced Features Integration Tests
 * Tests for streaming, async, and batch gRPC endpoints
 */

use ai_os_kernel::api::grpc_server::{GrpcServer, KernelServiceImpl};
use ai_os_kernel::process::ProcessManagerImpl;
use ai_os_kernel::security::SandboxManager;
use ai_os_kernel::syscalls::SyscallExecutor;
use std::net::SocketAddr;
use tempfile::TempDir;

// Note: These are integration tests that would require a running gRPC server
// For now, we test the service implementation directly

fn setup_service() -> (KernelServiceImpl, SandboxManager, TempDir) {
    let sandbox_manager = SandboxManager::new();
    let executor = SyscallExecutor::new(sandbox_manager.clone());
    let process_manager = ProcessManagerImpl::new();
    let temp_dir = TempDir::new().unwrap();

    let service = KernelServiceImpl::new(executor, process_manager, sandbox_manager.clone());

    (service, sandbox_manager, temp_dir)
}

#[tokio::test]
async fn test_async_syscall_submission() {
    let (service, sandbox_manager, temp_dir) = setup_service();
    let pid = 100;

    // Setup sandbox
    let mut config = ai_os_kernel::security::SandboxConfig::standard(pid);
    let canonical_path = temp_dir.path().canonicalize().unwrap();
    config.allow_path(canonical_path);
    sandbox_manager.create_sandbox(config);

    // Create async request
    use ai_os_kernel::api::grpc_server::kernel_proto::*;
    use tonic::Request;

    let syscall_req = SyscallRequest {
        pid,
        syscall: Some(syscall_request::Syscall::GetCurrentTime(GetCurrentTimeCall {})),
    };

    let response = service
        .execute_syscall_async(Request::new(syscall_req))
        .await
        .expect("Should submit async");

    let async_resp = response.into_inner();
    assert!(async_resp.accepted);
    assert!(!async_resp.task_id.is_empty());
}

#[tokio::test]
async fn test_async_status_tracking() {
    let (service, _, _) = setup_service();
    let pid = 100;

    use ai_os_kernel::api::grpc_server::kernel_proto::*;
    use tonic::Request;

    // Submit task
    let syscall_req = SyscallRequest {
        pid,
        syscall: Some(syscall_request::Syscall::Sleep(SleepCall {
            duration_ms: 100,
        })),
    };

    let async_resp = service
        .execute_syscall_async(Request::new(syscall_req))
        .await
        .expect("Should submit")
        .into_inner();

    let task_id = async_resp.task_id;

    // Check initial status
    let status_req = AsyncStatusRequest {
        task_id: task_id.clone(),
    };

    let status_resp = service
        .get_async_status(Request::new(status_req.clone()))
        .await
        .expect("Should get status")
        .into_inner();

    // Should be pending or running
    assert!(
        status_resp.status == AsyncStatusResponse::Status::Pending as i32
            || status_resp.status == AsyncStatusResponse::Status::Running as i32
    );

    // Wait for completion
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let final_status = service
        .get_async_status(Request::new(status_req))
        .await
        .expect("Should get final status")
        .into_inner();

    assert_eq!(
        final_status.status,
        AsyncStatusResponse::Status::Completed as i32
    );
}

#[tokio::test]
async fn test_batch_execution_basic() {
    let (service, sandbox_manager, temp_dir) = setup_service();
    let pid = 100;

    // Setup sandbox
    let mut config = ai_os_kernel::security::SandboxConfig::standard(pid);
    let canonical_path = temp_dir.path().canonicalize().unwrap();
    config.allow_path(canonical_path);
    sandbox_manager.create_sandbox(config);

    use ai_os_kernel::api::grpc_server::kernel_proto::*;
    use tonic::Request;

    let test_file = temp_dir.path().join("batch_test.txt");

    let batch_req = BatchSyscallRequest {
        requests: vec![
            SyscallRequest {
                pid,
                syscall: Some(syscall_request::Syscall::WriteFile(WriteFileCall {
                    path: test_file.to_string_lossy().to_string(),
                    data: b"batch data".to_vec(),
                })),
            },
            SyscallRequest {
                pid,
                syscall: Some(syscall_request::Syscall::ReadFile(ReadFileCall {
                    path: test_file.to_string_lossy().to_string(),
                })),
            },
        ],
        parallel: false,
    };

    let response = service
        .execute_syscall_batch(Request::new(batch_req))
        .await
        .expect("Batch should execute");

    let batch_resp = response.into_inner();
    assert_eq!(batch_resp.responses.len(), 2);
    assert_eq!(batch_resp.success_count, 2);
    assert_eq!(batch_resp.failure_count, 0);
}

#[tokio::test]
async fn test_batch_parallel_execution() {
    let (service, _, _) = setup_service();
    let pid = 100;

    use ai_os_kernel::api::grpc_server::kernel_proto::*;
    use tonic::Request;

    // Create multiple independent syscalls
    let requests: Vec<_> = (0..10)
        .map(|_| SyscallRequest {
            pid,
            syscall: Some(syscall_request::Syscall::GetCurrentTime(GetCurrentTimeCall {})),
        })
        .collect();

    let batch_req = BatchSyscallRequest {
        requests,
        parallel: true,
    };

    let start = std::time::Instant::now();
    let response = service
        .execute_syscall_batch(Request::new(batch_req))
        .await
        .expect("Batch should execute");
    let duration = start.elapsed();

    let batch_resp = response.into_inner();
    assert_eq!(batch_resp.responses.len(), 10);
    assert_eq!(batch_resp.success_count, 10);

    // Should be fast due to parallelism
    assert!(duration.as_millis() < 1000);
}

#[tokio::test]
async fn test_async_cancellation() {
    let (service, _, _) = setup_service();
    let pid = 100;

    use ai_os_kernel::api::grpc_server::kernel_proto::*;
    use tonic::Request;

    // Submit long-running task
    let syscall_req = SyscallRequest {
        pid,
        syscall: Some(syscall_request::Syscall::Sleep(SleepCall {
            duration_ms: 5000,
        })),
    };

    let async_resp = service
        .execute_syscall_async(Request::new(syscall_req))
        .await
        .expect("Should submit")
        .into_inner();

    let task_id = async_resp.task_id;

    // Give it time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Cancel
    let cancel_req = AsyncCancelRequest {
        task_id: task_id.clone(),
    };

    let cancel_resp = service
        .cancel_async(Request::new(cancel_req))
        .await
        .expect("Should cancel")
        .into_inner();

    assert!(cancel_resp.cancelled);

    // Verify status
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let status_req = AsyncStatusRequest { task_id };
    let status_resp = service
        .get_async_status(Request::new(status_req))
        .await
        .expect("Should get status")
        .into_inner();

    assert_eq!(
        status_resp.status,
        AsyncStatusResponse::Status::Cancelled as i32
    );
}

#[tokio::test]
async fn test_batch_error_handling() {
    let (service, _, _) = setup_service();
    let pid = 999; // No sandbox

    use ai_os_kernel::api::grpc_server::kernel_proto::*;
    use tonic::Request;

    let batch_req = BatchSyscallRequest {
        requests: vec![
            SyscallRequest {
                pid,
                syscall: Some(syscall_request::Syscall::GetCurrentTime(GetCurrentTimeCall {})),
            },
            SyscallRequest {
                pid,
                syscall: Some(syscall_request::Syscall::ReadFile(ReadFileCall {
                    path: "/invalid/path.txt".to_string(),
                })),
            },
        ],
        parallel: false,
    };

    let response = service
        .execute_syscall_batch(Request::new(batch_req))
        .await
        .expect("Should handle errors gracefully");

    let batch_resp = response.into_inner();
    assert_eq!(batch_resp.responses.len(), 2);
    assert!(batch_resp.failure_count > 0);
}
