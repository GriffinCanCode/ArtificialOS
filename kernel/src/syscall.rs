/*!
 * System Call Module
 * Provides safe, sandboxed access to OS operations
 */

use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::sandbox::{Capability, SandboxManager};

/// System call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyscallResult {
    Success { data: Option<Vec<u8>> },
    Error { message: String },
    PermissionDenied { reason: String },
}

impl SyscallResult {
    pub fn success() -> Self {
        Self::Success { data: None }
    }

    pub fn success_with_data(data: Vec<u8>) -> Self {
        Self::Success { data: Some(data) }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    pub fn permission_denied(reason: impl Into<String>) -> Self {
        Self::PermissionDenied {
            reason: reason.into(),
        }
    }
}

/// System call types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Syscall {
    // File system operations
    ReadFile {
        path: PathBuf,
    },
    WriteFile {
        path: PathBuf,
        data: Vec<u8>,
    },
    CreateFile {
        path: PathBuf,
    },
    DeleteFile {
        path: PathBuf,
    },
    ListDirectory {
        path: PathBuf,
    },
    FileExists {
        path: PathBuf,
    },
    FileStat {
        path: PathBuf,
    },
    MoveFile {
        source: PathBuf,
        destination: PathBuf,
    },
    CopyFile {
        source: PathBuf,
        destination: PathBuf,
    },
    CreateDirectory {
        path: PathBuf,
    },

    // Process operations
    SpawnProcess {
        command: String,
        args: Vec<String>,
    },
    KillProcess {
        target_pid: u32,
    },

    // System info
    GetSystemInfo,
    GetCurrentTime,
    GetEnvironmentVar {
        key: String,
    },

    // Network (placeholder for future)
    NetworkRequest {
        url: String,
    },

    // IPC - Pipes
    CreatePipe {
        reader_pid: u32,
        writer_pid: u32,
        capacity: Option<usize>,
    },
    WritePipe {
        pipe_id: u32,
        data: Vec<u8>,
    },
    ReadPipe {
        pipe_id: u32,
        size: usize,
    },
    ClosePipe {
        pipe_id: u32,
    },
    DestroyPipe {
        pipe_id: u32,
    },
    PipeStats {
        pipe_id: u32,
    },

    // IPC - Shared Memory
    CreateShm {
        size: usize,
    },
    AttachShm {
        segment_id: u32,
        read_only: bool,
    },
    DetachShm {
        segment_id: u32,
    },
    WriteShm {
        segment_id: u32,
        offset: usize,
        data: Vec<u8>,
    },
    ReadShm {
        segment_id: u32,
        offset: usize,
        size: usize,
    },
    DestroyShm {
        segment_id: u32,
    },
    ShmStats {
        segment_id: u32,
    },

    // Scheduler operations
    ScheduleNext,
    YieldProcess,
    GetCurrentScheduled,
    GetSchedulerStats,
}

/// System call executor
#[derive(Clone)]
pub struct SyscallExecutor {
    sandbox_manager: SandboxManager,
    pipe_manager: Option<crate::pipe::PipeManager>,
    shm_manager: Option<crate::shm::ShmManager>,
}

impl SyscallExecutor {
    pub fn new(sandbox_manager: SandboxManager) -> Self {
        info!("Syscall executor initialized");
        Self {
            sandbox_manager,
            pipe_manager: None,
            shm_manager: None,
        }
    }

    pub fn with_ipc(
        sandbox_manager: SandboxManager,
        pipe_manager: crate::pipe::PipeManager,
        shm_manager: crate::shm::ShmManager,
    ) -> Self {
        info!("Syscall executor initialized with IPC support");
        Self {
            sandbox_manager,
            pipe_manager: Some(pipe_manager),
            shm_manager: Some(shm_manager),
        }
    }

