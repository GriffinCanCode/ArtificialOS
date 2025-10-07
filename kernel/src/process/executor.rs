/*!
 * Process Executor
 * Handles OS-level process spawning and management
 */

use super::types::{ExecutionConfig, ProcessError, ProcessResult};
use crate::core::types::Pid;
use dashmap::DashMap;
use log::{error, info, warn};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

/// Represents an executing OS process
#[derive(Debug)]
pub struct ExecutingProcess {
    pub pid: Pid,    // Internal PID
    pub os_pid: Pid, // OS-level PID
    pub name: String,
    pub command: String,
    pub child: Child, // Process handle
}

/// Manages OS process execution
pub struct ProcessExecutor {
    processes: Arc<DashMap<u32, ExecutingProcess>>,
}

impl ProcessExecutor {
    pub fn new() -> Self {
        info!("Process executor initialized");
        Self {
            processes: Arc::new(DashMap::new()),
        }
    }

    /// Spawn a new OS process
    pub fn spawn(&self, pid: Pid, name: String, config: ExecutionConfig) -> ProcessResult<u32> {
        // Validate command
        self.validate_command(&config.command)?;

        // Build command
        let mut cmd = Command::new(&config.command);

        // Add arguments
        if !config.args.is_empty() {
            cmd.args(&config.args);
        }

        // Set environment (start with clean slate for security)
        cmd.env_clear();
        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }

        // Set working directory
        if let Some(ref dir) = config.working_dir {
            cmd.current_dir(dir);
        }

        // Configure I/O
        if config.capture_output {
            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
        } else {
            cmd.stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
        }

        // Spawn process
        let child = cmd
            .spawn()
            .map_err(|e| ProcessError::SpawnFailed(format!("{}: {}", config.command, e)))?;

        let os_pid = child.id();

        info!(
            "Spawned OS process: '{}' (PID: {}, OS PID: {})",
            name, pid, os_pid
        );

        // Store process
        let process = ExecutingProcess {
            pid,
            os_pid,
            name: name.clone(),
            command: config.command.clone(),
            child,
        };

        self.processes.insert(pid, process);

        Ok(os_pid)
    }

    /// Kill a running process
    pub fn kill(&self, pid: Pid) -> ProcessResult<()> {
        if let Some((_, mut process)) = self.processes.remove(&pid) {
            match process.child.kill() {
                Ok(_) => {
                    info!("Killed OS process PID {} (OS PID: {})", pid, process.os_pid);
                    // Wait for process to fully terminate
                    let _ = process.child.wait();
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to kill process PID {}: {}", pid, e);
                    Err(ProcessError::SpawnFailed(e.to_string()))
                }
            }
        } else {
            Err(ProcessError::ProcessNotFound(pid))
        }
    }

    /// Check if a process is still running
    pub fn is_running(&self, pid: Pid) -> bool {
        self.processes.contains_key(&pid)
    }

    /// Get OS PID for an internal PID
    pub fn get_os_pid(&self, pid: Pid) -> Option<u32> {
        self.processes.get(&pid).map(|p| p.os_pid)
    }

    /// Wait for a process to complete
    pub fn wait(&self, pid: Pid) -> ProcessResult<i32> {
        if let Some((_, mut process)) = self.processes.remove(&pid) {
            match process.child.wait() {
                Ok(status) => {
                    let code = status.code().unwrap_or(-1);
                    info!(
                        "Process '{}' (PID: {}) exited with code {}",
                        process.name, pid, code
                    );
                    Ok(code)
                }
                Err(e) => {
                    error!("Failed to wait for process PID {}: {}", pid, e);
                    Err(ProcessError::SpawnFailed(e.to_string()))
                }
            }
        } else {
            Err(ProcessError::ProcessNotFound(pid))
        }
    }

    /// Get count of running processes
    pub fn count(&self) -> usize {
        self.processes.len()
    }

    /// Validate command for security
    fn validate_command(&self, command: &str) -> ProcessResult<()> {
        // Empty command
        if command.trim().is_empty() {
            return Err(ProcessError::InvalidCommand("Empty command".to_string()));
        }

        // Shell injection prevention
        let dangerous_chars = [';', '|', '&', '\n', '\r', '\0', '`', '$', '(', ')'];
        if dangerous_chars.iter().any(|&c| command.contains(c)) {
            return Err(ProcessError::PermissionDenied(
                "Command contains dangerous characters".to_string(),
            ));
        }

        // Command traversal prevention
        if command.contains("..") {
            return Err(ProcessError::PermissionDenied(
                "Command contains path traversal".to_string(),
            ));
        }

        Ok(())
    }

    /// Cleanup zombie processes
    pub fn cleanup(&self) {
        let mut to_remove = Vec::new();

        for mut entry in self.processes.iter_mut() {
            let pid = *entry.key();
            let process = entry.value_mut();

            // Try to check if process is still running
            match process.child.try_wait() {
                Ok(Some(status)) => {
                    info!(
                        "Process PID {} exited with status: {:?}",
                        pid,
                        status.code()
                    );
                    to_remove.push(pid);
                }
                Ok(None) => {
                    // Still running
                }
                Err(e) => {
                    warn!("Error checking process PID {}: {}", pid, e);
                    to_remove.push(pid);
                }
            }
        }

        for pid in to_remove {
            self.processes.remove(&pid);
        }

        let count = self.processes.len();
        if count > 0 {
            info!("Cleanup: {} active processes remain", count);
        }
    }
}

impl Clone for ProcessExecutor {
    fn clone(&self) -> Self {
        Self {
            processes: Arc::clone(&self.processes),
        }
    }
}

impl Default for ProcessExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_simple_command() {
        let executor = ProcessExecutor::new();
        let config = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);

        let result = executor.spawn(1, "test-sleep".to_string(), config);
        assert!(result.is_ok());

        let os_pid = result.unwrap();
        assert!(os_pid > 0);

        // Cleanup
        executor.kill(1).ok();
    }

    #[test]
    fn test_invalid_command() {
        let executor = ProcessExecutor::new();
        let config = ExecutionConfig::new("echo; rm -rf /".to_string());

        let result = executor.spawn(1, "test-evil".to_string(), config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProcessError::PermissionDenied(_)
        ));
    }

    #[test]
    fn test_kill_process() {
        let executor = ProcessExecutor::new();
        let config = ExecutionConfig::new("sleep".to_string()).with_args(vec!["10".to_string()]);

        executor.spawn(1, "test-sleep".to_string(), config).unwrap();
        assert!(executor.is_running(1));

        let result = executor.kill(1);
        assert!(result.is_ok());
        assert!(!executor.is_running(1));
    }

    #[test]
    fn test_get_os_pid() {
        let executor = ProcessExecutor::new();
        let config = ExecutionConfig::new("sleep".to_string()).with_args(vec!["0.1".to_string()]);

        let os_pid = executor.spawn(1, "test-sleep".to_string(), config).unwrap();
        assert_eq!(executor.get_os_pid(1), Some(os_pid));

        executor.kill(1).ok();
    }
}
