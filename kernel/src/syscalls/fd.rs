/*!

* File Descriptor Syscalls
* Low-level file descriptor operations
*/

use crate::core::json;
use crate::core::types::Pid;
use crate::permissions::{PermissionChecker, PermissionRequest};
use crate::vfs::{FileSystem, OpenFlags, OpenMode};

use ahash::RandomState;
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use log::{error, info, warn};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use super::executor::SyscallExecutor;
use super::handle::FileHandle;
use super::types::SyscallResult;

/// File descriptor manager
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic FD counter (high-frequency I/O)
/// - HashSet-based per-process tracking for O(1) untrack operations
/// - Lock-free FD recycling via SegQueue to prevent FD exhaustion
/// - Atomic count tracking for O(1) limit checks
/// - Unified with VFS via FileHandle for all filesystem backends
#[repr(C, align(64))]
pub struct FdManager {
    next_fd: Arc<AtomicU32>,
    open_files: Arc<DashMap<u32, Arc<FileHandle>, RandomState>>,
    /// Track which FDs belong to which process (HashSet for O(1) untrack)
    process_fds: Arc<DashMap<Pid, HashSet<u32>, RandomState>>,
    /// Per-process FD counts for O(1) limit checks (lock-free via alter())
    process_fd_counts: Arc<DashMap<Pid, u32, RandomState>>,
    /// Lock-free queue for FD recycling (prevents FD exhaustion)
    free_fds: Arc<SegQueue<u32>>,
}

impl FdManager {
    pub fn new() -> Self {
        info!("FD manager initialized with lock-free FD recycling and O(1) tracking");
        Self {
            next_fd: Arc::new(AtomicU32::new(3)), // Start at 3 (0, 1, 2 are stdin, stdout, stderr)
            open_files: Arc::new(DashMap::with_hasher(RandomState::new())),
            process_fds: Arc::new(DashMap::with_hasher(RandomState::new())),
            process_fd_counts: Arc::new(DashMap::with_hasher(RandomState::new())),
            free_fds: Arc::new(SegQueue::new()),
        }
    }

    /// Allocate an FD (recycle or create new, lock-free)
    fn allocate_fd(&self) -> u32 {
        if let Some(recycled_fd) = self.free_fds.pop() {
            recycled_fd
        } else {
            self.next_fd.fetch_add(1, Ordering::SeqCst)
        }
    }

    /// Get current FD count for a process (O(1) lookup)
    pub fn get_fd_count(&self, pid: Pid) -> u32 {
        self.process_fd_counts
            .get(&pid)
            .map(|r| *r.value())
            .unwrap_or(0)
    }

    /// Check if process has any open FDs
    pub fn has_process_fds(&self, pid: Pid) -> bool {
        self.get_fd_count(pid) > 0
    }

    /// Track that a process owns an FD (atomic increment)
    pub(super) fn track_fd(&self, pid: Pid, fd: u32) {
        // Use entry API for HashSet (simpler and correct)
        self.process_fds
            .entry(pid)
            .or_insert_with(HashSet::new)
            .insert(fd);

        // Atomic increment using entry() for lock-free counting
        *self.process_fd_counts.entry(pid).or_insert(0) += 1;
    }

    /// Untrack an FD from a process (atomic decrement, O(1) removal)
    pub(super) fn untrack_fd(&self, pid: Pid, fd: u32) {
        if let Some(mut fds) = self.process_fds.get_mut(&pid) {
            fds.remove(&fd); // O(1) HashSet removal
        }

        // Atomic decrement using get_mut() for lock-free counting
        if let Some(mut count) = self.process_fd_counts.get_mut(&pid) {
            *count = count.saturating_sub(1);
        }
    }

    /// Cleanup all file descriptors for a terminated process
    pub fn cleanup_process_fds(&self, pid: Pid) -> usize {
        let fds_to_close = if let Some((_, fds)) = self.process_fds.remove(&pid) {
            fds.into_iter().collect::<Vec<_>>()
        } else {
            return 0;
        };

        let mut closed_count = 0;
        for fd in fds_to_close {
            if self.open_files.remove(&fd).is_some() {
                closed_count += 1;
                // Recycle FD for reuse (lock-free)
                self.free_fds.push(fd);
            }
        }

        // Remove the count entry
        self.process_fd_counts.remove(&pid);

        closed_count
    }
}

impl Clone for FdManager {
    fn clone(&self) -> Self {
        Self {
            next_fd: Arc::clone(&self.next_fd),
            open_files: Arc::clone(&self.open_files),
            process_fds: Arc::clone(&self.process_fds),
            process_fd_counts: Arc::clone(&self.process_fd_counts),
            free_fds: Arc::clone(&self.free_fds),
        }
    }
}

