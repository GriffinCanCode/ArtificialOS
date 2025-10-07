/*!
 * gRPC Server
 * Exposes kernel syscalls to AI service via gRPC
 */

use log::info;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tonic::{transport::Server, Request, Response, Status};

use crate::process::ProcessManagerImpl as ProcessManager;
use crate::security::{
    Capability as SandboxCapability, NetworkRule, SandboxConfig, SandboxManager, SandboxProvider,
};
use crate::syscalls::{Syscall, SyscallExecutor, SyscallResult};

use super::async_task::{AsyncTaskManager, TaskStatus};
use super::batch::BatchExecutor;
use super::streaming::StreamingManager;
use super::traits::ServerLifecycle;
use super::types::{ApiError, ApiResult, ServerConfig};

// Include generated protobuf code
pub mod kernel_proto {
    tonic::include_proto!("kernel");
}

use kernel_proto::kernel_service_server::{KernelService, KernelServiceServer};
use kernel_proto::*;

/// gRPC service implementation
pub struct KernelServiceImpl {
    syscall_executor: SyscallExecutor,
    process_manager: ProcessManager,
    sandbox_manager: SandboxManager,
    async_manager: AsyncTaskManager,
    streaming_manager: StreamingManager,
    batch_executor: BatchExecutor,
}

impl KernelServiceImpl {
    pub fn new(
        syscall_executor: SyscallExecutor,
        process_manager: ProcessManager,
        sandbox_manager: SandboxManager,
    ) -> Self {
        info!("gRPC service initialized");
        let async_manager = AsyncTaskManager::new(syscall_executor.clone());
        let streaming_manager = StreamingManager::new(syscall_executor.clone());
        let batch_executor = BatchExecutor::new(syscall_executor.clone());

        Self {
            syscall_executor,
            process_manager,
            sandbox_manager,
            async_manager,
            streaming_manager,
            batch_executor,
        }
    }
}

#[tonic::async_trait]
impl KernelService for KernelServiceImpl {
    type StreamEventsStream = tokio_stream::wrappers::ReceiverStream<Result<KernelEvent, Status>>;
    type StreamSyscallStream = tokio_stream::wrappers::ReceiverStream<Result<StreamSyscallChunk, Status>>;

