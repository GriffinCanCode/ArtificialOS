/*!
 * gRPC Server
 * Exposes kernel syscalls to AI service via gRPC
 */

use log::info;
use std::path::PathBuf;
use tonic::{transport::Server, Request, Response, Status};

use crate::process::ProcessManager;
use crate::sandbox::{Capability as SandboxCapability, SandboxConfig, SandboxManager};
use crate::syscall::{Syscall, SyscallExecutor, SyscallResult};

// Include generated protobuf code
pub mod kernel_proto {
    tonic::include_proto!("kernel");
}

use kernel_proto::kernel_service_server::{KernelService, KernelServiceServer};
use kernel_proto::*;

/// gRPC service implementation
pub struct KernelServiceImpl {
    syscall_executor: SyscallExecutor,
    process_manager: parking_lot::RwLock<ProcessManager>,
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
            process_manager: parking_lot::RwLock::new(process_manager),
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
            Some(syscall_request::Syscall::SpawnProcess(call)) => Syscall::SpawnProcess {
                command: call.command,
                args: call.args,
            },
            Some(syscall_request::Syscall::KillProcess(call)) => Syscall::KillProcess {
                target_pid: call.target_pid,
            },
            Some(syscall_request::Syscall::GetSystemInfo(_)) => Syscall::GetSystemInfo,
            Some(syscall_request::Syscall::GetCurrentTime(_)) => Syscall::GetCurrentTime,
            Some(syscall_request::Syscall::GetEnvVar(call)) => {
                Syscall::GetEnvironmentVar { key: call.key }
            }
            Some(syscall_request::Syscall::NetworkRequest(call)) => {
                Syscall::NetworkRequest { url: call.url }
            }
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

        // Create process
        let pm = self.process_manager.read();
        let pid = pm.create_process(req.name.clone(), req.priority as u8);

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
        };

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

/// Start the gRPC server
pub async fn start_grpc_server(
    addr: std::net::SocketAddr,
    syscall_executor: SyscallExecutor,
    process_manager: ProcessManager,
    sandbox_manager: SandboxManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = KernelServiceImpl::new(syscall_executor, process_manager, sandbox_manager);

    info!("🌐 gRPC server starting on {}", addr);

    Server::builder()
        .add_service(KernelServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
