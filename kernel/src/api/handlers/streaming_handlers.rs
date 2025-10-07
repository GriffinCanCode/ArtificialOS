/*!
 * Streaming-related gRPC handler implementations
 */

use tonic::{Request, Response, Status};
use tracing::info;
use crate::process::ProcessManagerImpl as ProcessManager;
use crate::security::SandboxManager;
use crate::api::grpc_server::kernel_proto::*;
use crate::api::streaming::StreamingManager;

pub async fn handle_stream_events(
    process_manager: &ProcessManager,
    sandbox_manager: &SandboxManager,
    request: Request<EventStreamRequest>,
) -> Result<Response<tokio_stream::wrappers::ReceiverStream<Result<KernelEvent, Status>>>, Status> {
    let req = request.into_inner();
    let event_types = req.event_types;

    info!(
        "gRPC: Event streaming requested for types: {:?}",
        event_types
    );

    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let process_manager = process_manager.clone();
    let sandbox_manager = sandbox_manager.clone();

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
                for proc in processes.iter().take(5) {
                    // Limit to 5 for demo
                    let event = KernelEvent {
                        timestamp,
                        event: Some(kernel_event::Event::ProcessCreated(ProcessCreatedEvent {
                            pid: proc.pid,
                            name: proc.name.clone(),
                        })),
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
            if event_types.is_empty() || event_types.contains(&"permission_denied".to_string())
            {
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

pub async fn handle_stream_syscall(
    streaming_manager: &StreamingManager,
    request: Request<tonic::Streaming<StreamSyscallRequest>>,
) -> Result<Response<tokio_stream::wrappers::ReceiverStream<Result<StreamSyscallChunk, Status>>>, Status> {
    let mut stream = request.into_inner();
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let streaming_manager = streaming_manager.clone();

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

                    match streaming_manager
                        .stream_file_read(req.pid, path, chunk_size)
                        .await
                    {
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
                            let _ = tx
                                .send(Ok(StreamSyscallChunk {
                                    chunk: Some(stream_syscall_chunk::Chunk::Error(e)),
                                }))
                                .await;
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

    Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
        rx,
    )))
}
