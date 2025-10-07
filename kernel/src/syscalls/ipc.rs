/*!

* IPC Syscalls
* Inter-process communication (pipes and shared memory)
*/

use crate::core::json;
use crate::core::types::Pid;

use log::{error, info};

use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

impl SyscallExecutor {
    // Pipe operations
    pub(super) fn create_pipe(
        &self,
        pid: Pid,
        reader_pid: Pid,
        writer_pid: Pid,
        capacity: Option<usize>,
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SendMessage)
        {
            return SyscallResult::permission_denied("Missing SendMessage capability");
        }

        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.create(reader_pid, writer_pid, capacity) {
            Ok(pipe_id) => {
                info!("PID {} created pipe {}", pid, pipe_id);
                match json::to_vec(&pipe_id) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        error!("Failed to serialize pipe ID: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(e) => {
                error!("Failed to create pipe: {}", e);
                SyscallResult::error(format!("Pipe creation failed: {}", e))
            }
        }
    }

    pub(super) fn write_pipe(&self, pid: Pid, pipe_id: u32, data: &[u8]) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SendMessage)
        {
            return SyscallResult::permission_denied("Missing SendMessage capability");
        }

        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.write(pipe_id, pid, data) {
            Ok(written) => {
                info!("PID {} wrote {} bytes to pipe {}", pid, written, pipe_id);
                match json::to_vec(&written) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        error!("Failed to serialize write result: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(e) => {
                error!("Pipe write failed: {}", e);
                SyscallResult::error(format!("Pipe write failed: {}", e))
            }
        }
    }

    pub(super) fn read_pipe(&self, pid: Pid, pipe_id: u32, size: usize) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReceiveMessage)
        {
            return SyscallResult::permission_denied("Missing ReceiveMessage capability");
        }

        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.read(pipe_id, pid, size) {
            Ok(data) => {
                info!(
                    "PID {} read {} bytes from pipe {}",
                    pid,
                    data.len(),
                    pipe_id
                );
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Pipe read failed: {}", e);
                SyscallResult::error(format!("Pipe read failed: {}", e))
            }
        }
    }

    pub(super) fn close_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult {
        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.close(pipe_id, pid) {
            Ok(_) => {
                info!("PID {} closed pipe {}", pid, pipe_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Pipe close failed: {}", e);
                SyscallResult::error(format!("Pipe close failed: {}", e))
            }
        }
    }

    pub(super) fn destroy_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult {
        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.destroy(pipe_id) {
            Ok(_) => {
                info!("PID {} destroyed pipe {}", pid, pipe_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Pipe destroy failed: {}", e);
                SyscallResult::error(format!("Pipe destroy failed: {}", e))
            }
        }
    }

    pub(super) fn pipe_stats(&self, pid: Pid, pipe_id: u32) -> SyscallResult {
        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.stats(pipe_id) {
            Ok(stats) => match json::to_vec(&stats) {
                Ok(data) => {
                    info!("PID {} retrieved stats for pipe {}", pid, pipe_id);
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize pipe stats: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            Err(e) => {
                error!("Pipe stats failed: {}", e);
                SyscallResult::error(format!("Pipe stats failed: {}", e))
            }
        }
    }

    // Shared memory operations
    pub(super) fn create_shm(&self, pid: Pid, size: usize) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SendMessage)
        {
            return SyscallResult::permission_denied("Missing SendMessage capability");
        }

        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.create(size, pid) {
            Ok(segment_id) => {
                info!(
                    "PID {} created shared memory segment {} ({} bytes)",
                    pid, segment_id, size
                );
                match json::to_vec(&segment_id) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        error!("Failed to serialize segment ID: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(e) => {
                error!("Failed to create shared memory: {}", e);
                SyscallResult::error(format!("Shared memory creation failed: {}", e))
            }
        }
    }

    pub(super) fn attach_shm(&self, pid: Pid, segment_id: u32, read_only: bool) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReceiveMessage)
        {
            return SyscallResult::permission_denied("Missing ReceiveMessage capability");
        }

        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.attach(segment_id, pid, read_only) {
            Ok(_) => {
                info!(
                    "PID {} attached to segment {} (read_only: {})",
                    pid, segment_id, read_only
                );
                SyscallResult::success()
            }
            Err(e) => {
                error!("Shared memory attach failed: {}", e);
                SyscallResult::error(format!("Attach failed: {}", e))
            }
        }
    }

    pub(super) fn detach_shm(&self, pid: Pid, segment_id: u32) -> SyscallResult {
        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.detach(segment_id, pid) {
            Ok(_) => {
                info!("PID {} detached from segment {}", pid, segment_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Shared memory detach failed: {}", e);
                SyscallResult::error(format!("Detach failed: {}", e))
            }
        }
    }

    pub(super) fn write_shm(
        &self,
        pid: Pid,
        segment_id: u32,
        offset: usize,
        data: &[u8],
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SendMessage)
        {
            return SyscallResult::permission_denied("Missing SendMessage capability");
        }

        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.write(segment_id, pid, offset, data) {
            Ok(_) => {
                info!(
                    "PID {} wrote {} bytes to segment {} at offset {}",
                    pid,
                    data.len(),
                    segment_id,
                    offset
                );
                SyscallResult::success()
            }
            Err(e) => {
                error!("Shared memory write failed: {}", e);
                SyscallResult::error(format!("Write failed: {}", e))
            }
        }
    }

    pub(super) fn read_shm(
        &self,
        pid: Pid,
        segment_id: u32,
        offset: usize,
        size: usize,
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReceiveMessage)
        {
            return SyscallResult::permission_denied("Missing ReceiveMessage capability");
        }

        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.read(segment_id, pid, offset, size) {
            Ok(data) => {
                info!(
                    "PID {} read {} bytes from segment {} at offset {}",
                    pid,
                    data.len(),
                    segment_id,
                    offset
                );
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Shared memory read failed: {}", e);
                SyscallResult::error(format!("Read failed: {}", e))
            }
        }
    }

    pub(super) fn destroy_shm(&self, pid: Pid, segment_id: u32) -> SyscallResult {
        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.destroy(segment_id, pid) {
            Ok(_) => {
                info!("PID {} destroyed segment {}", pid, segment_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Shared memory destroy failed: {}", e);
                SyscallResult::error(format!("Destroy failed: {}", e))
            }
        }
    }

    pub(super) fn shm_stats(&self, pid: Pid, segment_id: u32) -> SyscallResult {
        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.stats(segment_id) {
            Ok(stats) => match json::to_vec(&stats) {
                Ok(data) => {
                    info!("PID {} retrieved stats for segment {}", pid, segment_id);
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize segment stats: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            Err(e) => {
                error!("Shared memory stats failed: {}", e);
                SyscallResult::error(format!("Stats failed: {}", e))
            }
        }
    }

    // Queue operations
    pub(super) fn create_queue(
        &self,
        pid: Pid,
        queue_type: &str,
        capacity: Option<usize>,
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SendMessage)
        {
            return SyscallResult::permission_denied("Missing SendMessage capability");
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

    pub(super) fn send_queue(
        &self,
        pid: Pid,
        queue_id: u32,
        data: &[u8],
        priority: Option<u8>,
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SendMessage)
        {
            return SyscallResult::permission_denied("Missing SendMessage capability");
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

    pub(super) fn receive_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
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

                        match json::serialize_ipc_message(&response) {
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

    pub(super) fn subscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
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

    pub(super) fn unsubscribe_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
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

    pub(super) fn close_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
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

    pub(super) fn destroy_queue(&self, pid: Pid, queue_id: u32) -> SyscallResult {
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

    pub(super) fn queue_stats(&self, pid: Pid, queue_id: u32) -> SyscallResult {
        let queue_manager = match &self.queue_manager {
            Some(qm) => qm,
            None => return SyscallResult::error("Queue manager not available"),
        };

        match queue_manager.stats(queue_id) {
            Ok(stats) => match json::to_vec(&stats) {
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