    /// Execute a system call with sandboxing
    pub fn execute(&self, pid: u32, syscall: Syscall) -> SyscallResult {
        info!("Executing syscall for PID {}: {:?}", pid, syscall);

        match syscall {
            // File operations
            Syscall::ReadFile { ref path } => self.read_file(pid, path),
            Syscall::WriteFile { ref path, ref data } => self.write_file(pid, path, data),
            Syscall::CreateFile { ref path } => self.create_file(pid, path),
            Syscall::DeleteFile { ref path } => self.delete_file(pid, path),
            Syscall::ListDirectory { ref path } => self.list_directory(pid, path),
            Syscall::FileExists { ref path } => self.file_exists(pid, path),
            Syscall::FileStat { ref path } => self.file_stat(pid, path),
            Syscall::MoveFile {
                ref source,
                ref destination,
            } => self.move_file(pid, source, destination),
            Syscall::CopyFile {
                ref source,
                ref destination,
            } => self.copy_file(pid, source, destination),
            Syscall::CreateDirectory { ref path } => self.create_directory(pid, path),

            // Process operations
            Syscall::SpawnProcess {
                ref command,
                ref args,
            } => self.spawn_process(pid, command, args),
            Syscall::KillProcess { target_pid } => self.kill_process(pid, target_pid),

            // System info
            Syscall::GetSystemInfo => self.get_system_info(pid),
            Syscall::GetCurrentTime => self.get_current_time(pid),
            Syscall::GetEnvironmentVar { ref key } => self.get_env_var(pid, key),

            // Network
            Syscall::NetworkRequest { ref url } => self.network_request(pid, url),

            // IPC - Pipes
            Syscall::CreatePipe {
                reader_pid,
                writer_pid,
                capacity,
            } => self.create_pipe(pid, reader_pid, writer_pid, capacity),
            Syscall::WritePipe { pipe_id, ref data } => self.write_pipe(pid, pipe_id, data),
            Syscall::ReadPipe { pipe_id, size } => self.read_pipe(pid, pipe_id, size),
            Syscall::ClosePipe { pipe_id } => self.close_pipe(pid, pipe_id),
            Syscall::DestroyPipe { pipe_id } => self.destroy_pipe(pid, pipe_id),
            Syscall::PipeStats { pipe_id } => self.pipe_stats(pid, pipe_id),

            // IPC - Shared Memory
            Syscall::CreateShm { size } => self.create_shm(pid, size),
            Syscall::AttachShm {
                segment_id,
                read_only,
            } => self.attach_shm(pid, segment_id, read_only),
            Syscall::DetachShm { segment_id } => self.detach_shm(pid, segment_id),
            Syscall::WriteShm {
                segment_id,
                offset,
                ref data,
            } => self.write_shm(pid, segment_id, offset, data),
            Syscall::ReadShm {
                segment_id,
                offset,
                size,
            } => self.read_shm(pid, segment_id, offset, size),
            Syscall::DestroyShm { segment_id } => self.destroy_shm(pid, segment_id),
            Syscall::ShmStats { segment_id } => self.shm_stats(pid, segment_id),

            // Scheduler operations
            Syscall::ScheduleNext => self.schedule_next(pid),
            Syscall::YieldProcess => self.yield_process(pid),
            Syscall::GetCurrentScheduled => self.get_current_scheduled(pid),
            Syscall::GetSchedulerStats => self.get_scheduler_stats(pid),
        }
    }

    // ========================================================================
    // File System Operations
    // ========================================================================

    fn read_file(&self, pid: u32, path: &PathBuf) -> SyscallResult {
        // Check capabilities
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReadFile)
        {
            return SyscallResult::permission_denied("Missing ReadFile capability");
        }

