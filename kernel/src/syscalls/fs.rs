/*!

* File System Syscalls
* File and directory operations
*/

use crate::core::types::Pid;

use log::{error, info, warn};
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

impl SyscallExecutor {
    pub(super) fn read_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        self.vfs_read(pid, path)
    }

    pub(super) fn write_file(&self, pid: Pid, path: &PathBuf, data: &[u8]) -> SyscallResult {
        self.vfs_write(pid, path, data)
    }

    pub(super) fn create_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        self.vfs_write(pid, path, &[])
    }

    pub(super) fn delete_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        self.vfs_delete(pid, path)
    }

    pub(super) fn list_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        self.vfs_list_dir(pid, path)
    }

    pub(super) fn file_exists(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        self.vfs_exists(pid, path)
    }

    pub(super) fn file_stat(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile)
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        if !self.sandbox_manager.check_path_access(pid, path) {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        match fs::metadata(path) {
            Ok(metadata) => {
                #[cfg(unix)]
                let mode = format!("{:o}", metadata.permissions().mode());
                #[cfg(not(unix))]
                let mode = String::from("0644");

                let file_info = serde_json::json!({
                    "name": path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    "path": path.to_str().unwrap_or(""),
                    "size": metadata.len(),
                    "is_dir": metadata.is_dir(),
                    "mode": mode,
                    "modified": metadata.modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                    "extension": path.extension().and_then(|e| e.to_str()).unwrap_or(""),
                });

                info!("PID {} stat file: {:?}", pid, path);
                match serde_json::to_vec(&file_info) {
                    Ok(json) => SyscallResult::success_with_data(json),
                    Err(e) => {
                        error!("Failed to serialize file stat: {}", e);
                        SyscallResult::error("Failed to serialize file stat")
                    }
                }
            }
            Err(e) => {
                error!("Failed to stat file {:?}: {}", path, e);
                SyscallResult::error(format!("Stat failed: {}", e))
            }
        }
    }

    pub(super) fn move_file(
        &self,
        pid: Pid,
        source: &PathBuf,
        destination: &PathBuf,
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::WriteFile)
        {
            return SyscallResult::permission_denied("Missing WriteFile capability");
        }

        if !self.sandbox_manager.check_path_access(pid, source) {
            return SyscallResult::permission_denied(format!(
                "Source path not accessible: {:?}",
                source
            ));
        }

        if !self.sandbox_manager.check_path_access(pid, destination) {
            return SyscallResult::permission_denied(format!(
                "Destination path not accessible: {:?}",
                destination
            ));
        }

        match fs::rename(source, destination) {
            Ok(_) => {
                info!("PID {} moved file: {:?} -> {:?}", pid, source, destination);
                SyscallResult::success()
            }
            Err(e) => {
                error!(
                    "Failed to move file {:?} -> {:?}: {}",
                    source, destination, e
                );
                SyscallResult::error(format!("Move failed: {}", e))
            }
        }
    }

    pub(super) fn copy_file(
        &self,
        pid: Pid,
        source: &PathBuf,
        destination: &PathBuf,
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile)
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::WriteFile)
        {
            return SyscallResult::permission_denied("Missing WriteFile capability");
        }

        if !self.sandbox_manager.check_path_access(pid, source) {
            return SyscallResult::permission_denied(format!(
                "Source path not accessible: {:?}",
                source
            ));
        }

        if !self.sandbox_manager.check_path_access(pid, destination) {
            return SyscallResult::permission_denied(format!(
                "Destination path not accessible: {:?}",
                destination
            ));
        }

        match fs::copy(source, destination) {
            Ok(bytes) => {
                info!(
                    "PID {} copied file: {:?} -> {:?} ({} bytes)",
                    pid, source, destination, bytes
                );
                SyscallResult::success()
            }
            Err(e) => {
                error!(
                    "Failed to copy file {:?} -> {:?}: {}",
                    source, destination, e
                );
                SyscallResult::error(format!("Copy failed: {}", e))
            }
        }
    }

    pub(super) fn create_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        self.vfs_create_dir(pid, path)
    }

    pub(super) fn remove_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        self.vfs_remove_dir(pid, path)
    }

    pub(super) fn get_working_directory(&self, pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        match std::env::current_dir() {
            Ok(path) => {
                info!("PID {} retrieved working directory: {:?}", pid, path);
                match path.to_str() {
                    Some(s) => SyscallResult::success_with_data(s.as_bytes().to_vec()),
                    None => SyscallResult::error("Invalid UTF-8 in path"),
                }
            }
            Err(e) => {
                error!("Failed to get working directory: {}", e);
                SyscallResult::error(format!("Get working directory failed: {}", e))
            }
        }
    }

    pub(super) fn set_working_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        if !self.sandbox_manager.check_path_access(pid, path) {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        match std::env::set_current_dir(path) {
            Ok(_) => {
                info!("PID {} set working directory: {:?}", pid, path);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to set working directory {:?}: {}", path, e);
                SyscallResult::error(format!("Set working directory failed: {}", e))
            }
        }
    }

    pub(super) fn truncate_file(&self, pid: Pid, path: &PathBuf, size: u64) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::WriteFile)
        {
            return SyscallResult::permission_denied("Missing WriteFile capability");
        }

        if !self.sandbox_manager.check_path_access(pid, path) {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        match fs::OpenOptions::new().write(true).open(path) {
            Ok(file) => match file.set_len(size) {
                Ok(_) => {
                    info!("PID {} truncated file {:?} to {} bytes", pid, path, size);
                    SyscallResult::success()
                }
                Err(e) => {
                    error!("Failed to truncate file {:?}: {}", path, e);
                    SyscallResult::error(format!("Truncate failed: {}", e))
                }
            },
            Err(e) => {
                error!("Failed to open file {:?} for truncation: {}", path, e);
                SyscallResult::error(format!("Open failed: {}", e))
            }
        }
    }
}
