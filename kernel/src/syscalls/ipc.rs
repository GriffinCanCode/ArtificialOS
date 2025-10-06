/*!

 * IPC Syscalls
 * Inter-process communication (pipes and shared memory)
 */

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
                match serde_json::to_vec(&pipe_id) {
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
                match serde_json::to_vec(&written) {
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
                info!("PID {} read {} bytes from pipe {}", pid, data.len(), pipe_id);
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
            Ok(stats) => match serde_json::to_vec(&stats) {
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
                info!("PID {} created shared memory segment {} ({} bytes)", pid, segment_id, size);
                match serde_json::to_vec(&segment_id) {
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
                info!("PID {} attached to segment {} (read_only: {})", pid, segment_id, read_only);
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

    pub(super) fn write_shm(&self, pid: Pid, segment_id: u32, offset: usize, data: &[u8]) -> SyscallResult {
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
                info!("PID {} wrote {} bytes to segment {} at offset {}", pid, data.len(), segment_id, offset);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Shared memory write failed: {}", e);
                SyscallResult::error(format!("Write failed: {}", e))
            }
        }
    }

    pub(super) fn read_shm(&self, pid: Pid, segment_id: u32, offset: usize, size: usize) -> SyscallResult {
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
                info!("PID {} read {} bytes from segment {} at offset {}", pid, data.len(), segment_id, offset);
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
            Ok(stats) => match serde_json::to_vec(&stats) {
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
}
