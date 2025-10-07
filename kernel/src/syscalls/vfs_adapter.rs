/*!

* VFS Adapter for Syscalls
* Routes filesystem syscalls through VFS when available
*/

use crate::core::types::Pid;

use log::{info, warn};
use std::fs;
use std::path::Path;

use crate::security::Capability;
use crate::vfs::{FileSystem, VfsError};

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

impl SyscallExecutor {
    /// Read file using VFS if available, otherwise use std::fs
    pub(super) fn vfs_read(&self, pid: Pid, path: &Path) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile(None))
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        // Check sandbox path access
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        if !self.sandbox_manager.check_path_access(pid, &canonical_path) {
            return SyscallResult::permission_denied(format!(
                "Path not accessible: {:?}",
                canonical_path
            ));
        }

        // Try VFS first
        if let Some(ref vfs) = self.vfs {
            match vfs.read(path) {
                Ok(data) => {
                    info!(
                        "PID {} read file via VFS: {:?} ({} bytes)",
                        pid,
                        path,
                        data.len()
                    );
                    return SyscallResult::success_with_data(data);
                }
                Err(e) => {
                    warn!(
                        "VFS read failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                }
            }
        }

        // Fallback to std::fs
        match fs::read(&canonical_path) {
            Ok(data) => {
                info!("PID {} read file: {:?} ({} bytes)", pid, path, data.len());
                SyscallResult::success_with_data(data)
            }
            Err(e) => SyscallResult::error(format!("Read failed: {}", e)),
        }
    }

    /// Write file using VFS if available
    pub(super) fn vfs_write(&self, pid: Pid, path: &Path, data: &[u8]) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::WriteFile(None))
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
            path.to_path_buf()
        };

        if !self.sandbox_manager.check_path_access(pid, &check_path) {
            return SyscallResult::permission_denied(format!(
                "Path not accessible: {:?}",
                check_path
            ));
        }

        // Try VFS first
        if let Some(ref vfs) = self.vfs {
            match vfs.write(path, data) {
                Ok(()) => {
                    info!(
                        "PID {} wrote file via VFS: {:?} ({} bytes)",
                        pid,
                        path,
                        data.len()
                    );
                    return SyscallResult::success();
                }
                Err(e) => {
                    warn!(
                        "VFS write failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                }
            }
        }

        // Fallback to std::fs
        match fs::write(path, data) {
            Ok(_) => {
                info!("PID {} wrote file: {:?} ({} bytes)", pid, path, data.len());
                SyscallResult::success()
            }
            Err(e) => SyscallResult::error(format!("Write failed: {}", e)),
        }
    }

    /// Delete file using VFS if available
    pub(super) fn vfs_delete(&self, pid: Pid, path: &Path) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::DeleteFile(None))
        {
            return SyscallResult::permission_denied("Missing DeleteFile capability");
        }

        if !self
            .sandbox_manager
            .check_path_access(pid, &path.to_path_buf())
        {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        // Try VFS first
        if let Some(ref vfs) = self.vfs {
            match vfs.delete(path) {
                Ok(()) => {
                    info!("PID {} deleted file via VFS: {:?}", pid, path);
                    return SyscallResult::success();
                }
                Err(e) => {
                    warn!(
                        "VFS delete failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                }
            }
        }

        // Fallback to std::fs
        match fs::remove_file(path) {
            Ok(_) => {
                info!("PID {} deleted file: {:?}", pid, path);
                SyscallResult::success()
            }
            Err(e) => SyscallResult::error(format!("Delete failed: {}", e)),
        }
    }

    /// Check if file exists using VFS if available
    pub(super) fn vfs_exists(&self, pid: Pid, path: &Path) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile(None))
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        if !self
            .sandbox_manager
            .check_path_access(pid, &path.to_path_buf())
        {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        let exists = if let Some(ref vfs) = self.vfs {
            vfs.exists(path)
        } else {
            path.exists()
        };

        info!("PID {} checked file exists: {:?} = {}", pid, path, exists);
        let data = vec![if exists { 1 } else { 0 }];
        SyscallResult::success_with_data(data)
    }

    /// Create directory using VFS if available
    pub(super) fn vfs_create_dir(&self, pid: Pid, path: &Path) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::CreateFile(None))
        {
            return SyscallResult::permission_denied("Missing CreateFile capability");
        }

        if !self
            .sandbox_manager
            .check_path_access(pid, &path.to_path_buf())
        {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        // Try VFS first
        if let Some(ref vfs) = self.vfs {
            match vfs.create_dir(path) {
                Ok(()) => {
                    info!("PID {} created directory via VFS: {:?}", pid, path);
                    return SyscallResult::success();
                }
                Err(e) => {
                    warn!(
                        "VFS create_dir failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                }
            }
        }

        // Fallback to std::fs
        match fs::create_dir_all(path) {
            Ok(_) => {
                info!("PID {} created directory: {:?}", pid, path);
                SyscallResult::success()
            }
            Err(e) => SyscallResult::error(format!("Mkdir failed: {}", e)),
        }
    }

    /// Remove directory using VFS if available
    pub(super) fn vfs_remove_dir(&self, pid: Pid, path: &Path) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::DeleteFile(None))
        {
            return SyscallResult::permission_denied("Missing DeleteFile capability");
        }

        if !self
            .sandbox_manager
            .check_path_access(pid, &path.to_path_buf())
        {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        // Try VFS first
        if let Some(ref vfs) = self.vfs {
            match vfs.remove_dir_all(path) {
                Ok(()) => {
                    info!("PID {} removed directory via VFS: {:?}", pid, path);
                    return SyscallResult::success();
                }
                Err(e) => {
                    warn!(
                        "VFS remove_dir failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                }
            }
        }

        // Fallback to std::fs
        match fs::remove_dir_all(path) {
            Ok(_) => {
                info!("PID {} removed directory: {:?}", pid, path);
                SyscallResult::success()
            }
            Err(e) => SyscallResult::error(format!("Remove directory failed: {}", e)),
        }
    }

    /// List directory using VFS if available
    pub(super) fn vfs_list_dir(&self, pid: Pid, path: &Path) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ListDirectory(None))
        {
            return SyscallResult::permission_denied("Missing ListDirectory capability");
        }

        if !self
            .sandbox_manager
            .check_path_access(pid, &path.to_path_buf())
        {
            return SyscallResult::permission_denied(format!("Path not accessible: {:?}", path));
        }

        // Try VFS first
        if let Some(ref vfs) = self.vfs {
            match vfs.list_dir(path) {
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

                    match serde_json::to_vec(&files) {
                        Ok(json) => return SyscallResult::success_with_data(json),
                        Err(e) => {
                            return SyscallResult::error(format!(
                                "Failed to serialize directory listing: {}",
                                e
                            ))
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "VFS list_dir failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                }
            }
        }

        // Fallback to std::fs
        match fs::read_dir(path) {
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
                match serde_json::to_vec(&files) {
                    Ok(json) => SyscallResult::success_with_data(json),
                    Err(e) => SyscallResult::error(format!(
                        "Failed to serialize directory listing: {}",
                        e
                    )),
                }
            }
            Err(e) => SyscallResult::error(format!("List failed: {}", e)),
        }
    }
}

/// Convert VfsError to error message
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
