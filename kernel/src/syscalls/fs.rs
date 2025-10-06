/*!
 * File System Syscalls
 * File and directory operations
 */

use log::{error, info, warn};
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

impl SyscallExecutor {
    pub(super) fn read_file(&self, pid: u32, path: &PathBuf) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile)
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to canonicalize path {:?}: {}", path, e);
                return SyscallResult::error(format!("Invalid path: {}", e));
            }
        };

        if !self.sandbox_manager.check_path_access(pid, &canonical_path) {
            return SyscallResult::permission_denied(format!(
                "Path not accessible: {:?}",
                canonical_path
            ));
        }

        match fs::read(&canonical_path) {
            Ok(data) => {
                info!("PID {} read file: {:?} ({} bytes)", pid, path, data.len());
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Failed to read file {:?}: {}", path, e);
                SyscallResult::error(format!("Read failed: {}", e))
            }
        }
    }

    pub(super) fn write_file(&self, pid: u32, path: &PathBuf, data: &[u8]) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::WriteFile)
        {
            return SyscallResult::permission_denied("Missing WriteFile capability");
        }

        let check_path = if path.exists() {
            match path.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    warn!("Failed to canonicalize path {:?}: {}", path, e);
                    return SyscallResult::error(format!("Invalid path: {}", e));
                }
            }
        } else {
            path.clone()
        };

        if !self.sandbox_manager.check_path_access(pid, &check_path) {
            return SyscallResult::permission_denied(format!(
                "Path not accessible: {:?}",
                check_path
            ));
        }

        match fs::write(path, data) {
            Ok(_) => {
                info!("PID {} wrote file: {:?} ({} bytes)", pid, path, data.len());
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to write file {:?}: {}", path, e);
                SyscallResult::error(format!("Write failed: {}", e))
            }
        }
    }

    pub(super) fn create_file(&self, pid: u32, path: &PathBuf) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::CreateFile)
        {
            return SyscallResult::permission_denied("Missing CreateFile capability");
        }

        if !self.sandbox_manager.check_path_access(pid, path) {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        match fs::File::create(path) {
            Ok(_) => {
                info!("PID {} created file: {:?}", pid, path);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to create file {:?}: {}", path, e);
                SyscallResult::error(format!("Create failed: {}", e))
            }
        }
    }

    pub(super) fn delete_file(&self, pid: u32, path: &PathBuf) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::DeleteFile)
        {
            return SyscallResult::permission_denied("Missing DeleteFile capability");
        }

        if !self.sandbox_manager.check_path_access(pid, path) {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        match fs::remove_file(path) {
            Ok(_) => {
                info!("PID {} deleted file: {:?}", pid, path);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to delete file {:?}: {}", path, e);
                SyscallResult::error(format!("Delete failed: {}", e))
            }
        }
    }

    pub(super) fn list_directory(&self, pid: u32, path: &PathBuf) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ListDirectory)
        {
            return SyscallResult::permission_denied("Missing ListDirectory capability");
        }

        if !self.sandbox_manager.check_path_access(pid, path) {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        match fs::read_dir(path) {
            Ok(entries) => {
                let files: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect();

                info!(
                    "PID {} listed directory: {:?} ({} entries)",
                    pid,
                    path,
                    files.len()
                );
                match serde_json::to_vec(&files) {
                    Ok(json) => SyscallResult::success_with_data(json),
                    Err(e) => {
                        error!("Failed to serialize directory listing: {}", e);
                        SyscallResult::error("Failed to serialize directory listing")
                    }
                }
            }
            Err(e) => {
                error!("Failed to list directory {:?}: {}", path, e);
                SyscallResult::error(format!("List failed: {}", e))
            }
        }
    }

    pub(super) fn file_exists(&self, pid: u32, path: &PathBuf) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile)
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        if !self.sandbox_manager.check_path_access(pid, path) {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        let exists = path.exists();
        info!("PID {} checked file exists: {:?} = {}", pid, path, exists);
        let data = vec![if exists { 1 } else { 0 }];
        SyscallResult::success_with_data(data)
    }

    pub(super) fn file_stat(&self, pid: u32, path: &PathBuf) -> SyscallResult {
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

    pub(super) fn move_file(&self, pid: u32, source: &PathBuf, destination: &PathBuf) -> SyscallResult {
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

    pub(super) fn copy_file(&self, pid: u32, source: &PathBuf, destination: &PathBuf) -> SyscallResult {
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

    pub(super) fn create_directory(&self, pid: u32, path: &PathBuf) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::CreateFile)
        {
            return SyscallResult::permission_denied("Missing CreateFile capability");
        }

        if !self.sandbox_manager.check_path_access(pid, path) {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        match fs::create_dir_all(path) {
            Ok(_) => {
                info!("PID {} created directory: {:?}", pid, path);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to create directory {:?}: {}", path, e);
                SyscallResult::error(format!("Mkdir failed: {}", e))
            }
        }
    }

    pub(super) fn remove_directory(&self, pid: u32, path: &PathBuf) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::DeleteFile)
        {
            return SyscallResult::permission_denied("Missing DeleteFile capability");
        }

        if !self.sandbox_manager.check_path_access(pid, path) {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        match fs::remove_dir_all(path) {
            Ok(_) => {
                info!("PID {} removed directory: {:?}", pid, path);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to remove directory {:?}: {}", path, e);
                SyscallResult::error(format!("Remove directory failed: {}", e))
            }
        }
    }

    pub(super) fn get_working_directory(&self, pid: u32) -> SyscallResult {
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

    pub(super) fn set_working_directory(&self, pid: u32, path: &PathBuf) -> SyscallResult {
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

    pub(super) fn truncate_file(&self, pid: u32, path: &PathBuf, size: u64) -> SyscallResult {
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