        // Canonicalize path to prevent TOCTOU via symlinks
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to canonicalize path {:?}: {}", path, e);
                return SyscallResult::error(format!("Invalid path: {}", e));
            }
        };

        // Check path access on canonical path
        if !self.sandbox_manager.check_path_access(pid, &canonical_path) {
            return SyscallResult::permission_denied(format!(
                "Path not accessible: {:?}",
                canonical_path
            ));
        }

        // Execute operation - minimize time window after check
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

    fn write_file(&self, pid: u32, path: &PathBuf, data: &[u8]) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::WriteFile)
        {
            return SyscallResult::permission_denied("Missing WriteFile capability");
        }

        // Canonicalize parent dir if file exists, otherwise check parent
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

    fn create_file(&self, pid: u32, path: &PathBuf) -> SyscallResult {
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

    fn delete_file(&self, pid: u32, path: &PathBuf) -> SyscallResult {
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

    fn list_directory(&self, pid: u32, path: &PathBuf) -> SyscallResult {
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

    fn file_exists(&self, pid: u32, path: &PathBuf) -> SyscallResult {
        // File existence check only needs read capability
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

    fn file_stat(&self, pid: u32, path: &PathBuf) -> SyscallResult {
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

    fn move_file(&self, pid: u32, source: &PathBuf, destination: &PathBuf) -> SyscallResult {
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

    fn copy_file(&self, pid: u32, source: &PathBuf, destination: &PathBuf) -> SyscallResult {
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

    fn create_directory(&self, pid: u32, path: &PathBuf) -> SyscallResult {
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

    // ========================================================================
    // Process Operations
    // ========================================================================

    fn spawn_process(&self, pid: u32, command: &str, args: &[String]) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SpawnProcess)
        {
            return SyscallResult::permission_denied("Missing SpawnProcess capability");
        }

        // Validate command - prevent shell injection
        if command.is_empty() || command.contains([';', '|', '&', '\n', '\0']) {
            error!("Invalid command attempted: {:?}", command);
            return SyscallResult::error("Invalid command: contains shell metacharacters");
        }

        // Validate args - prevent injection through arguments
        for arg in args {
            if arg.contains('\0') {
                error!("Invalid argument attempted: contains null byte");
                return SyscallResult::error("Invalid argument: contains null byte");
            }
        }

        // Check resource limits - enforce max_processes
        if let Some(limits) = self.sandbox_manager.get_limits(pid) {
            if !self.sandbox_manager.can_spawn_process(pid) {
                let current = self.sandbox_manager.get_spawn_count(pid);
                error!(
                    "PID {} exceeded process limit: {}/{} processes",
                    pid, current, limits.max_processes
                );
                return SyscallResult::permission_denied(format!(
                    "Process limit exceeded: {}/{} processes spawned",
                    current, limits.max_processes
                ));
            }
        }

        // Spawn process (sandboxed)
        match Command::new(command).args(args).output() {
            Ok(output) => {
                // Record successful spawn
                self.sandbox_manager.record_spawn(pid);

                info!("PID {} spawned process: {} {:?}", pid, command, args);
                let process_output = ProcessOutput {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code().unwrap_or(-1),
                };

                // Process completed, decrement count
                self.sandbox_manager.record_termination(pid);

                match serde_json::to_vec(&process_output) {
                    Ok(result) => SyscallResult::success_with_data(result),
                    Err(e) => {
                        error!("Failed to serialize process output: {}", e);
                        SyscallResult::error("Failed to serialize process output")
                    }
                }
            }
            Err(e) => {
                error!("Failed to spawn process: {}", e);
                SyscallResult::error(format!("Spawn failed: {}", e))
            }
        }
    }

    fn kill_process(&self, pid: u32, target_pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::KillProcess)
        {
            return SyscallResult::permission_denied("Missing KillProcess capability");
        }

        // Clean up sandbox for terminated process
        self.sandbox_manager.remove_sandbox(target_pid);

        info!(
            "PID {} terminated PID {} and cleaned up sandbox",
            pid, target_pid
        );
        SyscallResult::success()
    }

    // ========================================================================
    // System Info Operations
    // ========================================================================

    fn get_system_info(&self, pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let info = SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            family: std::env::consts::FAMILY.to_string(),
        };

        info!("PID {} retrieved system info", pid);
        match serde_json::to_vec(&info) {
            Ok(data) => SyscallResult::success_with_data(data),
            Err(e) => {
                error!("Failed to serialize system info: {}", e);
                SyscallResult::error(format!("Failed to serialize system info: {}", e))
            }
        }
    }

    fn get_current_time(&self, pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::TimeAccess)
        {
            return SyscallResult::permission_denied("Missing TimeAccess capability");
        }

        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => {
                let timestamp = duration.as_secs();
                info!("PID {} retrieved current time: {}", pid, timestamp);
                let data = timestamp.to_le_bytes().to_vec();
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("System time error: {}", e);
                SyscallResult::error(format!("Failed to get system time: {}", e))
            }
        }
    }

    fn get_env_var(&self, pid: u32, key: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        match std::env::var(key) {
            Ok(value) => {
                info!("PID {} read env var: {} = {}", pid, key, value);
                SyscallResult::success_with_data(value.into_bytes())
            }
            Err(_) => SyscallResult::error(format!("Environment variable not found: {}", key)),
        }
    }

    // ========================================================================
    // Network Operations
    // ========================================================================

    fn network_request(&self, pid: u32, _url: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        // Placeholder for network operations
        warn!("Network operations not yet implemented");
        SyscallResult::error("Network operations not implemented")
    }

    // ========================================================================
    // IPC Operations - Pipes
    // ========================================================================

    fn create_pipe(
        &self,
        pid: u32,
        reader_pid: u32,
        writer_pid: u32,
        capacity: Option<usize>,
    ) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SendMessage)
        {
            return SyscallResult::permission_denied("Missing SendMessage capability");
        }

        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.create(reader_pid, writer_pid, capacity) {
            Ok(pipe_id) => {
                info!("PID {} created pipe {}", pid, pipe_id);
                match serde_json::to_vec(&pipe_id) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        error!("Failed to serialize pipe ID: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(e) => {
                error!("Failed to create pipe: {}", e);
                SyscallResult::error(format!("Pipe creation failed: {}", e))
            }
        }
    }

    fn write_pipe(&self, pid: u32, pipe_id: u32, data: &[u8]) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SendMessage)
        {
            return SyscallResult::permission_denied("Missing SendMessage capability");
        }

        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.write(pipe_id, pid, data) {
            Ok(written) => {
                info!("PID {} wrote {} bytes to pipe {}", pid, written, pipe_id);
                match serde_json::to_vec(&written) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        error!("Failed to serialize write result: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(e) => {
                error!("Pipe write failed: {}", e);
                SyscallResult::error(format!("Pipe write failed: {}", e))
            }
        }
    }

    fn read_pipe(&self, pid: u32, pipe_id: u32, size: usize) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReceiveMessage)
        {
            return SyscallResult::permission_denied("Missing ReceiveMessage capability");
        }

        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.read(pipe_id, pid, size) {
            Ok(data) => {
                info!("PID {} read {} bytes from pipe {}", pid, data.len(), pipe_id);
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Pipe read failed: {}", e);
                SyscallResult::error(format!("Pipe read failed: {}", e))
            }
        }
    }

    fn close_pipe(&self, pid: u32, pipe_id: u32) -> SyscallResult {
        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.close(pipe_id, pid) {
            Ok(_) => {
                info!("PID {} closed pipe {}", pid, pipe_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Pipe close failed: {}", e);
                SyscallResult::error(format!("Pipe close failed: {}", e))
            }
        }
    }

    fn destroy_pipe(&self, pid: u32, pipe_id: u32) -> SyscallResult {
        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.destroy(pipe_id) {
            Ok(_) => {
                info!("PID {} destroyed pipe {}", pid, pipe_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Pipe destroy failed: {}", e);
                SyscallResult::error(format!("Pipe destroy failed: {}", e))
            }
        }
    }

    fn pipe_stats(&self, pid: u32, pipe_id: u32) -> SyscallResult {
        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.stats(pipe_id) {
            Ok(stats) => match serde_json::to_vec(&stats) {
                Ok(data) => {
                    info!("PID {} retrieved stats for pipe {}", pid, pipe_id);
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize pipe stats: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            Err(e) => {
                error!("Pipe stats failed: {}", e);
                SyscallResult::error(format!("Pipe stats failed: {}", e))
            }
        }
    }

    // ========================================================================
    // IPC Operations - Shared Memory
    // ========================================================================

    fn create_shm(&self, pid: u32, size: usize) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SendMessage)
        {
            return SyscallResult::permission_denied("Missing SendMessage capability");
        }

        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.create(size, pid) {
            Ok(segment_id) => {
                info!("PID {} created shared memory segment {} ({} bytes)", pid, segment_id, size);
                match serde_json::to_vec(&segment_id) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        error!("Failed to serialize segment ID: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(e) => {
                error!("Failed to create shared memory: {}", e);
                SyscallResult::error(format!("Shared memory creation failed: {}", e))
            }
        }
    }

    fn attach_shm(&self, pid: u32, segment_id: u32, read_only: bool) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReceiveMessage)
        {
            return SyscallResult::permission_denied("Missing ReceiveMessage capability");
        }

        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.attach(segment_id, pid, read_only) {
            Ok(_) => {
                info!("PID {} attached to segment {} (read_only: {})", pid, segment_id, read_only);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Shared memory attach failed: {}", e);
                SyscallResult::error(format!("Attach failed: {}", e))
            }
        }
    }

    fn detach_shm(&self, pid: u32, segment_id: u32) -> SyscallResult {
        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.detach(segment_id, pid) {
            Ok(_) => {
                info!("PID {} detached from segment {}", pid, segment_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Shared memory detach failed: {}", e);
                SyscallResult::error(format!("Detach failed: {}", e))
            }
        }
    }

    fn write_shm(&self, pid: u32, segment_id: u32, offset: usize, data: &[u8]) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SendMessage)
        {
            return SyscallResult::permission_denied("Missing SendMessage capability");
        }

        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.write(segment_id, pid, offset, data) {
            Ok(_) => {
                info!("PID {} wrote {} bytes to segment {} at offset {}", pid, data.len(), segment_id, offset);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Shared memory write failed: {}", e);
                SyscallResult::error(format!("Write failed: {}", e))
            }
        }
    }

    fn read_shm(&self, pid: u32, segment_id: u32, offset: usize, size: usize) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::ReceiveMessage)
        {
            return SyscallResult::permission_denied("Missing ReceiveMessage capability");
        }

        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.read(segment_id, pid, offset, size) {
            Ok(data) => {
                info!("PID {} read {} bytes from segment {} at offset {}", pid, data.len(), segment_id, offset);
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Shared memory read failed: {}", e);
                SyscallResult::error(format!("Read failed: {}", e))
            }
        }
    }

    fn destroy_shm(&self, pid: u32, segment_id: u32) -> SyscallResult {
        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.destroy(segment_id, pid) {
            Ok(_) => {
                info!("PID {} destroyed segment {}", pid, segment_id);
                SyscallResult::success()
            }
            Err(e) => {
                error!("Shared memory destroy failed: {}", e);
                SyscallResult::error(format!("Destroy failed: {}", e))
            }
        }
    }

    fn shm_stats(&self, pid: u32, segment_id: u32) -> SyscallResult {
        let shm_manager = match &self.shm_manager {
            Some(sm) => sm,
            None => return SyscallResult::error("Shared memory manager not available"),
        };

        match shm_manager.stats(segment_id) {
            Ok(stats) => match serde_json::to_vec(&stats) {
                Ok(data) => {
                    info!("PID {} retrieved stats for segment {}", pid, segment_id);
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize segment stats: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            Err(e) => {
                error!("Shared memory stats failed: {}", e);
                SyscallResult::error(format!("Stats failed: {}", e))
            }
        }
    }

    // ========================================================================
    // Scheduler Operations
    // ========================================================================

    fn schedule_next(&self, _pid: u32) -> SyscallResult {
        info!("Schedule next syscall requested");
        // Note: Scheduler operations would require ProcessManager access
        // This is a placeholder that returns success
        // In a full implementation, this would call ProcessManager::schedule_next()
        SyscallResult::success()
    }

    fn yield_process(&self, pid: u32) -> SyscallResult {
        info!("Process {} yielding CPU", pid);
        // Note: This would call ProcessManager::yield_current()
        SyscallResult::success()
    }

    fn get_current_scheduled(&self, _pid: u32) -> SyscallResult {
        info!("Get current scheduled process requested");
        // Note: This would call ProcessManager::current_scheduled()
        SyscallResult::success()
    }

    fn get_scheduler_stats(&self, pid: u32) -> SyscallResult {
        info!("PID {} requested scheduler statistics", pid);
        // Note: This would call Scheduler::stats() via ProcessManager
        // Placeholder implementation
        SyscallResult::success()
    }
}

// Helper structs for serialization
#[derive(Debug, Serialize, Deserialize)]
struct ProcessOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemInfo {
    os: String,
    arch: String,
    family: String,
}
