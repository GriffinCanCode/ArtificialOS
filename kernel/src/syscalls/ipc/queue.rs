/*!
 * Queue Syscall Operations
 * Handle message queue creation, send, receive, and pub/sub
 */

use crate::core::bincode;
use crate::core::json;
use crate::core::types::Pid;
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};
use crate::security::Capability;
use crate::syscalls::executor::SyscallExecutor;
use crate::syscalls::types::SyscallResult;
use log::{error, info};

impl SyscallExecutor {
    pub(crate) fn create_queue(
        &self,
        pid: Pid,
        queue_type: &str,
        capacity: Option<usize>,
    ) -> SyscallResult {
        let request =
            PermissionRequest::new(pid, Resource::IpcChannel { channel_id: 0 }, Action::Create);
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let queue_manager = match &self.queue_manager {
            Some(qm) => qm,
            None => return SyscallResult::error("Queue manager not available"),
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
                match json::to_vec(&queue_id) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        error!("Failed to serialize queue ID: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(e) => {
                error!("Failed to create queue: {}", e);
                SyscallResult::error(format!("Queue creation failed: {}", e))
            }
        }
    }

    pub(crate) fn send_queue(
        &self,
        pid: Pid,
        queue_id: u32,
        data: &[u8],
        priority: Option<u8>,
    ) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::IpcChannel {
                channel_id: queue_id,
            },
            Action::Send,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let queue_manager = match &self.queue_manager {
            Some(qm) => qm,
            None => return SyscallResult::error("Queue manager not available"),
        };

        match queue_manager.send(queue_id, pid, data.to_vec(), priority) {
            Ok(_) => {
                info!(
                    "PID {} sent {} bytes to queue {}",
                    pid,
                    data.len(),
                    queue_id
                );
                SyscallResult::success()
            }
            Err(e) => {
                error!("Queue send failed: {}", e);
                SyscallResult::error(format!("Send failed: {}", e))
            }
        }
    }

    pub(crate) fn receive_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::IpcChannel {
                channel_id: queue_id,
            },
            Action::Receive,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // Use timeout operations if enabled
        if self.timeout_config.enabled && self.timeout_queue_ops.is_some() {
            let timeout_ops = self.timeout_queue_ops.as_ref().unwrap();
            match timeout_ops.receive_timeout(queue_id, pid, self.timeout_config.queue_receive) {
                Ok(msg) => {
                    // Read message data
                    let queue_manager = self.queue_manager.as_ref().unwrap();
                    match queue_manager.read_message_data(&msg) {
                        Ok(data) => {
                            info!(
                                "PID {} received {} bytes from queue {} (with timeout)",
                                pid,
                                data.len(),
                                queue_id
                            );

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
                            SyscallResult::error(format!("Read failed: {}", e))
                        }
                    }
                }
                Err(e) => {
                    error!("Queue receive failed: {}", e);
                    SyscallResult::error(format!("Receive failed: {}", e))
                }
            }
        } else {
            // Fallback to non-blocking operation
            let queue_manager = match &self.queue_manager {
                Some(qm) => qm,
                None => return SyscallResult::error("Queue manager not available"),
            };

            match queue_manager.receive(queue_id, pid) {
            Ok(Some(msg)) => {
                // Read message data from MemoryManager
                match queue_manager.read_message_data(&msg) {
                    Ok(data) => {
                        info!(
                            "PID {} received {} bytes from queue {}",
                            pid,
                            data.len(),
                            queue_id
                        );

                        // Create a serializable message with the data
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
                        SyscallResult::error(format!("Read failed: {}", e))
                    }
                }
            }
            Ok(None) => {
                // No message available (non-blocking)
                SyscallResult::success()
            }
            Err(e) => {
                error!("Queue receive failed: {}", e);
                SyscallResult::error(format!("Receive failed: {}", e))
            }
        }
        }
    }

    pub(crate) fn subscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReceiveMessage)
        {
            return SyscallResult::permission_denied("Missing ReceiveMessage capability");
        }

        let queue_manager = match &self.queue_manager {
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

    pub(crate) fn unsubscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let queue_manager = match &self.queue_manager {
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

    pub(crate) fn close_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let queue_manager = match &self.queue_manager {
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

    pub(crate) fn destroy_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let queue_manager = match &self.queue_manager {
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

    pub(crate) fn queue_stats(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let queue_manager = match &self.queue_manager {
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
