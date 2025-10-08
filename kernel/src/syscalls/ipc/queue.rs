/*!
 * Queue Syscall Operations
 * Handle message queue creation, send, receive, and pub/sub
 */

use crate::core::serialization::bincode;
use crate::core::serialization::json;
use crate::core::types::Pid;
use crate::monitoring::span_operation;
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};
use crate::security::Capability;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::types::SyscallResult;
use log::{error, info};

impl SyscallExecutorWithIpc {
    pub(in crate::syscalls) fn create_queue(
        &self,
        pid: Pid,
        queue_type: &str,
        capacity: Option<usize>,
    ) -> SyscallResult {
        let span = span_operation("queue_create");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("queue_type", queue_type);

        let request =
            PermissionRequest::new(pid, Resource::IpcChannel { channel_id: 0 }, Action::Create);
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Queue manager is legitimately optional (feature flag)
        let queue_manager = match &self.ipc().queue_manager() {
            Some(qm) => qm,
            None => {
                span.record_error("Queue manager not available");
                return SyscallResult::error("Queue manager not available");
            }
        };

        let q_type = match queue_type {
            "fifo" => crate::ipc::QueueType::Fifo,
            "priority" => crate::ipc::QueueType::Priority,
            "pubsub" => crate::ipc::QueueType::PubSub,
            _ => {
                return SyscallResult::error("Invalid queue type (must be: fifo, priority, pubsub)")
            }
        };

        match queue_manager.create(pid, q_type, capacity) {
            Ok(queue_id) => {
                info!("PID {} created {:?} queue {}", pid, q_type, queue_id);
                span.record("queue_id", &format!("{}", queue_id));
                span.record_result(true);
                match json::to_vec(&queue_id) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        error!("Failed to serialize queue ID: {}", e);
                        span.record_error("Serialization failed");
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(e) => {
                error!("Failed to create queue: {}", e);
                span.record_error(&format!("Queue creation failed: {}", e));
                SyscallResult::error(format!("Queue creation failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn send_queue(
        &self,
        pid: Pid,
        queue_id: u32,
        data: &[u8],
        priority: Option<u8>,
    ) -> SyscallResult {
        let span = span_operation("queue_send");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("queue_id", &format!("{}", queue_id));
        span.record("data_len", &format!("{}", data.len()));
        if let Some(p) = priority {
            span.record("priority", &format!("{}", p));
        }

        let request = PermissionRequest::new(
            pid,
            Resource::IpcChannel {
                channel_id: queue_id,
            },
            Action::Send,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        let queue_manager = match &self.ipc().queue_manager() {
            Some(qm) => qm,
            None => {
                span.record_error("Queue manager not available");
                return SyscallResult::error("Queue manager not available");
            }
        };

        match queue_manager.send(queue_id, pid, data.to_vec(), priority) {
            Ok(_) => {
                info!(
                    "PID {} sent {} bytes to queue {}",
                    pid,
                    data.len(),
                    queue_id
                );
                span.record_result(true);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Queue send failed: {}", e);
                span.record_error(&format!("Send failed: {}", e));
                SyscallResult::error(format!("Send failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn receive_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let span = span_operation("queue_receive");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("queue_id", &format!("{}", queue_id));

        let request = PermissionRequest::new(
            pid,
            Resource::IpcChannel {
                channel_id: queue_id,
            },
            Action::Receive,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        let queue_manager = match &self.ipc().queue_manager() {
            Some(qm) => qm,
            None => {
                span.record_error("Queue manager not available");
                return SyscallResult::error("Queue manager not available");
            }
        };

        // Use generic timeout executor for blocking receive
        use crate::ipc::types::IpcError;

        #[derive(Debug)]
        enum ReceiveError {
            NoMessage,
            Ipc(IpcError),
        }

        let result = self.timeout_executor().execute_with_retry(
            || match queue_manager.receive(queue_id, pid) {
                Ok(Some(msg)) => Ok(msg),
                Ok(None) => Err(ReceiveError::NoMessage),
                Err(e) => Err(ReceiveError::Ipc(e)),
            },
            |e| matches!(e, ReceiveError::NoMessage),
            self.timeout_config().queue_receive,
            "queue_receive",
        );

        match result {
            Ok(msg) => {
                // Read message data from MemoryManager
                match queue_manager.read_message_data(&msg) {
                    Ok(data) => {
                        info!(
                            "PID {} received {} bytes from queue {}",
                            pid,
                            data.len(),
                            queue_id
                        );
                        span.record("bytes_received", &format!("{}", data.len()));
                        span.record_result(true);

                        #[derive(serde::Serialize)]
                        struct MessageResponse {
                            id: u64,
                            from: u32,
                            data: Vec<u8>,
                            priority: u8,
                        }

                        let response = MessageResponse {
                            id: msg.id,
                            from: msg.from,
                            data,
                            priority: msg.priority,
                        };

                        match bincode::serialize_ipc_message(&response) {
                            Ok(serialized) => SyscallResult::success_with_data(serialized),
                            Err(e) => {
                                error!("Failed to serialize message: {}", e);
                                SyscallResult::error("Serialization failed")
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read message data: {}", e);
                        span.record_error(&format!("Read failed: {}", e));
                        SyscallResult::error(format!("Read failed: {}", e))
                    }
                }
            }
            Err(super::super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!(
                    "Queue receive timed out for PID {}, queue {} after {}ms",
                    pid, queue_id, elapsed_ms
                );
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error("Queue receive timed out")
            }
            Err(super::super::TimeoutError::Operation(ReceiveError::Ipc(e))) => {
                error!("Queue receive failed: {}", e);
                span.record_error(&format!("Receive failed: {}", e));
                SyscallResult::error(format!("Receive failed: {}", e))
            }
            Err(super::super::TimeoutError::Operation(ReceiveError::NoMessage)) => {
                // Should not reach here (filtered by is_would_block), but handle gracefully
                span.record_result(true);
                SyscallResult::success()
            }
        }
    }

    pub(in crate::syscalls) fn subscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        if !self
            .sandbox_manager()
            .check_permission(pid, &Capability::ReceiveMessage)
        {
            return SyscallResult::permission_denied("Missing ReceiveMessage capability");
        }

        let queue_manager = match &self.ipc().queue_manager() {
            Some(qm) => qm,
            None => return SyscallResult::error("Queue manager not available"),
        };

        match queue_manager.subscribe(queue_id, pid) {
            Ok(_) => {
                info!("PID {} subscribed to queue {}", pid, queue_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Queue subscribe failed: {}", e);
                SyscallResult::error(format!("Subscribe failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn unsubscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let queue_manager = match &self.ipc().queue_manager() {
            Some(qm) => qm,
            None => return SyscallResult::error("Queue manager not available"),
        };

        match queue_manager.unsubscribe(queue_id, pid) {
            Ok(_) => {
                info!("PID {} unsubscribed from queue {}", pid, queue_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Queue unsubscribe failed: {}", e);
                SyscallResult::error(format!("Unsubscribe failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn close_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let queue_manager = match &self.ipc().queue_manager() {
            Some(qm) => qm,
            None => return SyscallResult::error("Queue manager not available"),
        };

        match queue_manager.close(queue_id, pid) {
            Ok(_) => {
                info!("PID {} closed queue {}", pid, queue_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Queue close failed: {}", e);
                SyscallResult::error(format!("Close failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn destroy_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let queue_manager = match &self.ipc().queue_manager() {
            Some(qm) => qm,
            None => return SyscallResult::error("Queue manager not available"),
        };

        match queue_manager.destroy(queue_id, pid) {
            Ok(_) => {
                info!("PID {} destroyed queue {}", pid, queue_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Queue destroy failed: {}", e);
                SyscallResult::error(format!("Destroy failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn queue_stats(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let queue_manager = match &self.ipc().queue_manager() {
            Some(qm) => qm,
            None => return SyscallResult::error("Queue manager not available"),
        };

        match queue_manager.stats(queue_id) {
            Ok(stats) => match bincode::to_vec(&stats) {
                Ok(data) => {
                    info!("PID {} retrieved stats for queue {}", pid, queue_id);
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize queue stats: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            Err(e) => {
                error!("Queue stats failed: {}", e);
                SyscallResult::error(format!("Stats failed: {}", e))
            }
        }
    }
}