    async fn execute_syscall(
        &self,
        request: Request<SyscallRequest>,
    ) -> Result<Response<SyscallResponse>, Status> {
        let req = request.into_inner();
        let pid = req.pid;

        info!("gRPC: Executing syscall for PID {}", pid);

        // Convert proto syscall to internal syscall
        let syscall = match req.syscall {
            Some(syscall_request::Syscall::ReadFile(call)) => Syscall::ReadFile {
                path: PathBuf::from(call.path),
            },
            Some(syscall_request::Syscall::WriteFile(call)) => Syscall::WriteFile {
                path: PathBuf::from(call.path),
                data: call.data,
            },
            Some(syscall_request::Syscall::CreateFile(call)) => Syscall::CreateFile {
                path: PathBuf::from(call.path),
            },
            Some(syscall_request::Syscall::DeleteFile(call)) => Syscall::DeleteFile {
                path: PathBuf::from(call.path),
            },
            Some(syscall_request::Syscall::ListDirectory(call)) => Syscall::ListDirectory {
                path: PathBuf::from(call.path),
            },
            Some(syscall_request::Syscall::FileExists(call)) => Syscall::FileExists {
                path: PathBuf::from(call.path),
            },
            Some(syscall_request::Syscall::FileStat(call)) => Syscall::FileStat {
                path: PathBuf::from(call.path),
            },
            Some(syscall_request::Syscall::MoveFile(call)) => Syscall::MoveFile {
                source: PathBuf::from(call.source),
                destination: PathBuf::from(call.destination),
            },
            Some(syscall_request::Syscall::CopyFile(call)) => Syscall::CopyFile {
                source: PathBuf::from(call.source),
                destination: PathBuf::from(call.destination),
            },
            Some(syscall_request::Syscall::CreateDirectory(call)) => Syscall::CreateDirectory {
                path: PathBuf::from(call.path),
            },
            Some(syscall_request::Syscall::RemoveDirectory(call)) => Syscall::RemoveDirectory {
                path: PathBuf::from(call.path),
            },
            Some(syscall_request::Syscall::GetWorkingDirectory(_)) => Syscall::GetWorkingDirectory,
            Some(syscall_request::Syscall::SetWorkingDirectory(call)) => {
                Syscall::SetWorkingDirectory {
                    path: PathBuf::from(call.path),
                }
            }
            Some(syscall_request::Syscall::TruncateFile(call)) => Syscall::TruncateFile {
                path: PathBuf::from(call.path),
                size: call.size,
            },
            Some(syscall_request::Syscall::SpawnProcess(call)) => Syscall::SpawnProcess {
                command: call.command,
                args: call.args,
            },
            Some(syscall_request::Syscall::KillProcess(call)) => Syscall::KillProcess {
                target_pid: call.target_pid,
            },
            Some(syscall_request::Syscall::GetProcessInfo(call)) => Syscall::GetProcessInfo {
                target_pid: call.target_pid,
            },
            Some(syscall_request::Syscall::GetProcessList(_)) => Syscall::GetProcessList,
            Some(syscall_request::Syscall::SetProcessPriority(call)) => {
                Syscall::SetProcessPriority {
                    target_pid: call.target_pid,
                    priority: call.priority as u8,
                }
            }
            Some(syscall_request::Syscall::GetProcessState(call)) => Syscall::GetProcessState {
                target_pid: call.target_pid,
            },
            Some(syscall_request::Syscall::GetProcessStats(call)) => Syscall::GetProcessStats {
                target_pid: call.target_pid,
            },
            Some(syscall_request::Syscall::WaitProcess(call)) => Syscall::WaitProcess {
                target_pid: call.target_pid,
                timeout_ms: call.timeout_ms,
            },
            Some(syscall_request::Syscall::GetSystemInfo(_)) => Syscall::GetSystemInfo,
            Some(syscall_request::Syscall::GetCurrentTime(_)) => Syscall::GetCurrentTime,
            Some(syscall_request::Syscall::GetEnvVar(call)) => {
                Syscall::GetEnvironmentVar { key: call.key }
            }
            Some(syscall_request::Syscall::SetEnvVar(call)) => Syscall::SetEnvironmentVar {
                key: call.key,
                value: call.value,
            },
            // Time operations
            Some(syscall_request::Syscall::Sleep(call)) => Syscall::Sleep {
                duration_ms: call.duration_ms,
            },
            Some(syscall_request::Syscall::GetUptime(_)) => Syscall::GetUptime,
            // Memory operations
            Some(syscall_request::Syscall::GetMemoryStats(_)) => Syscall::GetMemoryStats,
            Some(syscall_request::Syscall::GetProcessMemoryStats(call)) => {
                Syscall::GetProcessMemoryStats {
                    target_pid: call.target_pid,
                }
            }
            Some(syscall_request::Syscall::TriggerGc(call)) => Syscall::TriggerGC {
                target_pid: call.target_pid,
            },
            // Signal operations
            Some(syscall_request::Syscall::SendSignal(call)) => Syscall::SendSignal {
                target_pid: call.target_pid,
                signal: call.signal,
            },
            Some(syscall_request::Syscall::NetworkRequest(call)) => {
                Syscall::NetworkRequest { url: call.url }
            }
            // Network - Sockets
            Some(syscall_request::Syscall::Socket(call)) => Syscall::Socket {
                domain: call.domain,
                socket_type: call.socket_type,
                protocol: call.protocol,
            },
            Some(syscall_request::Syscall::Bind(call)) => Syscall::Bind {
                sockfd: call.sockfd,
                address: call.address,
            },
            Some(syscall_request::Syscall::Listen(call)) => Syscall::Listen {
                sockfd: call.sockfd,
                backlog: call.backlog,
            },
            Some(syscall_request::Syscall::Accept(call)) => Syscall::Accept {
                sockfd: call.sockfd,
            },
            Some(syscall_request::Syscall::Connect(call)) => Syscall::Connect {
                sockfd: call.sockfd,
                address: call.address,
            },
            Some(syscall_request::Syscall::Send(call)) => Syscall::Send {
                sockfd: call.sockfd,
                data: call.data,
                flags: call.flags,
            },
            Some(syscall_request::Syscall::Recv(call)) => Syscall::Recv {
                sockfd: call.sockfd,
                size: call.size as usize,
                flags: call.flags,
            },
            Some(syscall_request::Syscall::SendTo(call)) => Syscall::SendTo {
                sockfd: call.sockfd,
                data: call.data,
                address: call.address,
                flags: call.flags,
            },
            Some(syscall_request::Syscall::RecvFrom(call)) => Syscall::RecvFrom {
                sockfd: call.sockfd,
                size: call.size as usize,
                flags: call.flags,
            },
            Some(syscall_request::Syscall::CloseSocket(call)) => Syscall::CloseSocket {
                sockfd: call.sockfd,
            },
            Some(syscall_request::Syscall::SetSockOpt(call)) => Syscall::SetSockOpt {
                sockfd: call.sockfd,
                level: call.level,
                optname: call.optname,
                optval: call.optval,
            },
            Some(syscall_request::Syscall::GetSockOpt(call)) => Syscall::GetSockOpt {
                sockfd: call.sockfd,
                level: call.level,
                optname: call.optname,
            },
            // File Descriptors
            Some(syscall_request::Syscall::Open(call)) => Syscall::Open {
                path: PathBuf::from(call.path),
                flags: call.flags,
                mode: call.mode,
            },
            Some(syscall_request::Syscall::Close(call)) => Syscall::Close { fd: call.fd },
            Some(syscall_request::Syscall::Dup(call)) => Syscall::Dup { fd: call.fd },
            Some(syscall_request::Syscall::Dup2(call)) => Syscall::Dup2 {
                oldfd: call.oldfd,
                newfd: call.newfd,
            },
            Some(syscall_request::Syscall::Lseek(call)) => Syscall::Lseek {
                fd: call.fd,
                offset: call.offset,
                whence: call.whence,
            },
            Some(syscall_request::Syscall::Fcntl(call)) => Syscall::Fcntl {
                fd: call.fd,
                cmd: call.cmd,
                arg: call.arg,
            },
            // IPC - Pipes
            Some(syscall_request::Syscall::CreatePipe(call)) => Syscall::CreatePipe {
                reader_pid: call.reader_pid,
                writer_pid: call.writer_pid,
                capacity: call.capacity.map(|c| c as usize),
            },
            Some(syscall_request::Syscall::WritePipe(call)) => Syscall::WritePipe {
                pipe_id: call.pipe_id,
                data: call.data,
            },
            Some(syscall_request::Syscall::ReadPipe(call)) => Syscall::ReadPipe {
                pipe_id: call.pipe_id,
                size: call.size as usize,
            },
            Some(syscall_request::Syscall::ClosePipe(call)) => Syscall::ClosePipe {
                pipe_id: call.pipe_id,
            },
            Some(syscall_request::Syscall::DestroyPipe(call)) => Syscall::DestroyPipe {
                pipe_id: call.pipe_id,
            },
            Some(syscall_request::Syscall::PipeStats(call)) => Syscall::PipeStats {
                pipe_id: call.pipe_id,
            },
            // IPC - Shared Memory
            Some(syscall_request::Syscall::CreateShm(call)) => Syscall::CreateShm {
                size: call.size as usize,
            },
            Some(syscall_request::Syscall::AttachShm(call)) => Syscall::AttachShm {
                segment_id: call.segment_id,
                read_only: call.read_only,
            },
            Some(syscall_request::Syscall::DetachShm(call)) => Syscall::DetachShm {
                segment_id: call.segment_id,
            },
            Some(syscall_request::Syscall::WriteShm(call)) => Syscall::WriteShm {
                segment_id: call.segment_id,
                offset: call.offset as usize,
                data: call.data,
            },
            Some(syscall_request::Syscall::ReadShm(call)) => Syscall::ReadShm {
                segment_id: call.segment_id,
                offset: call.offset as usize,
                size: call.size as usize,
            },
            Some(syscall_request::Syscall::DestroyShm(call)) => Syscall::DestroyShm {
                segment_id: call.segment_id,
            },
            Some(syscall_request::Syscall::ShmStats(call)) => Syscall::ShmStats {
                segment_id: call.segment_id,
            },
            // IPC - Async Queues
            Some(syscall_request::Syscall::CreateQueue(call)) => Syscall::CreateQueue {
                queue_type: call.queue_type,
                capacity: call.capacity.map(|c| c as usize),
            },
            Some(syscall_request::Syscall::SendQueue(call)) => Syscall::SendQueue {
                queue_id: call.queue_id,
                data: call.data,
                priority: call.priority.map(|p| p as u8),
            },
            Some(syscall_request::Syscall::ReceiveQueue(call)) => Syscall::ReceiveQueue {
                queue_id: call.queue_id,
            },
            Some(syscall_request::Syscall::SubscribeQueue(call)) => Syscall::SubscribeQueue {
                queue_id: call.queue_id,
            },
            Some(syscall_request::Syscall::UnsubscribeQueue(call)) => Syscall::UnsubscribeQueue {
                queue_id: call.queue_id,
            },
            Some(syscall_request::Syscall::CloseQueue(call)) => Syscall::CloseQueue {
                queue_id: call.queue_id,
            },
            Some(syscall_request::Syscall::DestroyQueue(call)) => Syscall::DestroyQueue {
                queue_id: call.queue_id,
            },
            Some(syscall_request::Syscall::QueueStats(call)) => Syscall::QueueStats {
                queue_id: call.queue_id,
            },
            // Scheduler operations
            Some(syscall_request::Syscall::ScheduleNext(_)) => Syscall::ScheduleNext,
            Some(syscall_request::Syscall::YieldProcess(_)) => Syscall::YieldProcess,
            Some(syscall_request::Syscall::GetCurrentScheduled(_)) => Syscall::GetCurrentScheduled,
            Some(syscall_request::Syscall::GetSchedulerStats(_)) => Syscall::GetSchedulerStats,
            None => {
                return Err(Status::invalid_argument("No syscall provided"));
            }
        };

        // Execute syscall
        let result = self.syscall_executor.execute(pid, syscall);

        // Convert result to proto
        let response = match result {
            SyscallResult::Success { data } => SyscallResponse {
                result: Some(syscall_response::Result::Success(SuccessResult {
                    data: data.unwrap_or_default(),
                })),
            },
            SyscallResult::Error { message } => SyscallResponse {
                result: Some(syscall_response::Result::Error(ErrorResult { message })),
            },
            SyscallResult::PermissionDenied { reason } => SyscallResponse {
                result: Some(syscall_response::Result::PermissionDenied(
                    PermissionDeniedResult { reason },
                )),
            },
        };

        Ok(Response::new(response))
    }

