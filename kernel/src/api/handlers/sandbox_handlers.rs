/*!
 * Sandbox-related gRPC handler implementations
 */

use crate::api::conversions::response::proto_to_sandbox_capability;
use crate::api::server::grpc_server::kernel_proto::*;
use crate::security::traits::SandboxProvider;
use crate::security::{SandboxConfig, SandboxManager};
use std::path::PathBuf;
use tonic::{Request, Response, Status};
use tracing::info;

pub async fn handle_update_sandbox(
    sandbox_manager: &SandboxManager,
    request: Request<UpdateSandboxRequest>,
) -> Result<Response<UpdateSandboxResponse>, Status> {
    let req = request.into_inner();

    info!("gRPC: Updating sandbox for PID {}", req.pid);

    // Get existing sandbox or create new one
    let mut config = sandbox_manager
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
    let success = sandbox_manager.update_sandbox(req.pid, config);

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
