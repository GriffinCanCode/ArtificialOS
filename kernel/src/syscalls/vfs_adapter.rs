/*!

* VFS Adapter for Syscalls
* Routes filesystem syscalls through VFS when available
*/

use crate::core::json;
use crate::core::types::Pid;
use crate::monitoring::span_operation;
use crate::permissions::{PermissionChecker, PermissionRequest};

use log::{error, info, trace, warn};
use std::fs;
use std::path::Path;

use crate::vfs::{FileSystem, VfsError};

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

impl SyscallExecutor {
    /// Read file using VFS if available, otherwise use std::fs
    /// Can block on slow storage (NFS, USB, slow disks)
    pub(super) fn vfs_read(&self, pid: Pid, path: &Path) -> SyscallResult {
        let span = span_operation("vfs_read");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("path", &format!("{:?}", path));

        // Check permission using centralized manager
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        let request = PermissionRequest::file_read(pid, canonical_path.clone());
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Try VFS first with timeout
        if let Some(ref vfs) = self.vfs {
            let vfs_clone = vfs.clone();
            let path_clone = path.to_path_buf();

            let result = self.timeout_executor.execute_with_deadline(
                || vfs_clone.read(&path_clone),
                self.timeout_config.file_io,
                "vfs_read",
            );

            match result {
                Ok(data) => {
                    info!(
                        "PID {} read file via VFS: {:?} ({} bytes)",
                        pid,
                        path,
                        data.len()
                    );
                    span.record("bytes_read", &format!("{}", data.len()));
                    span.record("method", "vfs");
                    span.record_result(true);
                    return SyscallResult::success_with_data(data);
                }
                Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                    warn!(
                        "VFS read timed out for {:?} after {}ms (slow storage?), falling back to std::fs",
                        path, elapsed_ms
                    );
                    span.record("vfs_timeout_ms", &format!("{}", elapsed_ms));
                }
                Err(super::TimeoutError::Operation(e)) => {
                    warn!(
                        "VFS read failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                    span.record("vfs_error", &format!("{}", e));
                }
            }
        }

        // Fallback to std::fs with timeout
        trace!("Falling back to std::fs for read");
        let canonical_clone = canonical_path.clone();
        let result = self.timeout_executor.execute_with_deadline(
            || fs::read(&canonical_clone),
            self.timeout_config.file_io,
            "fs_read",
        );

        match result {
            Ok(data) => {
                info!("PID {} read file: {:?} ({} bytes)", pid, path, data.len());
                span.record("bytes_read", &format!("{}", data.len()));
                span.record("method", "std::fs");
                span.record_result(true);
                SyscallResult::success_with_data(data)
            }
            Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("Read timed out for {:?} after {}ms", path, elapsed_ms);
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error(format!("Read timed out after {}ms (slow storage?)", elapsed_ms))
            }
            Err(super::TimeoutError::Operation(e)) => {
                error!("Read failed for {:?}: {}", path, e);
                span.record_error(&format!("Read failed: {}", e));
                SyscallResult::error(format!("Read failed: {}", e))
            }
        }
    }

    /// Write file using VFS if available
    /// Can block on slow storage (NFS, USB, slow disks)
    pub(super) fn vfs_write(&self, pid: Pid, path: &Path, data: &[u8]) -> SyscallResult {
        let span = span_operation("vfs_write");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("path", &format!("{:?}", path));
        span.record("data_len", &format!("{}", data.len()));

        let file_exists = path.exists();
        let check_path = if file_exists {
            match path.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    warn!("Failed to canonicalize path {:?}: {}", path, e);
                    span.record_error(&format!("Path canonicalization failed: {}", e));
                    return SyscallResult::error(format!("Invalid path: {}", e));
                }
            }
        } else {
            path.to_path_buf()
        };

        // Check permission using centralized manager
        // If file doesn't exist, this is a create operation; otherwise it's a write
        let request = if file_exists {
            PermissionRequest::file_write(pid, check_path.clone())
        } else {
            PermissionRequest::file_create(pid, check_path.clone())
        };
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        let data_len = data.len();

        // Try VFS first with timeout
        if let Some(ref vfs) = self.vfs {
            let vfs_clone = vfs.clone();
            let path_clone = path.to_path_buf();
            let data_clone = data.to_vec();

            let result = self.timeout_executor.execute_with_deadline(
                || vfs_clone.write(&path_clone, &data_clone),
                self.timeout_config.file_io,
                "vfs_write",
            );

            match result {
                Ok(()) => {
                    info!(
                        "PID {} wrote file via VFS: {:?} ({} bytes)",
                        pid, path, data_len
                    );
                    span.record("method", "vfs");
                    span.record_result(true);
                    return SyscallResult::success();
                }
                Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                    warn!(
                        "VFS write timed out for {:?} after {}ms (slow storage?), falling back to std::fs",
                        path, elapsed_ms
                    );
                    span.record("vfs_timeout_ms", &format!("{}", elapsed_ms));
                }
                Err(super::TimeoutError::Operation(e)) => {
                    warn!(
                        "VFS write failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                    span.record("vfs_error", &format!("{}", e));
                }
            }
        }

        // Fallback to std::fs with timeout
        trace!("Falling back to std::fs for write");
        let path_clone = path.to_path_buf();
        let data_clone = data.to_vec();
        let result = self.timeout_executor.execute_with_deadline(
            || fs::write(&path_clone, &data_clone),
            self.timeout_config.file_io,
            "fs_write",
        );

        match result {
            Ok(_) => {
                info!("PID {} wrote file: {:?} ({} bytes)", pid, path, data_len);
                span.record("method", "std::fs");
                span.record_result(true);
                SyscallResult::success()
            }
            Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("Write timed out for {:?} after {}ms", path, elapsed_ms);
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error(format!("Write timed out after {}ms (slow storage?)", elapsed_ms))
            }
            Err(super::TimeoutError::Operation(e)) => {
                error!("Write failed for {:?}: {}", path, e);
                span.record_error(&format!("Write failed: {}", e));
                SyscallResult::error(format!("Write failed: {}", e))
            }
        }
    }

    /// Delete file using VFS if available
    /// Can block on slow storage (NFS, USB, slow disks)
    pub(super) fn vfs_delete(&self, pid: Pid, path: &Path) -> SyscallResult {
        let span = span_operation("vfs_delete");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("path", &format!("{:?}", path));

        // Check permission using centralized manager
        let request = PermissionRequest::file_delete(pid, path.to_path_buf());
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Try VFS first with timeout
        if let Some(ref vfs) = self.vfs {
            let vfs_clone = vfs.clone();
            let path_clone = path.to_path_buf();

            let result = self.timeout_executor.execute_with_deadline(
                || vfs_clone.delete(&path_clone),
                self.timeout_config.file_io,
                "vfs_delete",
            );

            match result {
                Ok(()) => {
                    info!("PID {} deleted file via VFS: {:?}", pid, path);
                    span.record("method", "vfs");
                    span.record_result(true);
                    return SyscallResult::success();
                }
                Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                    warn!(
                        "VFS delete timed out for {:?} after {}ms (slow storage?), falling back to std::fs",
                        path, elapsed_ms
                    );
                    span.record("vfs_timeout_ms", &format!("{}", elapsed_ms));
                }
                Err(super::TimeoutError::Operation(e)) => {
                    warn!(
                        "VFS delete failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                    span.record("vfs_error", &format!("{}", e));
                }
            }
        }

        // Fallback to std::fs with timeout
        trace!("Falling back to std::fs for delete");
        let path_clone = path.to_path_buf();
        let result = self.timeout_executor.execute_with_deadline(
            || fs::remove_file(&path_clone),
            self.timeout_config.file_io,
            "fs_delete",
        );

        match result {
            Ok(_) => {
                info!("PID {} deleted file: {:?}", pid, path);
                span.record("method", "std::fs");
                span.record_result(true);
                SyscallResult::success()
            }
            Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("Delete timed out for {:?} after {}ms", path, elapsed_ms);
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error(format!("Delete timed out after {}ms (slow storage?)", elapsed_ms))
            }
            Err(super::TimeoutError::Operation(e)) => {
                error!("Delete failed for {:?}: {}", path, e);
                span.record_error(&format!("Delete failed: {}", e));
                SyscallResult::error(format!("Delete failed: {}", e))
            }
        }
    }

    /// Check if file exists using VFS if available
    pub(super) fn vfs_exists(&self, pid: Pid, path: &Path) -> SyscallResult {
        let span = span_operation("vfs_exists");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("path", &format!("{:?}", path));

        // Check permission using centralized manager
        let request = PermissionRequest::file_read(pid, path.to_path_buf());
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        let (exists, method) = if let Some(ref vfs) = self.vfs {
            (vfs.exists(path), "vfs")
        } else {
            (path.exists(), "std::fs")
        };

        info!("PID {} checked file exists: {:?} = {}", pid, path, exists);
        span.record("exists", &format!("{}", exists));
        span.record("method", method);
        span.record_result(true);
        let data = vec![if exists { 1 } else { 0 }];
        SyscallResult::success_with_data(data)
    }

    /// Create directory using VFS if available
    /// Can block on slow storage (NFS, USB, slow disks)
    pub(super) fn vfs_create_dir(&self, pid: Pid, path: &Path) -> SyscallResult {
        let span = span_operation("vfs_create_dir");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("path", &format!("{:?}", path));

        // Check permission using centralized manager
        let request = PermissionRequest::file_create(pid, path.to_path_buf());
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Try VFS first with timeout
        if let Some(ref vfs) = self.vfs {
            let vfs_clone = vfs.clone();
            let path_clone = path.to_path_buf();

            let result = self.timeout_executor.execute_with_deadline(
                || vfs_clone.create_dir(&path_clone),
                self.timeout_config.file_io,
                "vfs_create_dir",
            );

            match result {
                Ok(()) => {
                    info!("PID {} created directory via VFS: {:?}", pid, path);
                    span.record("method", "vfs");
                    span.record_result(true);
                    return SyscallResult::success();
                }
                Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                    warn!(
                        "VFS create_dir timed out for {:?} after {}ms (slow storage?), falling back to std::fs",
                        path, elapsed_ms
                    );
                    span.record("vfs_timeout_ms", &format!("{}", elapsed_ms));
                }
                Err(super::TimeoutError::Operation(e)) => {
                    warn!(
                        "VFS create_dir failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                    span.record("vfs_error", &format!("{}", e));
                }
            }
        }

        // Fallback to std::fs with timeout
        trace!("Falling back to std::fs for create_dir");
        let path_clone = path.to_path_buf();
        let result = self.timeout_executor.execute_with_deadline(
            || fs::create_dir_all(&path_clone),
            self.timeout_config.file_io,
            "fs_create_dir",
        );

        match result {
            Ok(_) => {
                info!("PID {} created directory: {:?}", pid, path);
                span.record("method", "std::fs");
                span.record_result(true);
                SyscallResult::success()
            }
            Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("Create directory timed out for {:?} after {}ms", path, elapsed_ms);
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error(format!("Mkdir timed out after {}ms (slow storage?)", elapsed_ms))
            }
            Err(super::TimeoutError::Operation(e)) => {
                error!("Create directory failed for {:?}: {}", path, e);
                span.record_error(&format!("Mkdir failed: {}", e));
                SyscallResult::error(format!("Mkdir failed: {}", e))
            }
        }
    }

    /// Remove directory using VFS if available
    /// Can block on slow storage, especially for large/nested directories
    pub(super) fn vfs_remove_dir(&self, pid: Pid, path: &Path) -> SyscallResult {
        let span = span_operation("vfs_remove_dir");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("path", &format!("{:?}", path));

        // Check permission using centralized manager
        let request = PermissionRequest::file_delete(pid, path.to_path_buf());
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Try VFS first with timeout
        if let Some(ref vfs) = self.vfs {
            let vfs_clone = vfs.clone();
            let path_clone = path.to_path_buf();

            let result = self.timeout_executor.execute_with_deadline(
                || vfs_clone.remove_dir_all(&path_clone),
                self.timeout_config.file_io,
                "vfs_remove_dir",
            );

            match result {
                Ok(()) => {
                    info!("PID {} removed directory via VFS: {:?}", pid, path);
                    span.record("method", "vfs");
                    span.record_result(true);
                    return SyscallResult::success();
                }
                Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                    warn!(
                        "VFS remove_dir timed out for {:?} after {}ms (slow storage or large directory?), falling back to std::fs",
                        path, elapsed_ms
                    );
                    span.record("vfs_timeout_ms", &format!("{}", elapsed_ms));
                }
                Err(super::TimeoutError::Operation(e)) => {
                    warn!(
                        "VFS remove_dir failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                    span.record("vfs_error", &format!("{}", e));
                }
            }
        }

        // Fallback to std::fs with timeout
        trace!("Falling back to std::fs for remove_dir");
        let path_clone = path.to_path_buf();
        let result = self.timeout_executor.execute_with_deadline(
            || fs::remove_dir_all(&path_clone),
            self.timeout_config.file_io,
            "fs_remove_dir",
        );

        match result {
            Ok(_) => {
                info!("PID {} removed directory: {:?}", pid, path);
                span.record("method", "std::fs");
                span.record_result(true);
                SyscallResult::success()
            }
            Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("Remove directory timed out for {:?} after {}ms", path, elapsed_ms);
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error(format!("Remove directory timed out after {}ms (slow storage or large directory?)", elapsed_ms))
            }
            Err(super::TimeoutError::Operation(e)) => {
                error!("Remove directory failed for {:?}: {}", path, e);
                span.record_error(&format!("Remove directory failed: {}", e));
                SyscallResult::error(format!("Remove directory failed: {}", e))
            }
        }
    }

    /// List directory using VFS if available
    /// Can block on slow storage, especially for large directories
    pub(super) fn vfs_list_dir(&self, pid: Pid, path: &Path) -> SyscallResult {
        let span = span_operation("vfs_list_dir");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("path", &format!("{:?}", path));

        // Check permission using centralized manager
        let request = PermissionRequest::dir_list(pid, path.to_path_buf());
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Try VFS first with timeout
        if let Some(ref vfs) = self.vfs {
            let vfs_clone = vfs.clone();
            let path_clone = path.to_path_buf();

            let result = self.timeout_executor.execute_with_deadline(
                || vfs_clone.list_dir(&path_clone),
                self.timeout_config.file_io,
                "vfs_list_dir",
            );

            match result {
                Ok(entries) => {
                    // Include file type information in the response
                    let files: Vec<serde_json::Value> = entries
                        .into_iter()
                        .map(|e| {
                            let is_dir =
                                matches!(e.file_type, crate::vfs::types::FileType::Directory);
                            serde_json::json!({
                                "name": e.name,
                                "is_dir": is_dir,
                                "type": if is_dir { "directory" } else { "file" }
                            })
                        })
                        .collect();

                    info!(
                        "PID {} listed directory via VFS: {:?} ({} entries)",
                        pid,
                        path,
                        files.len()
                    );
                    span.record("entry_count", &format!("{}", files.len()));
                    span.record("method", "vfs");

                    match json::serialize_vfs_batch(&files) {
                        Ok(json) => {
                            span.record_result(true);
                            return SyscallResult::success_with_data(json);
                        }
                        Err(e) => {
                            error!("Failed to serialize VFS directory listing: {}", e);
                            span.record_error(&format!("Serialization failed: {}", e));
                            return SyscallResult::error(format!(
                                "Failed to serialize directory listing: {}",
                                e
                            ));
                        }
                    }
                }
                Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                    warn!(
                        "VFS list_dir timed out for {:?} after {}ms (slow storage or large directory?), falling back to std::fs",
                        path, elapsed_ms
                    );
                    span.record("vfs_timeout_ms", &format!("{}", elapsed_ms));
                }
                Err(super::TimeoutError::Operation(e)) => {
                    warn!(
                        "VFS list_dir failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                    span.record("vfs_error", &format!("{}", e));
                }
            }
        }

        // Fallback to std::fs with timeout
        trace!("Falling back to std::fs for list_dir");
        let path_clone = path.to_path_buf();
        let result = self.timeout_executor.execute_with_deadline(
            || fs::read_dir(&path_clone),
            self.timeout_config.file_io,
            "fs_list_dir",
        );

        match result {
            Ok(entries) => {
                let files: Vec<serde_json::Value> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|entry| {
                        let name = entry.file_name().into_string().ok()?;
                        let is_dir = entry.file_type().ok()?.is_dir();
                        Some(serde_json::json!({
                            "name": name,
                            "is_dir": is_dir,
                            "type": if is_dir { "directory" } else { "file" }
                        }))
                    })
                    .collect();

                info!(
                    "PID {} listed directory: {:?} ({} entries)",
                    pid,
                    path,
                    files.len()
                );
                span.record("entry_count", &format!("{}", files.len()));
                span.record("method", "std::fs");
                match json::serialize_vfs_batch(&files) {
                    Ok(json) => {
                        span.record_result(true);
                        SyscallResult::success_with_data(json)
                    }
                    Err(e) => {
                        error!("Failed to serialize directory listing: {}", e);
                        span.record_error(&format!("Serialization failed: {}", e));
                        SyscallResult::error(format!(
                            "Failed to serialize directory listing: {}",
                            e
                        ))
                    }
                }
            }
            Err(super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("List directory timed out for {:?} after {}ms", path, elapsed_ms);
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error(format!("List directory timed out after {}ms (slow storage or large directory?)", elapsed_ms))
            }
            Err(super::TimeoutError::Operation(e)) => {
                error!("List directory failed for {:?}: {}", path, e);
                span.record_error(&format!("List failed: {}", e));
                SyscallResult::error(format!("List failed: {}", e))
            }
        }
    }
}

/// Convert VfsError to error message
#[allow(dead_code)]
fn vfs_error_to_string(err: VfsError) -> String {
    match err {
        VfsError::NotFound(msg) => format!("Not found: {}", msg),
        VfsError::PermissionDenied(msg) => format!("Permission denied: {}", msg),
        VfsError::AlreadyExists(msg) => format!("Already exists: {}", msg),
        VfsError::IsADirectory(msg) => format!("Is a directory: {}", msg),
        VfsError::NotADirectory(msg) => format!("Not a directory: {}", msg),
        VfsError::InvalidPath(msg) => format!("Invalid path: {}", msg),
        VfsError::IoError(msg) => format!("I/O error: {}", msg),
        VfsError::NotSupported(msg) => format!("Not supported: {}", msg),
        VfsError::OutOfSpace => "Out of space".to_string(),
        VfsError::InvalidArgument(msg) => format!("Invalid argument: {}", msg),
        VfsError::FileTooLarge => "File too large".to_string(),
        VfsError::ReadOnly => "Read-only filesystem".to_string(),
        VfsError::CrossDevice => "Cross-device link".to_string(),
    }
}
