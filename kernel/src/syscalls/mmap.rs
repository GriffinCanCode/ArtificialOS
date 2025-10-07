/*!
 * Memory-Mapped File Syscalls
 * File-backed shared memory operations
 */

use crate::core::json;
use crate::core::types::Pid;
use crate::ipc::{MapFlags, ProtFlags};
use crate::permissions::{PermissionChecker, PermissionRequest};
use crate::security::Capability;
use log::{error, info};
use std::path::PathBuf;

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

impl SyscallExecutor {
    pub(super) fn mmap(
        &self,
        pid: Pid,
        path: &str,
        offset: usize,
        length: usize,
        prot: u8,
        shared: bool,
    ) -> SyscallResult {
        // Check file read permission using centralized manager
        let path_buf = PathBuf::from(path);
        let request = PermissionRequest::file_read(pid, path_buf.clone());
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let mmap_manager = match &self.mmap_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Mmap manager not available"),
        };

        // Parse protection flags
        let prot_flags = ProtFlags {
            read: (prot & 0x01) != 0,
            write: (prot & 0x02) != 0,
            exec: (prot & 0x04) != 0,
        };

        // Check write permission if write requested
        if prot_flags.write {
            let write_request = PermissionRequest::file_write(pid, path_buf);
            let write_response = self.permission_manager.check_and_audit(&write_request);

            if !write_response.is_allowed() {
                return SyscallResult::permission_denied(write_response.reason());
            }
        }

        let map_flags = if shared {
            MapFlags::Shared
        } else {
            MapFlags::Private
        };

        match mmap_manager.mmap(pid, path.to_string(), offset, length, prot_flags, map_flags) {
            Ok(mmap_id) => {
                info!(
                    "PID {} created mmap {} for file '{}' ({} bytes)",
                    pid, mmap_id, path, length
                );
                match json::to_vec(&mmap_id) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        error!("Failed to serialize mmap ID: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(e) => {
                error!("Failed to create mmap for PID {}: {}", pid, e);
                SyscallResult::error(format!("Mmap failed: {}", e))
            }
        }
    }

    pub(super) fn mmap_read(
        &self,
        pid: Pid,
        mmap_id: u32,
        offset: usize,
        length: usize,
    ) -> SyscallResult {
        let mmap_manager = match &self.mmap_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Mmap manager not available"),
        };

        match mmap_manager.read(pid, mmap_id, offset, length) {
            Ok(data) => {
                info!("PID {} read {} bytes from mmap {}", pid, data.len(), mmap_id);
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Failed to read from mmap {} for PID {}: {}", mmap_id, pid, e);
                SyscallResult::error(format!("Mmap read failed: {}", e))
            }
        }
    }

    pub(super) fn mmap_write(
        &self,
        pid: Pid,
        mmap_id: u32,
        offset: usize,
        data: &[u8],
    ) -> SyscallResult {
        let mmap_manager = match &self.mmap_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Mmap manager not available"),
        };

        match mmap_manager.write(pid, mmap_id, offset, data) {
            Ok(_) => {
                info!("PID {} wrote {} bytes to mmap {}", pid, data.len(), mmap_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to write to mmap {} for PID {}: {}", mmap_id, pid, e);
                SyscallResult::error(format!("Mmap write failed: {}", e))
            }
        }
    }

    pub(super) fn msync(&self, pid: Pid, mmap_id: u32) -> SyscallResult {
        let mmap_manager = match &self.mmap_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Mmap manager not available"),
        };

        match mmap_manager.msync(pid, mmap_id) {
            Ok(_) => {
                info!("PID {} synced mmap {}", pid, mmap_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to sync mmap {} for PID {}: {}", mmap_id, pid, e);
                SyscallResult::error(format!("Msync failed: {}", e))
            }
        }
    }

    pub(super) fn munmap(&self, pid: Pid, mmap_id: u32) -> SyscallResult {
        let mmap_manager = match &self.mmap_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Mmap manager not available"),
        };

        match mmap_manager.munmap(pid, mmap_id) {
            Ok(_) => {
                info!("PID {} unmapped mmap {}", pid, mmap_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to unmap mmap {} for PID {}: {}", mmap_id, pid, e);
                SyscallResult::error(format!("Munmap failed: {}", e))
            }
        }
    }

    pub(super) fn mmap_stats(&self, pid: Pid, mmap_id: u32) -> SyscallResult {
        // Check permission using centralized manager
        use crate::permissions::{Resource, Action};
        let request = PermissionRequest::new(pid, Resource::System("mmap".to_string()), Action::Inspect);
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let mmap_manager = match &self.mmap_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Mmap manager not available"),
        };

        match mmap_manager.get_info(mmap_id) {
            Some(info) => match json::to_vec(&info) {
                Ok(data) => {
                    info!("PID {} retrieved stats for mmap {}", pid, mmap_id);
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize mmap stats: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            None => {
                error!("Mmap {} not found", mmap_id);
                SyscallResult::error(format!("Mmap {} not found", mmap_id))
            }
        }
    }
}
