/*!
 * Async operation handler implementations
 */

use crate::api::conversions::{proto_to_syscall_simple, syscall_result_to_proto};
use crate::api::execution::{
    AsyncTaskManager, BatchExecutor, IoUringManager, SyscallOpType, SyscallSubmissionEntry,
    TaskStatus,
};
use crate::api::server::grpc_server::kernel_proto::*;
use crate::monitoring::span_grpc;
use crate::syscalls::{Syscall, SyscallResult};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{info, instrument, warn};

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
                        result: Some(syscall_response::Result::Error(ErrorResult { message: e })),
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

/// io_uring-style syscall submission
///
/// This handler tries to convert syscalls to io_uring operations when beneficial.
/// Falls back to regular async execution for non-I/O operations.
#[instrument(skip(iouring_manager, async_manager, request), fields(pid, trace_id))]
pub async fn handle_execute_syscall_iouring(
    iouring_manager: &Arc<IoUringManager>,
    async_manager: &AsyncTaskManager,
    request: Request<SyscallRequest>,
) -> Result<Response<AsyncSyscallResponse>, Status> {
    let span = span_grpc("execute_syscall_iouring");
    let _guard = span.enter();

    let req = request.into_inner();
    let pid = req.pid;

    info!(
        pid = pid,
        trace_id = %span.trace_id(),
        "gRPC: Submitting io_uring syscall"
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

    // Try to convert to io_uring operation
    let op = match syscall_to_iouring_op(&syscall) {
        Some(op) if op.is_io_bound() => op,
        _ => {
            // Not an I/O operation, fall back to regular async
            info!("Falling back to regular async execution for non-I/O syscall");
            let task_id = async_manager.submit(pid, syscall);
            return Ok(Response::new(AsyncSyscallResponse {
                task_id,
                accepted: true,
                error: String::new(),
            }));
        }
    };

    // Submit to io_uring
    let entry = SyscallSubmissionEntry::new(pid, op, 0);
    match iouring_manager.submit(pid, entry) {
        Ok(seq) => {
            let task_id = format!("iouring_{}", seq);
            info!("io_uring operation submitted with seq: {}", seq);

            Ok(Response::new(AsyncSyscallResponse {
                task_id,
                accepted: true,
                error: String::new(),
            }))
        }
        Err(e) => {
            warn!("io_uring submission failed: {}, falling back to async", e);
            // Fall back to regular async on error
            let task_id = async_manager.submit(pid, syscall);
            Ok(Response::new(AsyncSyscallResponse {
                task_id,
                accepted: true,
                error: String::new(),
            }))
        }
    }
}

/// Get io_uring operation status
///
/// Supports both io_uring task IDs (iouring_<seq>) and regular async task IDs
#[instrument(
    skip(_iouring_manager, async_manager, request),
    fields(task_id, trace_id)
)]
pub async fn handle_get_iouring_status(
    _iouring_manager: &Arc<IoUringManager>,
    async_manager: &AsyncTaskManager,
    request: Request<AsyncStatusRequest>,
) -> Result<Response<AsyncStatusResponse>, Status> {
    let span = span_grpc("get_iouring_status");
    let _guard = span.enter();

    let req = request.into_inner();
    let task_id = &req.task_id;

    info!(
        task_id = %task_id,
        trace_id = %span.trace_id(),
        "gRPC: Getting io_uring status"
    );

    // Check if this is an io_uring task
    if let Some(seq_str) = task_id.strip_prefix("iouring_") {
        if let Ok(_seq) = seq_str.parse::<u64>() {
            // Parse PID from context or assume it's in task_id
            // For now, try to reap completions and find this sequence
            // In production, we'd need better tracking

            // This is a simplified version - in production we'd track pid->seq mappings
            return Err(Status::unimplemented(
                "io_uring status check requires PID tracking - use reap_completions",
            ));
        }
    }

    // Fall back to regular async status check
    handle_get_async_status(async_manager, Request::new(req)).await
}

/// Reap io_uring completions
///
/// Returns all pending completions for a process
#[instrument(skip(iouring_manager, request), fields(pid, max_completions, trace_id))]
pub async fn handle_reap_iouring_completions(
    iouring_manager: &Arc<IoUringManager>,
    request: Request<ReapCompletionsRequest>,
) -> Result<Response<ReapCompletionsResponse>, Status> {
    let span = span_grpc("reap_iouring_completions");
    let _guard = span.enter();

    let req = request.into_inner();
    let pid = req.pid;
    let max = if req.max_completions > 0 {
        Some(req.max_completions as usize)
    } else {
        None
    };

    info!(
        pid = pid,
        max = ?max,
        trace_id = %span.trace_id(),
        "gRPC: Reaping io_uring completions"
    );

    match iouring_manager.reap_completions(pid, max) {
        Ok(completions) => {
            let responses: Vec<_> = completions
                .into_iter()
                .map(|c| {
                    let result = syscall_result_to_proto(c.result);
                    IoUringCompletion {
                        seq: c.seq,
                        user_data: c.user_data,
                        result: Some(result),
                    }
                })
                .collect();

            let count = responses.len() as u32;

            Ok(Response::new(ReapCompletionsResponse {
                completions: responses,
                count,
            }))
        }
        Err(e) => Err(Status::internal(format!(
            "Failed to reap completions: {}",
            e
        ))),
    }
}

