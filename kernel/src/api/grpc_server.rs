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
use crate::security::{Capability as SandboxCapability, SandboxConfig, SandboxManager};
use crate::syscalls::{Syscall, SyscallExecutor, SyscallResult};

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
}

impl KernelServiceImpl {
    pub fn new(
        syscall_executor: SyscallExecutor,
        process_manager: ProcessManager,
        sandbox_manager: SandboxManager,
    ) -> Self {
        info!("gRPC service initialized");
        Self {
            syscall_executor,
            process_manager,
            sandbox_manager,
        }
    }
}

#[tonic::async_trait]
impl KernelService for KernelServiceImpl {
    type StreamEventsStream = tokio_stream::wrappers::ReceiverStream<Result<KernelEvent, Status>>;

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
        _request: Request<EventStreamRequest>,
    ) -> Result<Response<Self::StreamEventsStream>, Status> {
        // TODO: Implement event streaming
        // For now, return an empty stream
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
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
}

// Helper function to convert proto capability to sandbox capability
fn proto_to_sandbox_capability(cap: Capability) -> SandboxCapability {
    match cap {
        Capability::ReadFile => SandboxCapability::ReadFile,
        Capability::WriteFile => SandboxCapability::WriteFile,
        Capability::CreateFile => SandboxCapability::CreateFile,
        Capability::DeleteFile => SandboxCapability::DeleteFile,
        Capability::ListDirectory => SandboxCapability::ListDirectory,
        Capability::SpawnProcess => SandboxCapability::SpawnProcess,
        Capability::KillProcess => SandboxCapability::KillProcess,
        Capability::NetworkAccess => SandboxCapability::NetworkAccess,
        Capability::BindPort => SandboxCapability::BindPort,
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
