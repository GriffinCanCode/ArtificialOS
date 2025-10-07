/*!
 * Process-related gRPC handler implementations
 */

use tonic::{Request, Response, Status};
use tracing::{info, instrument};
use crate::monitoring::{span_grpc, GrpcSpan};
use crate::process::ProcessManagerImpl as ProcessManager;
use crate::security::{SandboxConfig, SandboxManager};
use crate::security::traits::SandboxProvider;
use crate::api::server::grpc_server::kernel_proto::*;

#[instrument(skip(process_manager, sandbox_manager, request), fields(process_name, priority, sandbox_level, trace_id))]
pub async fn handle_create_process(
    process_manager: &ProcessManager,
    sandbox_manager: &SandboxManager,
    request: Request<CreateProcessRequest>,
) -> Result<Response<CreateProcessResponse>, Status> {
    let span = span_grpc("create_process");
    let _guard = span.enter();

    let req = request.into_inner();

    info!(
        process_name = %req.name,
        priority = req.priority,
        sandbox_level = %req.sandbox_level,
        trace_id = %span.trace_id(),
        "gRPC: Creating process"
    );

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
    let pid = process_manager.create_process_with_command(
        req.name.clone(),
        req.priority as u8,
        exec_config,
    );
    info!("Created process, PID: {}", pid);

    // Get OS PID if available
    let os_pid = process_manager.get_process(pid).and_then(|p| p.os_pid);
    info!("Got OS PID: {:?}", os_pid);

    // Create sandbox based on level
    let sandbox_config = match SandboxLevel::try_from(req.sandbox_level) {
        Ok(SandboxLevel::Minimal) => SandboxConfig::minimal(pid),
        Ok(SandboxLevel::Privileged) => SandboxConfig::privileged(pid),
        _ => SandboxConfig::standard(pid),
    };

    sandbox_manager.create_sandbox(sandbox_config);

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