/// Submit batch of io_uring operations
#[instrument(skip(iouring_manager, request), fields(pid, batch_size, trace_id))]
pub async fn handle_submit_iouring_batch(
    iouring_manager: &Arc<IoUringManager>,
    request: Request<BatchSyscallRequest>,
) -> Result<Response<IoUringBatchResponse>, Status> {
    let span = span_grpc("submit_iouring_batch");
    let _guard = span.enter();

    let req = request.into_inner();
    let batch_size = req.requests.len();

    info!(
        batch_size = batch_size,
        trace_id = %span.trace_id(),
        "gRPC: Submitting io_uring batch"
    );

    let mut entries = Vec::new();
    let mut first_pid = None;

    for syscall_req in req.requests {
        let pid = syscall_req.pid;
        if first_pid.is_none() {
            first_pid = Some(pid);
        }

        let syscall = match proto_to_syscall_simple(&syscall_req) {
            Ok(s) => s,
            Err(e) => {
                return Err(Status::invalid_argument(format!("Invalid syscall: {}", e)));
            }
        };

        if let Some(op) = syscall_to_iouring_op(&syscall) {
            if op.is_io_bound() {
                let entry = SyscallSubmissionEntry::new(pid, op, 0);
                entries.push(entry);
            } else {
                return Err(Status::invalid_argument(
                    "Batch contains non-I/O operations",
                ));
            }
        } else {
            return Err(Status::invalid_argument(
                "Batch contains operations not suitable for io_uring",
            ));
        }
    }

    let pid = first_pid.ok_or_else(|| Status::invalid_argument("Empty batch"))?;

    match iouring_manager.submit_batch(pid, entries) {
        Ok(seqs) => Ok(Response::new(IoUringBatchResponse {
            sequences: seqs,
            accepted: true,
            error: String::new(),
        })),
        Err(e) => Ok(Response::new(IoUringBatchResponse {
            sequences: vec![],
            accepted: false,
            error: format!("Batch submission failed: {}", e),
        })),
    }
}

/// Convert a syscall to an io_uring operation
fn syscall_to_iouring_op(syscall: &Syscall) -> Option<SyscallOpType> {
    match syscall {
        // File I/O
        Syscall::ReadFile { path } => Some(SyscallOpType::ReadFile { path: path.clone() }),
        Syscall::WriteFile { path, data } => Some(SyscallOpType::WriteFile {
            path: path.clone(),
            data: data.clone(),
        }),
        Syscall::Open { path, flags, mode } => Some(SyscallOpType::Open {
            path: path.clone(),
            flags: *flags,
            mode: *mode,
        }),
        Syscall::Close { fd } => Some(SyscallOpType::Close { fd: *fd }),

        // Network I/O
        Syscall::Send {
            sockfd,
            data,
            flags,
        } => Some(SyscallOpType::Send {
            sockfd: *sockfd,
            data: data.clone(),
            flags: *flags,
        }),
        Syscall::Recv {
            sockfd,
            size,
            flags,
        } => Some(SyscallOpType::Recv {
            sockfd: *sockfd,
            size: *size,
            flags: *flags,
        }),
        Syscall::Accept { sockfd } => Some(SyscallOpType::Accept { sockfd: *sockfd }),
        Syscall::Connect { sockfd, address } => Some(SyscallOpType::Connect {
            sockfd: *sockfd,
            address: address.clone(),
        }),
        Syscall::SendTo {
            sockfd,
            data,
            address,
            flags,
        } => Some(SyscallOpType::SendTo {
            sockfd: *sockfd,
            data: data.clone(),
            address: address.clone(),
            flags: *flags,
        }),
        Syscall::RecvFrom {
            sockfd,
            size,
            flags,
        } => Some(SyscallOpType::RecvFrom {
            sockfd: *sockfd,
            size: *size,
            flags: *flags,
        }),

        // IPC - Note: SendQueue/ReceiveQueue would need queue_id, so not included here
        // Direct IPC messaging syscalls don't exist in current implementation

        // Not suitable for io_uring
        _ => None,
    }
}
