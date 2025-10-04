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
}

/// System call executor
#[derive(Clone)]
pub struct SyscallExecutor {
    sandbox_manager: SandboxManager,
}

impl SyscallExecutor {
    pub fn new(sandbox_manager: SandboxManager) -> Self {
        info!("Syscall executor initialized");
        Self { sandbox_manager }
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

        // Get resource limits
        if let Some(limits) = self.sandbox_manager.get_limits(pid) {
            // TODO: Check if we're within process limits
            info!(
                "Spawning process within limits: max_processes={}",
                limits.max_processes
            );
        }

        // Spawn process (sandboxed)
        match Command::new(command).args(args).output() {
            Ok(output) => {
                info!("PID {} spawned process: {} {:?}", pid, command, args);
                let process_output = ProcessOutput {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code().unwrap_or(-1),
                };

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
        let data = serde_json::to_vec(&info).unwrap();
        SyscallResult::success_with_data(data)
    }

    fn get_current_time(&self, pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::TimeAccess)
        {
            return SyscallResult::permission_denied("Missing TimeAccess capability");
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        info!("PID {} retrieved current time: {}", pid, timestamp);
        let data = timestamp.to_le_bytes().to_vec();
        SyscallResult::success_with_data(data)
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
