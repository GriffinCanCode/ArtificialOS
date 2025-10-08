/*!
 * gRPC Server
 * Exposes kernel syscalls to AI service via gRPC
 */

use std::sync::Arc;
use std::time::Duration;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, instrument};

use crate::monitoring::span_grpc;
use crate::process::ProcessManagerImpl as ProcessManager;
use crate::security::SandboxManager;
use crate::syscalls::SyscallExecutorWithIpc;

use crate::api::conversions::{proto_to_syscall_full, syscall_result_to_proto};
use crate::api::execution::{
    AsyncTaskManager, BatchExecutor, IoUringExecutor, IoUringManager, StreamingManager,
};
use crate::api::handlers::{
    async_handlers, process_handlers, sandbox_handlers, scheduler_handlers, streaming_handlers,
};
use crate::api::traits::ServerLifecycle;
use crate::api::types::{ApiError, ApiResult, ServerConfig};

// Include generated protobuf code
pub mod kernel_proto {
    tonic::include_proto!("kernel");
}

use kernel_proto::kernel_service_server::{KernelService, KernelServiceServer};
use kernel_proto::*;

/// gRPC service implementation
pub struct KernelServiceImpl {
    syscall_executor: SyscallExecutorWithIpc,
    process_manager: ProcessManager,
    sandbox_manager: SandboxManager,
    async_manager: AsyncTaskManager,
    streaming_manager: StreamingManager,
    batch_executor: BatchExecutor,
    iouring_manager: Arc<IoUringManager>,
}

impl KernelServiceImpl {
    pub fn new(
        syscall_executor: SyscallExecutorWithIpc,
        process_manager: ProcessManager,
        sandbox_manager: SandboxManager,
    ) -> Self {
        info!("gRPC service initialized");
        let async_manager = AsyncTaskManager::new(syscall_executor.clone());
        let streaming_manager = StreamingManager::new(syscall_executor.clone());
        let batch_executor = BatchExecutor::new(syscall_executor.clone());

        // Initialize io_uring manager for efficient I/O syscall completion
        let iouring_executor = Arc::new(IoUringExecutor::new(syscall_executor.clone()));
        let iouring_manager = Arc::new(IoUringManager::new(iouring_executor));

        Self {
            syscall_executor,
            process_manager,
            sandbox_manager,
            async_manager,
            streaming_manager,
            batch_executor,
            iouring_manager,
        }
    }

    /// Get the io_uring manager
    pub fn iouring_manager(&self) -> &Arc<IoUringManager> {
        &self.iouring_manager
    }
}

#[tonic::async_trait]
impl KernelService for KernelServiceImpl {
    type StreamEventsStream = tokio_stream::wrappers::ReceiverStream<Result<KernelEvent, Status>>;
    type StreamSyscallStream =
        tokio_stream::wrappers::ReceiverStream<Result<StreamSyscallChunk, Status>>;

    #[instrument(skip(self, request), fields(pid, syscall_type, trace_id))]
    async fn execute_syscall(
        &self,
        request: Request<SyscallRequest>,
    ) -> Result<Response<SyscallResponse>, Status> {
        let span = span_grpc("execute_syscall");
        let _guard = span.enter();

        let req = request.into_inner();
        let pid = req.pid;

        info!(
            pid = pid,
            trace_id = %span.trace_id(),
            "gRPC: Executing syscall"
        );

        // Convert proto syscall to internal syscall
        let syscall = match proto_to_syscall_full(&req) {
            Ok(s) => s,
            Err(e) => {
                if e.contains("not yet implemented") {
                    return Err(Status::unimplemented(e));
                } else if e.contains("No syscall provided") {
                    return Err(Status::invalid_argument(e));
                } else {
                    return Err(Status::invalid_argument(e));
                }
            }
        };

        // Execute syscall
        let result = self.syscall_executor.execute(pid, syscall);

        // Convert result to proto
        let response = syscall_result_to_proto(result);

        Ok(Response::new(response))
    }

    async fn create_process(
        &self,
        request: Request<CreateProcessRequest>,
    ) -> Result<Response<CreateProcessResponse>, Status> {
        process_handlers::handle_create_process(
            &self.process_manager,
            &self.sandbox_manager,
            request,
        )
        .await
    }

    async fn update_sandbox(
        &self,
        request: Request<UpdateSandboxRequest>,
    ) -> Result<Response<UpdateSandboxResponse>, Status> {
        sandbox_handlers::handle_update_sandbox(&self.sandbox_manager, request).await
    }

    async fn stream_events(
        &self,
        request: Request<EventStreamRequest>,
    ) -> Result<Response<Self::StreamEventsStream>, Status> {
        streaming_handlers::handle_stream_events(
            &self.process_manager,
            &self.sandbox_manager,
            request,
        )
        .await
    }

    async fn schedule_next(
        &self,
        request: Request<ScheduleNextRequest>,
    ) -> Result<Response<ScheduleNextResponse>, Status> {
        scheduler_handlers::handle_schedule_next(&self.process_manager, request).await
    }