impl SyscallExecutor {
    pub(super) fn open(&self, pid: Pid, path: &PathBuf, flags: u32, mode: u32) -> SyscallResult {
        // Check per-process FD limit BEFORE doing expensive operations
        use crate::security::ResourceLimitProvider;
        if let Some(limits) = self.sandbox_manager.get_limits(pid) {
            let current_fd_count = self.fd_manager.get_fd_count(pid);
            if current_fd_count >= limits.max_file_descriptors {
                error!(
                    "PID {} exceeded FD limit: {}/{} file descriptors",
                    pid, current_fd_count, limits.max_file_descriptors
                );
                return SyscallResult::permission_denied(format!(
                    "File descriptor limit exceeded: {}/{} FDs open",
                    current_fd_count, limits.max_file_descriptors
                ));
            }
        }

        // Check permissions based on flags
        let read_flag = flags & 0x0001; // O_RDONLY or O_RDWR
        let write_flag = flags & 0x0002; // O_WRONLY or O_RDWR
        let create_flag = flags & 0x0040; // O_CREAT

        // For permission checks, use the path as-is (VFS handles its own resolution)
        let check_path = path.clone();

        // Check permissions using centralized manager based on operation
        if create_flag != 0 {
            let request = PermissionRequest::file_create(pid, check_path.clone());
            let response = self.permission_manager.check_and_audit(&request);
            if !response.is_allowed() {
                return SyscallResult::permission_denied(response.reason());
            }
        } else if write_flag != 0 {
            let request = PermissionRequest::file_write(pid, check_path.clone());
            let response = self.permission_manager.check_and_audit(&request);
            if !response.is_allowed() {
                return SyscallResult::permission_denied(response.reason());
            }
        } else if read_flag != 0 {
            let request = PermissionRequest::file_read(pid, check_path.clone());
            let response = self.permission_manager.check_and_audit(&request);
            if !response.is_allowed() {
                return SyscallResult::permission_denied(response.reason());
            }
        }

        // Try VFS first for unified file handle
        if let Some(ref vfs) = self.vfs {
            let vfs_flags = OpenFlags::from_posix(flags);
            let vfs_mode = OpenMode::new(mode);

            match vfs.open(path, vfs_flags, vfs_mode) {
                Ok(vfs_file) => {
                    // Allocate FD and store VFS file handle
                    let fd = self.fd_manager.allocate_fd();
                    let handle = Arc::new(FileHandle::from_vfs(vfs_file));
                    self.fd_manager.open_files.insert(fd, handle);
                    self.fd_manager.track_fd(pid, fd);

                info!(
                    "PID {} opened {:?} via VFS with FD {}, flags: 0x{:x}",
                    pid, path, fd, flags
                );

                return match json::to_vec(&serde_json::json!({ "fd": fd })) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        // Clean up FD on serialization error to prevent leak
                        let _ = self.close_fd(pid, fd);
                        warn!("Failed to serialize open result: {}", e);
                        SyscallResult::error("Internal serialization error")
                    }
                };
                }
                Err(e) => {
                    warn!(
                        "VFS open failed for {:?}: {}, falling back to std::fs",
                        path, e
                    );
                }
            }
        }

        // Fallback to std::fs if VFS unavailable or failed
        // Canonicalize path for std::fs (only for real filesystem)
        let std_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                if create_flag != 0 {
                    path.clone()
                } else {
                    return SyscallResult::error("File does not exist");
                }
            }
        };

        let mut options = OpenOptions::new();

        // Access mode
        if flags & 0x0002 != 0 {
            options.write(true); // O_WRONLY
        } else if flags & 0x0003 != 0 {
            options.read(true).write(true); // O_RDWR
        } else {
            options.read(true); // O_RDONLY (default)
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

        match options.open(&std_path) {
            Ok(file) => {
                // Allocate FD and store std file handle
                let fd = self.fd_manager.allocate_fd();
                let handle = Arc::new(FileHandle::from_std(file));
                self.fd_manager.open_files.insert(fd, handle);
                self.fd_manager.track_fd(pid, fd);

                info!(
                    "PID {} opened {:?} with FD {}, flags: 0x{:x}, mode: 0o{:o}",
                    pid, path, fd, flags, mode
                );

                match json::to_vec(&serde_json::json!({ "fd": fd })) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        // Clean up FD on serialization error to prevent leak
                        let _ = self.close_fd(pid, fd);
                        warn!("Failed to serialize open result: {}", e);
                        SyscallResult::error("Internal serialization error")
                    }
                }
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
            // Untrack FD from process
            self.fd_manager.untrack_fd(pid, fd);
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

        // Check per-process FD limit BEFORE duplication
        use crate::security::ResourceLimitProvider;
        if let Some(limits) = self.sandbox_manager.get_limits(pid) {
            let current_fd_count = self.fd_manager.get_fd_count(pid);
            if current_fd_count >= limits.max_file_descriptors {
                error!(
                    "PID {} exceeded FD limit during dup: {}/{} file descriptors",
                    pid, current_fd_count, limits.max_file_descriptors
                );
                return SyscallResult::permission_denied(format!(
                    "File descriptor limit exceeded: {}/{} FDs open",
                    current_fd_count, limits.max_file_descriptors
                ));
            }
        }

        // Check if the FD exists and clone the Arc<FileHandle> reference
        if let Some(handle_ref) = self.fd_manager.open_files.get(&fd) {
            // Clone the Arc to increment reference count
            let cloned_handle = Arc::clone(handle_ref.value());

            // Allocate new FD pointing to same handle via Arc
            let new_fd = self.fd_manager.allocate_fd();
            self.fd_manager.open_files.insert(new_fd, cloned_handle);

            // Track the new FD for this process
            self.fd_manager.track_fd(pid, new_fd);

            info!(
                "PID {} duplicated FD {} to {} (Arc reference count incremented)",
                pid, fd, new_fd
            );

            match json::to_vec(&serde_json::json!({
                "new_fd": new_fd
            })) {
                Ok(data) => SyscallResult::success_with_data(data),
                Err(e) => {
                    warn!("Failed to serialize dup result: {}", e);
                    SyscallResult::error("Internal serialization error")
                }
            }
        } else {
            SyscallResult::error("Invalid file descriptor")
        }
    }

    pub(super) fn dup2(&self, pid: Pid, oldfd: u32, newfd: u32) -> SyscallResult {
        // Note: dup2 doesn't check specific path permissions, just general file access
        // The original fd already had permissions checked at open time

        // Check per-process FD limit BEFORE dup2 (only if newfd is not already open)
        use crate::security::ResourceLimitProvider;
        if !self.fd_manager.open_files.contains_key(&newfd) {
            if let Some(limits) = self.sandbox_manager.get_limits(pid) {
                let current_fd_count = self.fd_manager.get_fd_count(pid);
                if current_fd_count >= limits.max_file_descriptors {
                    error!(
                        "PID {} exceeded FD limit during dup2: {}/{} file descriptors",
                        pid, current_fd_count, limits.max_file_descriptors
                    );
                    return SyscallResult::permission_denied(format!(
                        "File descriptor limit exceeded: {}/{} FDs open",
                        current_fd_count, limits.max_file_descriptors
                    ));
                }
            }
        }

        // Check if the old FD exists and clone the Arc<FileHandle> reference
        if let Some(handle_ref) = self.fd_manager.open_files.get(&oldfd) {
            // Clone the Arc to increment reference count
            let cloned_handle = Arc::clone(handle_ref.value());

            // If newfd is already open, close it first (Arc will auto-drop)
            if self.fd_manager.open_files.contains_key(&newfd) {
                self.fd_manager.open_files.remove(&newfd);
                self.fd_manager.untrack_fd(pid, newfd);
                info!("PID {} closed existing FD {} before dup2", pid, newfd);
            }

            // Insert the cloned Arc reference at newfd
            self.fd_manager.open_files.insert(newfd, cloned_handle);

            // Track the new FD for this process
            self.fd_manager.track_fd(pid, newfd);

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

        if let Some(handle_arc) = self.fd_manager.open_files.get(&fd) {
            let seek_pos = match whence {
                0 => SeekFrom::Start(offset as u64), // SEEK_SET
                1 => SeekFrom::Current(offset),      // SEEK_CUR
                2 => SeekFrom::End(offset),          // SEEK_END
                _ => {
                    return SyscallResult::error("Invalid whence value");
                }
            };

            match handle_arc.seek(seek_pos) {
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

                    match json::to_vec(&serde_json::json!({
                        "offset": new_offset
                    })) {
                        Ok(data) => SyscallResult::success_with_data(data),
                        Err(e) => {
                            warn!("Failed to serialize lseek result: {}", e);
                            SyscallResult::error("Internal serialization error")
                        }
                    }
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

        match json::to_vec(&serde_json::json!({
            "result": 0
        })) {
            Ok(data) => SyscallResult::success_with_data(data),
            Err(e) => {
                warn!("Failed to serialize fcntl result: {}", e);
                SyscallResult::error("Internal serialization error")
            }
        }
    }

    pub(super) fn fsync_fd(&self, pid: Pid, fd: u32) -> SyscallResult {
        // Fsync synchronizes file data and metadata to disk

        if let Some(handle_arc) = self.fd_manager.open_files.get(&fd) {
            match handle_arc.sync() {
                Ok(_) => {
                    info!("PID {} synchronized FD {} to disk", pid, fd);
                    SyscallResult::success()
                }
                Err(e) => {
                    error!("Fsync failed for FD {}: {}", fd, e);
                    SyscallResult::error(format!("Fsync failed: {}", e))
                }
            }
        } else {
            SyscallResult::error("Invalid file descriptor")
        }
    }

    #[allow(dead_code)]
    pub(super) fn fdatasync_fd(&self, pid: Pid, fd: u32) -> SyscallResult {
        // Fdatasync synchronizes file data (not metadata) to disk

        if let Some(handle_arc) = self.fd_manager.open_files.get(&fd) {
            match handle_arc.sync_data() {
                Ok(_) => {
                    info!("PID {} synchronized FD {} data to disk", pid, fd);
                    SyscallResult::success()
                }
                Err(e) => {
                    error!("Fdatasync failed for FD {}: {}", fd, e);
                    SyscallResult::error(format!("Fdatasync failed: {}", e))
                }
            }
        } else {
            SyscallResult::error("Invalid file descriptor")
        }
    }
}
