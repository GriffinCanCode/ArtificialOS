/*!
 * Async operation handler implementations
 */

use tonic::{Request, Response, Status};
use tracing::{info, instrument};
use crate::monitoring::{span_grpc, GrpcSpan};
use crate::api::server::grpc_server::kernel_proto::*;
use crate::api::execution::{AsyncTaskManager, BatchExecutor, TaskStatus};
use crate::api::conversions::{proto_to_syscall_simple, syscall_result_to_proto};
use crate::syscalls::SyscallResult;

#[instrument(skip(async_manager, request), fields(pid, task_id, trace_id))]
pub async fn handle_execute_syscall_async(
    async_manager: &AsyncTaskManager,
    request: Request<SyscallRequest>,
) -> Result<Response<AsyncSyscallResponse>, Status> {
    let span = span_grpc("execute_syscall_async");
    let _guard = span.enter();

    let req = request.into_inner();
    let pid = req.pid;

    info!(
        pid = pid,
        trace_id = %span.trace_id(),
        "gRPC: Submitting async syscall"
    );

    // Convert proto syscall to internal
    let syscall = match proto_to_syscall_simple(&req) {
        Ok(s) => s,
        Err(e) => {
            return Ok(Response::new(AsyncSyscallResponse {
                task_id: String::new(),
                accepted: false,
                error: e,
            }));
        }
    };

    let task_id = async_manager.submit(pid, syscall);

    Ok(Response::new(AsyncSyscallResponse {
        task_id,
        accepted: true,
        error: String::new(),
    }))
}

pub async fn handle_get_async_status(
    async_manager: &AsyncTaskManager,
    request: Request<AsyncStatusRequest>,
) -> Result<Response<AsyncStatusResponse>, Status> {
    let req = request.into_inner();

    match async_manager.get_status(&req.task_id) {
        Some((status, progress)) => {
            let (proto_status, result) = match status {
                TaskStatus::Pending => (async_status_response::Status::Pending, None),
                TaskStatus::Running => (async_status_response::Status::Running, None),
                TaskStatus::Completed(res) => (
                    async_status_response::Status::Completed,
                    Some(syscall_result_to_proto(res)),
                ),
                TaskStatus::Failed(msg) => (
                    async_status_response::Status::Failed,
                    Some(SyscallResponse {
                        result: Some(syscall_response::Result::Error(ErrorResult {
                            message: msg,
                        })),
                    }),
                ),
                TaskStatus::Cancelled => (async_status_response::Status::Cancelled, None),
            };

            Ok(Response::new(AsyncStatusResponse {
                status: proto_status as i32,
                result,
                progress,
            }))
        }
        None => Err(Status::not_found("Task not found")),
    }
}

#[instrument(skip(async_manager, request), fields(task_id, trace_id))]
pub async fn handle_cancel_async(
    async_manager: &AsyncTaskManager,
    request: Request<AsyncCancelRequest>,
) -> Result<Response<AsyncCancelResponse>, Status> {
    let span = span_grpc("cancel_async");
    let _guard = span.enter();

    let req = request.into_inner();

    info!(
        task_id = %req.task_id,
        trace_id = %span.trace_id(),
        "gRPC: Cancelling async task"
    );

    let cancelled = async_manager.cancel(&req.task_id);

    Ok(Response::new(AsyncCancelResponse {
        cancelled,
        error: if cancelled {
            String::new()
        } else {
            "Task not found or already completed".to_string()
        },
    }))
}

#[instrument(skip(batch_executor, request), fields(batch_size, parallel, trace_id))]
pub async fn handle_execute_syscall_batch(
    batch_executor: &BatchExecutor,
    request: Request<BatchSyscallRequest>,
) -> Result<Response<BatchSyscallResponse>, Status> {
    let span = span_grpc("execute_syscall_batch");
    let _guard = span.enter();

    let req = request.into_inner();
    let parallel = req.parallel;
    let batch_size = req.requests.len();

    info!(
        batch_size = batch_size,
        parallel = parallel,
        trace_id = %span.trace_id(),
        "gRPC: Executing batch syscalls"
    );

    let mut syscalls = Vec::new();
    for syscall_req in req.requests {
        let pid = syscall_req.pid;
        match proto_to_syscall_simple(&syscall_req) {
            Ok(syscall) => syscalls.push((pid, syscall)),
            Err(e) => {
                return Ok(Response::new(BatchSyscallResponse {
                    responses: vec![SyscallResponse {
                        result: Some(syscall_response::Result::Error(ErrorResult {
                            message: e,
                        })),
                    }],
                    success_count: 0,
                    failure_count: 1,
                }));
            }
        }
    }

    let results = batch_executor.execute_batch(syscalls, parallel).await;

    let mut success_count = 0;
    let mut failure_count = 0;
    let responses: Vec<_> = results
        .into_iter()
        .map(|r| {
            match &r {
                SyscallResult::Success { .. } => success_count += 1,
                _ => failure_count += 1,
            }
            syscall_result_to_proto(r)
        })
        .collect();

    Ok(Response::new(BatchSyscallResponse {
        responses,
        success_count,
        failure_count,
    }))
}
