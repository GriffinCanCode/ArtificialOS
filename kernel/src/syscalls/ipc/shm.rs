/*!
 * Shared Memory Syscall Operations
 * Handle shared memory creation, attach, detach, and access
 */

use crate::core::{bincode, json};
use crate::core::types::Pid;
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};
use crate::syscalls::executor::SyscallExecutor;
use crate::syscalls::types::SyscallResult;
use log::{error, info};

impl SyscallExecutor {
    pub(crate) fn create_shm(&self, pid: Pid, size: usize) -> SyscallResult {
        let request = PermissionRequest::new(pid, Resource::IpcChannel { channel_id: 0 }, Action::Create);
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
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

    pub(crate) fn attach_shm(&self, pid: Pid, segment_id: u32, read_only: bool) -> SyscallResult {
        let request = PermissionRequest::new(pid, Resource::IpcChannel { channel_id: segment_id }, Action::Read);
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
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

    pub(crate) fn detach_shm(&self, pid: Pid, segment_id: u32) -> SyscallResult {
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

    pub(crate) fn write_shm(
        &self,
        pid: Pid,
        segment_id: u32,
        offset: usize,
        data: &[u8],
    ) -> SyscallResult {
        let request = PermissionRequest::new(pid, Resource::IpcChannel { channel_id: segment_id }, Action::Write);
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
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

    pub(crate) fn read_shm(
        &self,
        pid: Pid,
        segment_id: u32,
        offset: usize,
        size: usize,
    ) -> SyscallResult {
        let request = PermissionRequest::new(pid, Resource::IpcChannel { channel_id: segment_id }, Action::Read);
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
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

    pub(crate) fn destroy_shm(&self, pid: Pid, segment_id: u32) -> SyscallResult {
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

    pub(crate) fn shm_stats(&self, pid: Pid, segment_id: u32) -> SyscallResult {
        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.stats(segment_id) {
            Ok(stats) => match bincode::to_vec(&stats) {
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
