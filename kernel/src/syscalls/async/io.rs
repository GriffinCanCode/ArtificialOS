/*!
 * True Async I/O Operations
 *
 * No spawn_blocking required - true cooperative async I/O
 */

use crate::core::types::Pid;
use crate::permissions::{PermissionChecker, PermissionRequest};
use crate::syscalls::types::SyscallResult;
use crate::vfs::MountManager;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::{error, info, trace};

/// Async file I/O operations using tokio::fs
pub struct AsyncFileOps {
    permission_checker: Arc<dyn PermissionChecker>,
    mount_manager: Arc<MountManager>,
}

impl AsyncFileOps {
    pub fn new(permission_checker: Arc<dyn PermissionChecker>, mount_manager: Arc<MountManager>) -> Self {
        Self {
            permission_checker,
            mount_manager,
        }
    }

    /// Read file asynchronously (true async I/O)
    #[inline]
    pub async fn read(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        // Permission check (fast, in-memory)
        let request = PermissionRequest::file_read(pid, path.clone());
        let response = self.permission_checker.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // TODO: Integrate with VFS properly - for now use path directly
        // True async I/O - yields to runtime
        match fs::read(path).await {
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

    /// Write file asynchronously (true async I/O)
    #[inline]
    pub async fn write(&self, pid: Pid, path: &PathBuf, data: &[u8]) -> SyscallResult {
        let request = PermissionRequest::file_write(pid, path.clone());
        let response = self.permission_checker.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // TODO: Integrate with VFS properly - for now use path directly
        // True async I/O - yields to runtime
        match fs::write(path, data).await {
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

    /// Create file asynchronously
    #[inline]
    pub async fn create(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        self.write(pid, path, &[]).await
    }

    /// Delete file asynchronously
    #[inline]
    pub async fn delete(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        let request = PermissionRequest::file_delete(pid, path.clone());
        let response = self.permission_checker.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // TODO: Integrate with VFS properly - for now use path directly
        match fs::remove_file(path).await {
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

    /// Get file metadata asynchronously
    #[inline]
    pub async fn metadata(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        let request = PermissionRequest::file_read(pid, path.clone());
        let response = self.permission_checker.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // TODO: Integrate with VFS properly - for now use path directly
        match fs::metadata(path).await {
            Ok(metadata) => {
                #[cfg(unix)]
                use std::os::unix::fs::PermissionsExt;

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
                match crate::core::serialization::json::to_vec(&file_info) {
                    Ok(json) => SyscallResult::success_with_data(json),
                    Err(e) => {
                        error!("Failed to serialize file stat: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(e) => {
                error!("Failed to stat file {:?}: {}", path, e);
                SyscallResult::error(format!("Stat failed: {}", e))
            }
        }
    }

    /// Copy file asynchronously (true async I/O)
    #[inline]
    pub async fn copy(&self, pid: Pid, source: &PathBuf, dest: &PathBuf) -> SyscallResult {
        // Check source read permission
        let req_src = PermissionRequest::file_read(pid, source.clone());
        let resp_src = self.permission_checker.check(&req_src);

        if !resp_src.is_allowed() {
            return SyscallResult::permission_denied(resp_src.reason());
        }

        // Check destination write permission
        let req_dst = PermissionRequest::file_create(pid, dest.clone());
        let resp_dst = self.permission_checker.check_and_audit(&req_dst);

        if !resp_dst.is_allowed() {
            return SyscallResult::permission_denied(resp_dst.reason());
        }

        // TODO: Integrate with VFS properly - for now use paths directly
        // True async copy
        match fs::copy(source, dest).await {
            Ok(bytes) => {
                info!("PID {} copied file: {:?} -> {:?} ({} bytes)", pid, source, dest, bytes);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to copy file {:?} -> {:?}: {}", source, dest, e);
                SyscallResult::error(format!("Copy failed: {}", e))
            }
        }
    }

    /// Move/rename file asynchronously
    #[inline]
    pub async fn rename(&self, pid: Pid, source: &PathBuf, dest: &PathBuf) -> SyscallResult {
        let req_src = PermissionRequest::file_delete(pid, source.clone());
        let resp_src = self.permission_checker.check_and_audit(&req_src);

        if !resp_src.is_allowed() {
            return SyscallResult::permission_denied(resp_src.reason());
        }

        let req_dst = PermissionRequest::file_create(pid, dest.clone());
        let resp_dst = self.permission_checker.check_and_audit(&req_dst);

        if !resp_dst.is_allowed() {
            return SyscallResult::permission_denied(resp_dst.reason());
        }

        // TODO: Integrate with VFS properly - for now use paths directly
        match fs::rename(source, dest).await {
            Ok(_) => {
                info!("PID {} moved file: {:?} -> {:?}", pid, source, dest);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to move file {:?} -> {:?}: {}", source, dest, e);
                SyscallResult::error(format!("Move failed: {}", e))
            }
        }
    }

    /// Read directory asynchronously
    #[inline]
    pub async fn read_dir(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        let request = PermissionRequest::file_read(pid, path.clone());
        let response = self.permission_checker.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // TODO: Integrate with VFS properly - for now use path directly
        // Read directory entries
        let mut entries = match fs::read_dir(path).await {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to read directory {:?}: {}", path, e);
                return SyscallResult::error(format!("Read dir failed: {}", e));
            }
        };

        let mut files = Vec::new();

        // Collect all entries asynchronously
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(metadata) = entry.metadata().await {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                #[cfg(unix)]
                use std::os::unix::fs::PermissionsExt;

                #[cfg(unix)]
                let mode = format!("{:o}", metadata.permissions().mode());
                #[cfg(not(unix))]
                let mode = String::from("0644");

                let file_info = serde_json::json!({
                    "name": name_str,
                    "path": entry.path().to_string_lossy(),
                    "size": metadata.len(),
                    "is_dir": metadata.is_dir(),
                    "mode": mode,
                    "modified": metadata.modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                });

                files.push(file_info);
            }
        }

        info!("PID {} listed directory: {:?} ({} entries)", pid, path, files.len());

        match crate::core::serialization::json::to_vec(&files) {
            Ok(json) => SyscallResult::success_with_data(json),
            Err(e) => {
                error!("Failed to serialize directory listing: {}", e);
                SyscallResult::error("Serialization failed")
            }
        }
    }

    /// Create directory asynchronously
    #[inline]
    pub async fn create_dir(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        let request = PermissionRequest::file_create(pid, path.clone());
        let response = self.permission_checker.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // TODO: Integrate with VFS properly - for now use path directly
        match fs::create_dir_all(path).await {
            Ok(_) => {
                info!("PID {} created directory: {:?}", pid, path);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to create directory {:?}: {}", path, e);
                SyscallResult::error(format!("Create dir failed: {}", e))
            }
        }
    }

    /// Remove directory asynchronously
    #[inline]
    pub async fn remove_dir(&self, pid: Pid, path: &PathBuf) -> SyscallResult {
        let request = PermissionRequest::file_delete(pid, path.clone());
        let response = self.permission_checker.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // TODO: Integrate with VFS properly - for now use path directly
        match fs::remove_dir_all(path).await {
            Ok(_) => {
                info!("PID {} removed directory: {:?}", pid, path);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Failed to remove directory {:?}: {}", path, e);
                SyscallResult::error(format!("Remove dir failed: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SandboxManager;
    use tempfile::TempDir;

    fn create_test_ops() -> (AsyncFileOps, TempDir, Pid) {
        let temp_dir = TempDir::new().unwrap();
        let sandbox = Arc::new(SandboxManager::new());
        let mount_manager = Arc::new(MountManager::new());
        let ops = AsyncFileOps::new(sandbox, mount_manager);
        let pid = 1;

        (ops, temp_dir, pid)
    }

    #[tokio::test]
    async fn test_async_read_write() {
        let (ops, temp_dir, pid) = create_test_ops();
        let test_file = temp_dir.path().join("test.txt");
        let test_data = b"Hello, async world!";

        // Write
        let result = ops.write(pid, &test_file, test_data).await;
        assert!(matches!(result, SyscallResult::Success { .. }));

        // Read
        let result = ops.read(pid, &test_file).await;
        match result {
            SyscallResult::Success { data: Some(data) } => {
                assert_eq!(data, test_data);
            }
            _ => panic!("Expected success with data"),
        }
    }

    #[tokio::test]
    async fn test_async_copy() {
        let (ops, temp_dir, pid) = create_test_ops();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");
        let test_data = b"Copy test";

        // Create source
        ops.write(pid, &source, test_data).await;

        // Copy
        let result = ops.copy(pid, &source, &dest).await;
        assert!(matches!(result, SyscallResult::Success { .. }));

        // Verify dest
        let result = ops.read(pid, &dest).await;
        match result {
            SyscallResult::Success { data: Some(data) } => {
                assert_eq!(data, test_data);
            }
            _ => panic!("Expected success with data"),
        }
    }

    #[tokio::test]
    async fn test_async_dir_operations() {
        let (ops, temp_dir, pid) = create_test_ops();
        let test_dir = temp_dir.path().join("testdir");

        // Create
        let result = ops.create_dir(pid, &test_dir).await;
        assert!(matches!(result, SyscallResult::Success { .. }));

        // Verify exists
        assert!(test_dir.exists());

        // Remove
        let result = ops.remove_dir(pid, &test_dir).await;
        assert!(matches!(result, SyscallResult::Success { .. }));

        // Verify removed
        assert!(!test_dir.exists());
    }
}