    async fn get_scheduler_stats(
        &self,
        request: Request<GetSchedulerStatsRequest>,
    ) -> Result<Response<GetSchedulerStatsResponse>, Status> {
        scheduler_handlers::handle_get_scheduler_stats(&self.process_manager, request).await
    }

    async fn set_scheduling_policy(
        &self,
        request: Request<SetSchedulingPolicyRequest>,
    ) -> Result<Response<SetSchedulingPolicyResponse>, Status> {
        scheduler_handlers::handle_set_scheduling_policy(&self.process_manager, request).await
    }

    async fn stream_syscall(
        &self,
        request: Request<tonic::Streaming<StreamSyscallRequest>>,
    ) -> Result<Response<Self::StreamSyscallStream>, Status> {
        streaming_handlers::handle_stream_syscall(&self.streaming_manager, request).await
    }

    async fn execute_syscall_async(
        &self,
        request: Request<SyscallRequest>,
    ) -> Result<Response<AsyncSyscallResponse>, Status> {
        async_handlers::handle_execute_syscall_async(&self.async_manager, request).await
    }

    async fn get_async_status(
        &self,
        request: Request<AsyncStatusRequest>,
    ) -> Result<Response<AsyncStatusResponse>, Status> {
        async_handlers::handle_get_async_status(&self.async_manager, request).await
    }

    async fn cancel_async(
        &self,
        request: Request<AsyncCancelRequest>,
    ) -> Result<Response<AsyncCancelResponse>, Status> {
        async_handlers::handle_cancel_async(&self.async_manager, request).await
    }

    async fn execute_syscall_batch(
        &self,
        request: Request<BatchSyscallRequest>,
    ) -> Result<Response<BatchSyscallResponse>, Status> {
        async_handlers::handle_execute_syscall_batch(&self.batch_executor, request).await
    }

    async fn execute_syscall_iouring(
        &self,
        request: Request<SyscallRequest>,
    ) -> Result<Response<AsyncSyscallResponse>, Status> {
        async_handlers::handle_execute_syscall_iouring(
            &self.iouring_manager,
            &self.async_manager,
            request,
        )
        .await
    }

    async fn reap_completions(
        &self,
        request: Request<ReapCompletionsRequest>,
    ) -> Result<Response<ReapCompletionsResponse>, Status> {
        async_handlers::handle_reap_iouring_completions(&self.iouring_manager, request).await
    }

    async fn submit_iouring_batch(
        &self,
        request: Request<BatchSyscallRequest>,
    ) -> Result<Response<IoUringBatchResponse>, Status> {
        async_handlers::handle_submit_iouring_batch(&self.iouring_manager, request).await
    }
}

/// gRPC Server wrapper that implements ServerLifecycle trait
pub struct GrpcServer {
    config: ServerConfig,
    syscall_executor: SyscallExecutorWithIpc,
    process_manager: ProcessManager,
    sandbox_manager: SandboxManager,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl GrpcServer {
    pub fn new(
        config: ServerConfig,
        syscall_executor: SyscallExecutorWithIpc,
        process_manager: ProcessManager,
        sandbox_manager: SandboxManager,
    ) -> Self {
        Self {
            config,
            syscall_executor,
            process_manager,
            sandbox_manager,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

impl ServerLifecycle for GrpcServer {
    fn start(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ApiResult<()>> + Send + '_>> {
        Box::pin(async move {
            let service = KernelServiceImpl::new(
                self.syscall_executor.clone(),
                self.process_manager.clone(),
                self.sandbox_manager.clone(),
            );

            info!("gRPC server starting on {}", self.config.address);

            self.running
                .store(true, std::sync::atomic::Ordering::SeqCst);

            // Configure server with settings from ServerConfig
            Server::builder()
                .timeout(Duration::from_secs(self.config.timeout_secs))
                .http2_keepalive_interval(Some(Duration::from_secs(
                    self.config.keepalive_interval_secs,
                )))
                .http2_keepalive_timeout(Some(Duration::from_secs(
                    self.config.keepalive_timeout_secs,
                )))
                .http2_adaptive_window(Some(true))
                .tcp_nodelay(true)
                .add_service(KernelServiceServer::new(service))
                .serve(self.config.address)
                .await
                .map_err(|e| ApiError::InternalError(format!("Server error: {}", e)))?;

            self.running
                .store(false, std::sync::atomic::Ordering::SeqCst);

            Ok(())
        })
    }

    fn stop(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ApiResult<()>> + Send + '_>> {
        Box::pin(async move {
            // Note: Graceful shutdown would require holding a server handle
            // For now, we just mark as not running
            self.running
                .store(false, std::sync::atomic::Ordering::SeqCst);
            info!("gRPC server stopped");
            Ok(())
        })
    }

    fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn config(&self) -> &ServerConfig {
        &self.config
    }
}

/// Start the gRPC server (legacy function for backward compatibility)
pub async fn start_grpc_server(
    addr: std::net::SocketAddr,
    syscall_executor: SyscallExecutorWithIpc,
    process_manager: ProcessManager,
    sandbox_manager: SandboxManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::new(addr);
    let server = GrpcServer::new(config, syscall_executor, process_manager, sandbox_manager);

    server
        .start()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
