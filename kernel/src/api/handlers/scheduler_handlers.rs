/*!
 * Scheduler-related gRPC handler implementations
 */

use tonic::{Request, Response, Status};
use tracing::info;
use crate::process::ProcessManagerImpl as ProcessManager;
use crate::api::grpc_server::kernel_proto::*;

pub async fn handle_schedule_next(
    process_manager: &ProcessManager,
    _request: Request<ScheduleNextRequest>,
) -> Result<Response<ScheduleNextResponse>, Status> {
    info!("gRPC: Schedule next requested");

    match process_manager.schedule_next() {
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

pub async fn handle_get_scheduler_stats(
    process_manager: &ProcessManager,
    _request: Request<GetSchedulerStatsRequest>,
) -> Result<Response<GetSchedulerStatsResponse>, Status> {
    info!("gRPC: Scheduler stats requested");

    if let Some(stats) = process_manager.get_scheduler_stats() {
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

pub async fn handle_set_scheduling_policy(
    process_manager: &ProcessManager,
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
    if process_manager.set_scheduling_policy(policy) {
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