    async fn create_process(
        &self,
        request: Request<CreateProcessRequest>,
    ) -> Result<Response<CreateProcessResponse>, Status> {
        let req = request.into_inner();

        info!("gRPC: Creating process: {}", req.name);

        // Build execution config if command provided
        let exec_config = if let Some(command) = req.command {
            if !command.is_empty() {
                let mut config = crate::process::ExecutionConfig::new(command);
                if !req.args.is_empty() {
                    config = config.with_args(req.args);
                }
                if !req.env_vars.is_empty() {
                    let env: Vec<(String, String)> = req
                        .env_vars
                        .iter()
                        .filter_map(|e| {
                            let parts: Vec<&str> = e.splitn(2, '=').collect();
                            if parts.len() == 2 {
                                Some((parts[0].to_string(), parts[1].to_string()))
                            } else {
                                None
                            }
                        })
                        .collect();
                    config = config.with_env(env);
                }
                Some(config)
            } else {
                None
            }
        } else {
            None
        };

        // Create process (with or without OS execution)
        info!("About to call create_process_with_command");
        let pid = self.process_manager.create_process_with_command(
            req.name.clone(),
            req.priority as u8,
            exec_config,
        );
        info!("Created process, PID: {}", pid);

        // Get OS PID if available
        let os_pid = self.process_manager.get_process(pid).and_then(|p| p.os_pid);
        info!("Got OS PID: {:?}", os_pid);

        // Create sandbox based on level
        let sandbox_config = match SandboxLevel::try_from(req.sandbox_level) {
            Ok(SandboxLevel::Minimal) => SandboxConfig::minimal(pid),
            Ok(SandboxLevel::Privileged) => SandboxConfig::privileged(pid),
            _ => SandboxConfig::standard(pid),
        };

        self.sandbox_manager.create_sandbox(sandbox_config);

        let response = CreateProcessResponse {
            pid,
            success: true,
            error: String::new(),
            os_pid,
        };

        info!(
            "gRPC: Created process {} (PID: {}, OS PID: {:?})",
            req.name, pid, os_pid
        );

        Ok(Response::new(response))
    }

