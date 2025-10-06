/*!

 * File Descriptor Syscalls
 * Low-level file descriptor operations
 */

use crate::core::types::Pid;

use log::{error, info, warn};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

/// File descriptor manager
pub struct FdManager {
    next_fd: Arc<RwLock<u32>>,
    open_files: Arc<RwLock<HashMap<u32, File>>>,
}

impl FdManager {
    pub fn new() -> Self {
        Self {
            next_fd: Arc::new(RwLock::new(3)), // Start at 3 (0, 1, 2 are stdin, stdout, stderr)
            open_files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn allocate_fd(&self) -> u32 {
        let mut next = self.next_fd.write().unwrap();
        let fd = *next;
        *next += 1;
        fd
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

        if read_flag != 0 && !self.sandbox_manager.check_permission(pid, &Capability::ReadFile) {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        if write_flag != 0 && !self.sandbox_manager.check_permission(pid, &Capability::WriteFile) {
            return SyscallResult::permission_denied("Missing WriteFile capability");
        }

        if create_flag != 0 && !self.sandbox_manager.check_permission(pid, &Capability::CreateFile) {
            return SyscallResult::permission_denied("Missing CreateFile capability");
        }

        // Check path access
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

        if !self.sandbox_manager.check_path_access(pid, &canonical_path) {
            return SyscallResult::permission_denied(format!(
                "Path not accessible: {:?}",
                canonical_path
            ));
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
            Ok(_file) => {
                // Allocate FD (placeholder - would store file in fd_manager)
                let fd = 10 + pid; // Mock FD
                info!("PID {} opened {:?} with FD {}, flags: 0x{:x}, mode: 0o{:o}", pid, path, fd, flags, mode);

                let data = serde_json::to_vec(&serde_json::json!({
                    "fd": fd
                })).unwrap();

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

        warn!("Close FD syscall not fully implemented: fd={}", fd);
        info!("PID {} closed FD {}", pid, fd);
        SyscallResult::success()
    }

    pub(super) fn dup(&self, pid: Pid, fd: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile)
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        warn!("Dup syscall not fully implemented: fd={}", fd);

        // Return mock new FD
        let new_fd = fd + 100;
        info!("PID {} duplicated FD {} to {}", pid, fd, new_fd);

        let data = serde_json::to_vec(&serde_json::json!({
            "new_fd": new_fd
        })).unwrap();

        SyscallResult::success_with_data(data)
    }

    pub(super) fn dup2(&self, pid: Pid, oldfd: u32, newfd: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile)
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        warn!("Dup2 syscall not fully implemented: oldfd={}, newfd={}", oldfd, newfd);
        info!("PID {} duplicated FD {} to {}", pid, oldfd, newfd);
        SyscallResult::success()
    }

    pub(super) fn lseek(&self, pid: Pid, fd: u32, offset: i64, whence: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile)
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        warn!("Lseek syscall not fully implemented: fd={}, offset={}, whence={}", fd, offset, whence);

        let whence_str = match whence {
            0 => "SEEK_SET",
            1 => "SEEK_CUR",
            2 => "SEEK_END",
            _ => "UNKNOWN",
        };

        info!("PID {} seeked FD {} to offset {} ({})", pid, fd, offset, whence_str);

        let data = serde_json::to_vec(&serde_json::json!({
            "offset": offset
        })).unwrap();

        SyscallResult::success_with_data(data)
    }

    pub(super) fn fcntl(&self, pid: Pid, fd: u32, cmd: u32, arg: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile)
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        warn!("Fcntl syscall not fully implemented: fd={}, cmd={}, arg={}", fd, cmd, arg);
        info!("PID {} performed fcntl on FD {}", pid, fd);

        let data = serde_json::to_vec(&serde_json::json!({
            "result": 0
        })).unwrap();

        SyscallResult::success_with_data(data)
    }
}
