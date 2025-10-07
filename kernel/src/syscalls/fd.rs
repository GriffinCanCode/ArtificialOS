/*!

* File Descriptor Syscalls
* Low-level file descriptor operations
*/

use crate::core::json;
use crate::core::types::Pid;
use crate::permissions::{PermissionChecker, PermissionRequest};

use dashmap::DashMap;
use ahash::RandomState;
use log::{error, info, warn};
use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

/// File descriptor manager
pub struct FdManager {
    next_fd: Arc<AtomicU32>,
    open_files: Arc<DashMap<u32, Arc<RwLock<File>>, RandomState>>,
}

impl FdManager {
    pub fn new() -> Self {
        Self {
            next_fd: Arc::new(AtomicU32::new(3)), // Start at 3 (0, 1, 2 are stdin, stdout, stderr)
            open_files: Arc::new(DashMap::with_hasher(RandomState::new())),
        }
    }

    fn allocate_fd(&self) -> u32 {
        self.next_fd.fetch_add(1, Ordering::SeqCst)
    }
}

impl Clone for FdManager {
    fn clone(&self) -> Self {
        Self {
            next_fd: Arc::clone(&self.next_fd),
            open_files: Arc::clone(&self.open_files),
        }
    }
}

impl SyscallExecutor {
    pub(super) fn open(&self, pid: Pid, path: &PathBuf, flags: u32, mode: u32) -> SyscallResult {
        // Check permissions based on flags
        let read_flag = flags & 0x0001; // O_RDONLY or O_RDWR
        let write_flag = flags & 0x0002; // O_WRONLY or O_RDWR
        let create_flag = flags & 0x0040; // O_CREAT

        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                if create_flag != 0 {
                    path.clone()
                } else {
                    return SyscallResult::error("File does not exist");
                }
            }
        };

        // Check permissions using centralized manager based on operation
        if create_flag != 0 {
            let request = PermissionRequest::file_create(pid, canonical_path.clone());
            let response = self.permission_manager.check_and_audit(&request);
            if !response.is_allowed() {
                return SyscallResult::permission_denied(response.reason());
            }
        } else if write_flag != 0 {
            let request = PermissionRequest::file_write(pid, canonical_path.clone());
            let response = self.permission_manager.check_and_audit(&request);
            if !response.is_allowed() {
                return SyscallResult::permission_denied(response.reason());
            }
        } else if read_flag != 0 {
            let request = PermissionRequest::file_read(pid, canonical_path.clone());
            let response = self.permission_manager.check_and_audit(&request);
            if !response.is_allowed() {
                return SyscallResult::permission_denied(response.reason());
            }
        }

        // Build OpenOptions based on flags
        let mut options = OpenOptions::new();

        // Access mode
        if flags & 0x0002 != 0 {
            // O_WRONLY
            options.write(true);
        } else if flags & 0x0003 != 0 {
            // O_RDWR
            options.read(true).write(true);
        } else {
            // O_RDONLY (default)
            options.read(true);
        }

        // Additional flags
        if flags & 0x0040 != 0 {
            options.create(true); // O_CREAT
        }
        if flags & 0x0200 != 0 {
            options.truncate(true); // O_TRUNC
        }
        if flags & 0x0400 != 0 {
            options.append(true); // O_APPEND
        }

        match options.open(path) {
            Ok(file) => {
                // Allocate FD and store file with Arc for reference counting
                let fd = self.fd_manager.allocate_fd();
                self.fd_manager
                    .open_files
                    .insert(fd, Arc::new(RwLock::new(file)));

                info!(
                    "PID {} opened {:?} with FD {}, flags: 0x{:x}, mode: 0o{:o}",
                    pid, path, fd, flags, mode
                );

                let data = json::to_vec(&serde_json::json!({
                    "fd": fd
                }))
                .unwrap();

                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Failed to open file {:?}: {}", path, e);
                SyscallResult::error(format!("Open failed: {}", e))
            }
        }
    }

    pub(super) fn close_fd(&self, pid: Pid, fd: u32) -> SyscallResult {
        // No capability check - closing is always allowed

        // Remove file from fd_manager
        if self.fd_manager.open_files.remove(&fd).is_some() {
            info!("PID {} closed FD {}", pid, fd);
            SyscallResult::success()
        } else {
            warn!("PID {} attempted to close non-existent FD {}", pid, fd);
            SyscallResult::error("Invalid file descriptor")
        }
    }

    pub(super) fn dup(&self, pid: Pid, fd: u32) -> SyscallResult {
        // Note: dup doesn't check specific path permissions, just general file access
        // The original fd already had permissions checked at open time

        // Check if the FD exists and clone the Arc reference
        if let Some(file_ref) = self.fd_manager.open_files.get(&fd) {
            // Clone the Arc to increment reference count
            let cloned_file = Arc::clone(file_ref.value());

            // Allocate new FD pointing to same file via Arc
            let new_fd = self.fd_manager.allocate_fd();
            self.fd_manager.open_files.insert(new_fd, cloned_file);

            info!(
                "PID {} duplicated FD {} to {} (Arc reference count incremented)",
                pid, fd, new_fd
            );

            let data = json::to_vec(&serde_json::json!({
                "new_fd": new_fd
            }))
            .unwrap();

            SyscallResult::success_with_data(data)
        } else {
            SyscallResult::error("Invalid file descriptor")
        }
    }

    pub(super) fn dup2(&self, pid: Pid, oldfd: u32, newfd: u32) -> SyscallResult {
        // Note: dup2 doesn't check specific path permissions, just general file access
        // The original fd already had permissions checked at open time

        // Check if the old FD exists and clone the Arc reference
        if let Some(file_ref) = self.fd_manager.open_files.get(&oldfd) {
            // Clone the Arc to increment reference count
            let cloned_file = Arc::clone(file_ref.value());

            // If newfd is already open, close it first (Arc will auto-drop)
            if self.fd_manager.open_files.contains_key(&newfd) {
                self.fd_manager.open_files.remove(&newfd);
                info!("PID {} closed existing FD {} before dup2", pid, newfd);
            }

            // Insert the cloned Arc reference at newfd
            self.fd_manager.open_files.insert(newfd, cloned_file);

            info!(
                "PID {} duplicated FD {} to {} (Arc reference count incremented)",
                pid, oldfd, newfd
            );
            SyscallResult::success()
        } else {
            SyscallResult::error("Invalid file descriptor")
        }
    }

    pub(super) fn lseek(&self, pid: Pid, fd: u32, offset: i64, whence: u32) -> SyscallResult {
        // Note: lseek operates on already-open fds with validated permissions

        if let Some(file_arc) = self.fd_manager.open_files.get(&fd) {
            let mut file = file_arc.write();
            let seek_result = match whence {
                0 => file.seek(SeekFrom::Start(offset as u64)), // SEEK_SET
                1 => file.seek(SeekFrom::Current(offset)),      // SEEK_CUR
                2 => file.seek(SeekFrom::End(offset)),          // SEEK_END
                _ => {
                    return SyscallResult::error("Invalid whence value");
                }
            };

            match seek_result {
                Ok(new_offset) => {
                    let whence_str = match whence {
                        0 => "SEEK_SET",
                        1 => "SEEK_CUR",
                        2 => "SEEK_END",
                        _ => "UNKNOWN",
                    };

                    info!(
                        "PID {} seeked FD {} to offset {} ({})",
                        pid, fd, new_offset, whence_str
                    );

                    let data = json::to_vec(&serde_json::json!({
                        "offset": new_offset
                    }))
                    .unwrap();

                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Seek failed for FD {}: {}", fd, e);
                    SyscallResult::error(format!("Seek failed: {}", e))
                }
            }
        } else {
            SyscallResult::error("Invalid file descriptor")
        }
    }

    pub(super) fn fcntl(&self, pid: Pid, fd: u32, cmd: u32, arg: u32) -> SyscallResult {
        // Note: fcntl operates on already-open fds with validated permissions

        // Verify FD exists
        if !self.fd_manager.open_files.contains_key(&fd) {
            return SyscallResult::error("Invalid file descriptor");
        }

        // Basic fcntl commands (F_GETFD, F_SETFD, etc.)
        // For now, we acknowledge the command but don't implement full functionality
        info!(
            "PID {} performed fcntl on FD {} (cmd={}, arg={})",
            pid, fd, cmd, arg
        );

        let data = json::to_vec(&serde_json::json!({
            "result": 0
        }))
        .unwrap();

        SyscallResult::success_with_data(data)
    }
}