    async fn update_sandbox(
        &self,
        request: Request<UpdateSandboxRequest>,
    ) -> Result<Response<UpdateSandboxResponse>, Status> {
        let req = request.into_inner();

        info!("gRPC: Updating sandbox for PID {}", req.pid);

        // Get existing sandbox or create new one
        let mut config = self
            .sandbox_manager
            .get_sandbox(req.pid)
            .unwrap_or_else(|| SandboxConfig::standard(req.pid));

        // Update capabilities
        config.capabilities.clear();
        for cap in req.capabilities {
            if let Ok(capability) = Capability::try_from(cap) {
                let sandbox_cap = proto_to_sandbox_capability(capability);
                config.capabilities.insert(sandbox_cap);
            }
        }

        // Update paths
        config.allowed_paths = req.allowed_paths.into_iter().map(PathBuf::from).collect();
        config.blocked_paths = req.blocked_paths.into_iter().map(PathBuf::from).collect();

        // Update limits
        if let Some(limits) = req.limits {
            config.resource_limits.max_memory_bytes = limits.max_memory_bytes as usize;
            config.resource_limits.max_cpu_time_ms = limits.max_cpu_time_ms;
            config.resource_limits.max_file_descriptors = limits.max_file_descriptors;
            config.resource_limits.max_processes = limits.max_processes;
            config.resource_limits.max_network_connections = limits.max_network_connections;
        }

        // Update sandbox
        let success = self.sandbox_manager.update_sandbox(req.pid, config);

        let response = UpdateSandboxResponse {
            success,
            error: if success {
                String::new()
            } else {
                "Failed to update sandbox".to_string()
            },
        };

        Ok(Response::new(response))
    }

