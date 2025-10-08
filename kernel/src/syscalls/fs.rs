/*!

* File System Syscalls
* File and directory operations
*/

use crate::core::json;
use crate::core::types::Pid;
use crate::core::{SyscallGuard, TransactionGuard, Operation};
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};

use log::{error, info, trace};
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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
        let mut guard = match &self.collector {
            Some(collector) => Some(SyscallGuard::new("file_stat", pid, collector.clone())),
            None => None,
        };

        let request = PermissionRequest::file_read(pid, path.clone());
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            let err = SyscallResult::permission_denied(response.reason());
            if let Some(ref mut g) = guard {
                g.record_result::<()>(&Err(response.reason()));
            }
            return err;
        }

        // Use timeout executor - metadata operations can block on slow/remote storage
        let path_clone = path.clone();
        let result = self.timeout_executor.execute_with_deadline(
            || fs::metadata(&path_clone),
            self.timeout_config.file_io,
            "file_stat",
        );

        let result = match result {
            Ok(metadata) => {
                #[cfg(unix)]
                let mode = format!("{:o}", metadata.permissions().mode());
                #[cfg(not(unix))]
                let mode = String::from("0644");

                let size = metadata.len();
                let is_dir = metadata.is_dir();

                trace!("File stat - size: {}, is_dir: {}, mode: {}", size, is_dir, mode);

                let file_info = serde_json::json!({
                    "name": path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    "path": path.to_str().unwrap_or(""),
                    "size": size,
                    "is_dir": is_dir,
                    "mode": mode,
                    "modified": metadata.modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                    "extension": path.extension().and_then(|e| e.to_str()).unwrap_or(""),
                });

                info!("PID {} stat file: {:?}", pid, path);
                match json::to_vec(&file_info) {
                    Ok(json) => Ok(SyscallResult::success_with_data(json)),
                    Err(e) => {
                        error!("Failed to serialize file stat: {}", e);
                        Err("Serialization failed".to_string())
                    }
                }
            }
            Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("File stat timed out for {:?} after {}ms (slow storage?)", path, elapsed_ms);
                Err(format!("Timeout after {}ms", elapsed_ms))
            }
            Err(super::TimeoutError::Operation(e)) => {
                error!("Failed to stat file {:?}: {}", path, e);
                Err(format!("Stat failed: {}", e))
            }
        };

        // Record result in guard
        if let Some(ref mut g) = guard {
            g.record_result(&result);
        }

        result.unwrap_or_else(|e| SyscallResult::error(e))
    }

    pub(super) fn move_file(
        &self,
        pid: Pid,
        source: &PathBuf,
        destination: &PathBuf,
    ) -> SyscallResult {
        let mut guard = match &self.collector {
            Some(collector) => Some(SyscallGuard::new("file_move", pid, collector.clone())),
            None => None,
        };

        // Check permission for source (read/delete)
        let req_src = PermissionRequest::file_delete(pid, source.clone());
        let resp_src = self.permission_manager.check_and_audit(&req_src);

        if !resp_src.is_allowed() {
            let err = SyscallResult::permission_denied(resp_src.reason());
            if let Some(ref mut g) = guard {
                g.record_result::<()>(&Err(format!("Source permission denied: {}", resp_src.reason())));
            }
            return err;
        }

        // Check permission for destination (write/create)
        let req_dst = PermissionRequest::file_create(pid, destination.clone());
        let resp_dst = self.permission_manager.check_and_audit(&req_dst);

        if !resp_dst.is_allowed() {
            let err = SyscallResult::permission_denied(resp_dst.reason());
            if let Some(ref mut g) = guard {
                g.record_result::<()>(&Err(format!("Destination permission denied: {}", resp_dst.reason())));
            }
            return err;
        }

        // Create transaction guard for atomic move operation
        // If the move fails, we ensure proper cleanup
        let src_backup = source.clone();
        let dst_backup = destination.clone();

        let mut transaction = TransactionGuard::new(
            Some(pid),
            |_ops| Ok(()), // Commit is a no-op for move (operation is atomic at OS level)
            move |_ops| {
                // Rollback: If destination was created, try to remove it
                // This is best-effort cleanup
                if dst_backup.exists() && !src_backup.exists() {
                    let _ = fs::remove_file(&dst_backup);
                }
                Ok(())
            },
        );

        transaction.add_operation(Operation::new(
            "move",
            format!("{:?} -> {:?}", source, destination).into_bytes(),
        )).ok();

        // Use timeout executor - rename can block on slow/cross-filesystem operations
        let src_clone = source.clone();
        let dst_clone = destination.clone();
        let result = self.timeout_executor.execute_with_deadline(
            || fs::rename(&src_clone, &dst_clone),
            self.timeout_config.file_io,
            "file_move",
        );

        let result = match result {
            Ok(_) => {
                info!("PID {} moved file: {:?} -> {:?}", pid, source, destination);
                transaction.commit().ok();
                Ok(SyscallResult::success())
            }
            Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("Move timed out for {:?} -> {:?} after {}ms (slow storage?)", source, destination, elapsed_ms);
                Err(format!("Timeout after {}ms", elapsed_ms))
            }
            Err(super::TimeoutError::Operation(e)) => {
                error!("Failed to move file {:?} -> {:?}: {}", source, destination, e);
                Err(format!("Move failed: {}", e))
            }
        };

        // Record result in guard
        if let Some(ref mut g) = guard {
            g.record_result(&result);
        }

        result.unwrap_or_else(|e| SyscallResult::error(e))
    }

    pub(super) fn copy_file(
        &self,
        pid: Pid,
        source: &PathBuf,
        destination: &PathBuf,
    ) -> SyscallResult {
        let mut guard = match &self.collector {
            Some(collector) => Some(SyscallGuard::new("file_copy", pid, collector.clone())),
            None => None,
        };

        // Check permission for source (read)
        let req_src = PermissionRequest::file_read(pid, source.clone());
        let resp_src = self.permission_manager.check(&req_src);

        if !resp_src.is_allowed() {
            let err = SyscallResult::permission_denied(resp_src.reason());
            if let Some(ref mut g) = guard {
                g.record_result::<()>(&Err(format!("Source permission denied: {}", resp_src.reason())));
            }
            return err;
        }

        // Check permission for destination (write/create)
        let req_dst = PermissionRequest::file_create(pid, destination.clone());
        let resp_dst = self.permission_manager.check_and_audit(&req_dst);

        if !resp_dst.is_allowed() {
            let err = SyscallResult::permission_denied(resp_dst.reason());
            if let Some(ref mut g) = guard {
                g.record_result::<()>(&Err(format!("Destination permission denied: {}", resp_dst.reason())));
            }
            return err;
        }

        // Create transaction guard for atomic copy with rollback
        // If copy fails partially, clean up the destination
        let dst_backup = destination.clone();

        let mut transaction = TransactionGuard::new(
            Some(pid),
            |_ops| Ok(()), // Commit is a no-op (copy operation is complete)
            move |_ops| {
                // Rollback: Remove partially copied file on failure
                if dst_backup.exists() {
                    let _ = fs::remove_file(&dst_backup);
                }
                Ok(())
            },
        );

        transaction.add_operation(Operation::new(
            "copy",
            format!("{:?} -> {:?}", source, destination).into_bytes(),
        )).ok();

        // Use timeout executor - copy can block on slow storage, especially for large files
        let src_clone = source.clone();
        let dst_clone = destination.clone();
        let result = self.timeout_executor.execute_with_deadline(
            || fs::copy(&src_clone, &dst_clone),
            self.timeout_config.file_io,
            "file_copy",
        );

        let result = match result {
            Ok(bytes) => {
                info!("PID {} copied file: {:?} -> {:?} ({} bytes)", pid, source, destination, bytes);
                transaction.commit().ok();
                Ok(SyscallResult::success())
            }
            Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("Copy timed out for {:?} -> {:?} after {}ms (slow storage or large file?)", source, destination, elapsed_ms);
                Err(format!("Timeout after {}ms", elapsed_ms))
            }
            Err(super::TimeoutError::Operation(e)) => {
                error!("Failed to copy file {:?} -> {:?}: {}", source, destination, e);
                Err(format!("Copy failed: {}", e))
            }
        };

        // Record result in guard
        if let Some(ref mut g) = guard {
            g.record_result(&result);
        }

        result.unwrap_or_else(|e| SyscallResult::error(e))
    }

    pub(super) fn create_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        self.vfs_create_dir(pid, path)
    }

    pub(super) fn remove_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        self.vfs_remove_dir(pid, path)
    }

    pub(super) fn get_working_directory(&self, pid: Pid) -> SyscallResult {
        let mut guard = match &self.collector {
            Some(collector) => Some(SyscallGuard::new("get_cwd", pid, collector.clone())),
            None => None,
        };

        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "cwd".to_string(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            let err = SyscallResult::permission_denied(response.reason());
            if let Some(ref mut g) = guard {
                g.record_result::<()>(&Err(response.reason()));
            }
            return err;
        }

        let result = match std::env::current_dir() {
            Ok(path) => {
                info!("PID {} retrieved working directory: {:?}", pid, path);
                match path.to_str() {
                    Some(s) => Ok(SyscallResult::success_with_data(s.as_bytes().to_vec())),
                    None => Err("Invalid UTF-8 in path".to_string()),
                }
            }
            Err(e) => {
                error!("Failed to get working directory: {}", e);
                Err(format!("Get working directory failed: {}", e))
            }
        };

        // Record result in guard
        if let Some(ref mut g) = guard {
            g.record_result(&result);
        }

        result.unwrap_or_else(|e| SyscallResult::error(e))
    }

    pub(super) fn set_working_directory(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        let mut guard = match &self.collector {
            Some(collector) => Some(SyscallGuard::new("set_cwd", pid, collector.clone())),
            None => None,
        };

        let request = PermissionRequest::file_read(pid, path.clone());
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            let err = SyscallResult::permission_denied(response.reason());
            if let Some(ref mut g) = guard {
                g.record_result::<()>(&Err(response.reason()));
            }
            return err;
        }

        let result = match std::env::set_current_dir(path) {
            Ok(_) => {
                info!("PID {} set working directory: {:?}", pid, path);
                Ok(SyscallResult::success())
            }
            Err(e) => {
                error!("Failed to set working directory {:?}: {}", path, e);
                Err(format!("Set working directory failed: {}", e))
            }
        };

        // Record result in guard
        if let Some(ref mut g) = guard {
            g.record_result(&result);
        }

        result.unwrap_or_else(|e| SyscallResult::error(e))
    }

    pub(super) fn truncate_file(&self, pid: Pid, path: &PathBuf, size: u64) -> SyscallResult {
        let mut guard = match &self.collector {
            Some(collector) => Some(SyscallGuard::new("file_truncate", pid, collector.clone())),
            None => None,
        };

        let request = PermissionRequest::file_write(pid, path.clone());
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            let err = SyscallResult::permission_denied(response.reason());
            if let Some(ref mut g) = guard {
                g.record_result::<()>(&Err(response.reason()));
            }
            return err;
        }

        // Create transaction guard with rollback capability
        // Store original size to restore on failure
        let path_backup = path.clone();
        let original_size = path.metadata().ok().map(|m| m.len());

        let mut transaction = TransactionGuard::new(
            Some(pid),
            |_ops| Ok(()), // Commit is a no-op (truncate is atomic)
            move |_ops| {
                // Rollback: Restore original size if possible
                if let Some(orig_size) = original_size {
                    if let Ok(file) = fs::OpenOptions::new().write(true).open(&path_backup) {
                        let _ = file.set_len(orig_size);
                    }
                }
                Ok(())
            },
        );

        transaction.add_operation(Operation::new(
            "truncate",
            format!("{:?} to {} bytes", path, size).into_bytes(),
        )).ok();

        // Use timeout executor - truncate can block on slow storage
        let path_clone = path.clone();
        let result: Result<(), super::TimeoutError<std::io::Error>> = self.timeout_executor.execute_with_deadline(
            || {
                let file = fs::OpenOptions::new().write(true).open(&path_clone)?;
                file.set_len(size)?;
                Ok(())
            },
            self.timeout_config.file_io,
            "file_truncate",
        );

        let result = match result {
            Ok(()) => {
                info!("PID {} truncated file {:?} to {} bytes", pid, path, size);
                transaction.commit().ok();
                Ok(SyscallResult::success())
            }
            Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("Truncate timed out for {:?} after {}ms (slow storage?)", path, elapsed_ms);
                Err(format!("Timeout after {}ms", elapsed_ms))
            }
            Err(super::TimeoutError::Operation(e)) => {
                error!("Failed to truncate file {:?}: {}", path, e);
                Err(format!("Truncate failed: {}", e))
            }
        };

        // Record result in guard
        if let Some(ref mut g) = guard {
            g.record_result(&result);
        }

        result.unwrap_or_else(|e| SyscallResult::error(e))
    }
}