    async fn stream_events(
        &self,
        request: Request<EventStreamRequest>,
    ) -> Result<Response<Self::StreamEventsStream>, Status> {
        let req = request.into_inner();
        let event_types = req.event_types;

        info!("gRPC: Event streaming requested for types: {:?}", event_types);

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let process_manager = self.process_manager.clone();
        let sandbox_manager = self.sandbox_manager.clone();

        // Spawn background task to emit events
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

            loop {
                interval.tick().await;

                // Generate system events based on current state
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                // Emit process list events if requested
                if event_types.is_empty() || event_types.contains(&"process_created".to_string()) {
                    // Get current processes
                    let processes = process_manager.list_processes();
                    for proc in processes.iter().take(5) { // Limit to 5 for demo
                        let event = KernelEvent {
                            timestamp,
                            event: Some(kernel_event::Event::ProcessCreated(
                                ProcessCreatedEvent {
                                    pid: proc.pid,
                                    name: proc.name.clone(),
                                },
                            )),
                        };

                        if tx.send(Ok(event)).await.is_err() {
                            info!("Event stream closed by client");
                            return;
                        }
                    }
                }

                // Emit syscall execution events (based on scheduler activity)
                if event_types.is_empty() || event_types.contains(&"syscall_executed".to_string()) {
                    if let Some(stats) = process_manager.get_scheduler_stats() {
                        let event = KernelEvent {
                            timestamp,
                            event: Some(kernel_event::Event::SyscallExecuted(
                                SyscallExecutedEvent {
                                    pid: 0,
                                    syscall_type: "schedule_next".to_string(),
                                    success: true,
                                },
                            )),
                        };

                        if tx.send(Ok(event)).await.is_err() {
                            info!("Event stream closed by client");
                            return;
                        }
                    }
                }

                // Emit permission denied events (based on sandbox stats)
                if event_types.is_empty() || event_types.contains(&"permission_denied".to_string()) {
                    use crate::security::traits::SandboxProvider;
                    let sandbox_stats = sandbox_manager.stats();
                    if sandbox_stats.permission_denials > 0 {
                        let event = KernelEvent {
                            timestamp,
                            event: Some(kernel_event::Event::PermissionDenied(
                                PermissionDeniedEvent {
                                    pid: 0,
                                    syscall_type: "unknown".to_string(),
                                    reason: "Capability check failed".to_string(),
                                },
                            )),
                        };

                        if tx.send(Ok(event)).await.is_err() {
                            info!("Event stream closed by client");
                            return;
                        }
                    }
                }

                // Small delay to prevent flooding
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    async fn schedule_next(
        &self,
        _request: Request<ScheduleNextRequest>,
    ) -> Result<Response<ScheduleNextResponse>, Status> {
        info!("gRPC: Schedule next requested");

        match self.process_manager.schedule_next() {
            Some(pid) => Ok(Response::new(ScheduleNextResponse {
                success: true,
                next_pid: Some(pid),
                error: String::new(),
            })),
            None => Ok(Response::new(ScheduleNextResponse {
                success: true,
                next_pid: None,
                error: String::new(),
            })),
        }
    }

    async fn get_scheduler_stats(
        &self,
        _request: Request<GetSchedulerStatsRequest>,
    ) -> Result<Response<GetSchedulerStatsResponse>, Status> {
        info!("gRPC: Scheduler stats requested");

        if let Some(stats) = self.process_manager.get_scheduler_stats() {
            let policy_str = match stats.policy {
                crate::process::SchedulingPolicy::RoundRobin => "RoundRobin",
                crate::process::SchedulingPolicy::Priority => "Priority",
                crate::process::SchedulingPolicy::Fair => "Fair",
            };

            Ok(Response::new(GetSchedulerStatsResponse {
                success: true,
                stats: Some(SchedulerStats {
                    total_scheduled: stats.total_scheduled,
                    context_switches: stats.context_switches,
                    preemptions: stats.preemptions,
                    active_processes: stats.active_processes as u32,
                    policy: policy_str.to_string(),
                    quantum_micros: stats.quantum_micros,
                }),
                error: String::new(),
            }))
        } else {
            Ok(Response::new(GetSchedulerStatsResponse {
                success: false,
                stats: None,
                error: "Scheduler not available".to_string(),
            }))
        }
    }

    async fn set_scheduling_policy(
        &self,
        request: Request<SetSchedulingPolicyRequest>,
    ) -> Result<Response<SetSchedulingPolicyResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC: Set scheduling policy to: {}", req.policy);

        // Convert string policy to enum
        let policy = match req.policy.to_lowercase().as_str() {
            "roundrobin" | "round_robin" => crate::process::SchedulingPolicy::RoundRobin,
            "priority" => crate::process::SchedulingPolicy::Priority,
            "fair" => crate::process::SchedulingPolicy::Fair,
            _ => {
                return Ok(Response::new(SetSchedulingPolicyResponse {
                    success: false,
                    error: format!(
                        "Unknown policy: {}. Use RoundRobin, Priority, or Fair",
                        req.policy
                    ),
                }));
            }
        };

        // Set the policy dynamically
        if self.process_manager.set_scheduling_policy(policy) {
            info!("Successfully changed scheduling policy to {:?}", policy);
            Ok(Response::new(SetSchedulingPolicyResponse {
                success: true,
                error: String::new(),
            }))
        } else {
            Ok(Response::new(SetSchedulingPolicyResponse {
                success: false,
                error: "Scheduler not available".to_string(),
            }))
        }
    }

    async fn stream_syscall(
        &self,
        request: Request<tonic::Streaming<StreamSyscallRequest>>,
    ) -> Result<Response<Self::StreamSyscallStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let streaming_manager = self.streaming_manager.clone();

        tokio::spawn(async move {
            while let Ok(Some(req)) = stream.message().await {
                match req.request {
                    Some(stream_syscall_request::Request::Read(read_req)) => {
                        let path = std::path::PathBuf::from(read_req.path);
                        let chunk_size = if read_req.chunk_size > 0 {
                            Some(read_req.chunk_size)
                        } else {
                            None
                        };

                        match streaming_manager.stream_file_read(req.pid, path, chunk_size).await {
                            Ok(file_stream) => {
                                use futures::StreamExt;
                                tokio::pin!(file_stream);
                                while let Some(result) = file_stream.next().await {
                                    let chunk = match result {
                                        Ok(data) => StreamSyscallChunk {
                                            chunk: Some(stream_syscall_chunk::Chunk::Data(data)),
                                        },
                                        Err(e) => StreamSyscallChunk {
                                            chunk: Some(stream_syscall_chunk::Chunk::Error(e)),
                                        },
                                    };
                                    if tx.send(Ok(chunk)).await.is_err() {
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Ok(StreamSyscallChunk {
                                    chunk: Some(stream_syscall_chunk::Chunk::Error(e)),
                                })).await;
                                return;
                            }
                        }
                    }
                    Some(stream_syscall_request::Request::Write(_)) => {
                        // Write streaming handled separately
                    }
                    None => {}
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn execute_syscall_async(
        &self,
        request: Request<SyscallRequest>,
    ) -> Result<Response<AsyncSyscallResponse>, Status> {
        let req = request.into_inner();
        let pid = req.pid;

        // Convert proto syscall to internal (reuse existing logic)
        let syscall = match self.proto_to_syscall(&req) {
            Ok(s) => s,
            Err(e) => {
                return Ok(Response::new(AsyncSyscallResponse {
                    task_id: String::new(),
                    accepted: false,
                    error: e,
                }));
            }
        };

        let task_id = self.async_manager.submit(pid, syscall);

        Ok(Response::new(AsyncSyscallResponse {
            task_id,
            accepted: true,
            error: String::new(),
        }))
    }

    async fn get_async_status(
        &self,
        request: Request<AsyncStatusRequest>,
    ) -> Result<Response<AsyncStatusResponse>, Status> {
        let req = request.into_inner();

        match self.async_manager.get_status(&req.task_id) {
            Some((status, progress)) => {
                let (proto_status, result) = match status {
                    TaskStatus::Pending => (async_status_response::Status::Pending, None),
                    TaskStatus::Running => (async_status_response::Status::Running, None),
                    TaskStatus::Completed(res) => {
                        (async_status_response::Status::Completed, Some(self.syscall_result_to_proto(res)))
                    }
                    TaskStatus::Failed(msg) => (
                        async_status_response::Status::Failed,
                        Some(SyscallResponse {
                            result: Some(syscall_response::Result::Error(ErrorResult { message: msg })),
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

    async fn cancel_async(
        &self,
        request: Request<AsyncCancelRequest>,
    ) -> Result<Response<AsyncCancelResponse>, Status> {
        let req = request.into_inner();
        let cancelled = self.async_manager.cancel(&req.task_id);

        Ok(Response::new(AsyncCancelResponse {
            cancelled,
            error: if cancelled { String::new() } else { "Task not found or already completed".to_string() },
        }))
    }

    async fn execute_syscall_batch(
        &self,
        request: Request<BatchSyscallRequest>,
    ) -> Result<Response<BatchSyscallResponse>, Status> {
        let req = request.into_inner();
        let parallel = req.parallel;

        let mut syscalls = Vec::new();
        for syscall_req in req.requests {
            let pid = syscall_req.pid;
            match self.proto_to_syscall(&syscall_req) {
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

        let results = self.batch_executor.execute_batch(syscalls, parallel).await;

        let mut success_count = 0;
        let mut failure_count = 0;
        let responses: Vec<_> = results.into_iter().map(|r| {
            match &r {
                SyscallResult::Success { .. } => success_count += 1,
                _ => failure_count += 1,
            }
            self.syscall_result_to_proto(r)
        }).collect();

        Ok(Response::new(BatchSyscallResponse {
            responses,
            success_count,
            failure_count,
        }))
    }
}

impl KernelServiceImpl {
    fn proto_to_syscall(&self, req: &SyscallRequest) -> Result<Syscall, String> {
        // Extract syscall from protobuf (reuse existing conversion logic from execute_syscall)
        // This is a helper to avoid duplication
        match &req.syscall {
            Some(syscall_request::Syscall::ReadFile(call)) => Ok(Syscall::ReadFile {
                path: PathBuf::from(call.path.clone()),
            }),
            Some(syscall_request::Syscall::WriteFile(call)) => Ok(Syscall::WriteFile {
                path: PathBuf::from(call.path.clone()),
                data: call.data.clone(),
            }),
            // Add more syscalls as needed - for now support file ops
            Some(syscall_request::Syscall::CreateFile(call)) => Ok(Syscall::CreateFile {
                path: PathBuf::from(call.path.clone()),
            }),
            Some(syscall_request::Syscall::DeleteFile(call)) => Ok(Syscall::DeleteFile {
                path: PathBuf::from(call.path.clone()),
            }),
            Some(syscall_request::Syscall::SpawnProcess(call)) => Ok(Syscall::SpawnProcess {
                command: call.command.clone(),
                args: call.args.clone(),
            }),
            Some(syscall_request::Syscall::Sleep(call)) => Ok(Syscall::Sleep {
                duration_ms: call.duration_ms,
            }),
            _ => Err("Unsupported syscall for async/batch".to_string()),
        }
    }

    fn syscall_result_to_proto(&self, result: SyscallResult) -> SyscallResponse {
        match result {
            SyscallResult::Success { data } => SyscallResponse {
                result: Some(syscall_response::Result::Success(SuccessResult {
                    data: data.unwrap_or_default(),
                })),
            },
            SyscallResult::Error { message } => SyscallResponse {
                result: Some(syscall_response::Result::Error(ErrorResult { message })),
            },
            SyscallResult::PermissionDenied { reason } => SyscallResponse {
                result: Some(syscall_response::Result::PermissionDenied(
                    PermissionDeniedResult { reason },
                )),
            },
        }
    }
}

// Helper function to convert proto capability to sandbox capability
fn proto_to_sandbox_capability(cap: Capability) -> SandboxCapability {
    match cap {
        Capability::ReadFile => SandboxCapability::ReadFile(None),
        Capability::WriteFile => SandboxCapability::WriteFile(None),
        Capability::CreateFile => SandboxCapability::CreateFile(None),
        Capability::DeleteFile => SandboxCapability::DeleteFile(None),
        Capability::ListDirectory => SandboxCapability::ListDirectory(None),
        Capability::SpawnProcess => SandboxCapability::SpawnProcess,
        Capability::KillProcess => SandboxCapability::KillProcess,
        Capability::NetworkAccess => SandboxCapability::NetworkAccess(NetworkRule::AllowAll),
        Capability::BindPort => SandboxCapability::BindPort(None),
        Capability::SystemInfo => SandboxCapability::SystemInfo,
        Capability::TimeAccess => SandboxCapability::TimeAccess,
        Capability::SendMessage => SandboxCapability::SendMessage,
        Capability::ReceiveMessage => SandboxCapability::ReceiveMessage,
    }
}

/// gRPC Server wrapper that implements ServerLifecycle trait
pub struct GrpcServer {
    config: ServerConfig,
    syscall_executor: SyscallExecutor,
    process_manager: ProcessManager,
    sandbox_manager: SandboxManager,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl GrpcServer {
    pub fn new(
        config: ServerConfig,
        syscall_executor: SyscallExecutor,
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
    syscall_executor: SyscallExecutor,
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
