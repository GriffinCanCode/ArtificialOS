/*!
 * Process Executor
 * Handles OS-level process spawning and management
 */

use super::types::{ExecutionConfig, ProcessError, ProcessResult};
use crate::core::types::Pid;
use crate::security::types::Limits;
use dashmap::DashMap;
use log::{error, info, warn};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

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

        // Validate arguments for path traversal and injection attacks
        for arg in &config.args {
            self.validate_argument(arg)?;
        }

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

        // Apply resource limits atomically via pre-exec hook (Unix only)
        // This ensures limits are in place BEFORE the process runs
        #[cfg(unix)]
        if let Some(ref limits) = config.limits {
            let limits_clone = limits.clone();
            unsafe {
                cmd.pre_exec(move || Self::apply_rlimits_preexec(&limits_clone));
            }
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

        // Path traversal prevention - comprehensive checks
        // Check for direct .. usage
        if command.contains("..") {
            return Err(ProcessError::PermissionDenied(
                "Command contains path traversal".to_string(),
            ));
        }

        // Check for encoded dots and traversal bypass attempts
        let bypass_patterns = [
            "%2e%2e",  // URL encoded ..
            "%252e",   // Double encoded .
            "..%2f",   // Encoded slash variants
            "%2e.",    // Partial encoding
            ".%2e",    // Partial encoding
            "..\\",    // Windows-style path traversal
            "\\..",    // Windows-style path traversal
            "%5c..",   // Encoded backslash
            "..%5c",   // Encoded backslash
            "\u{2024}\u{2024}", // Unicode two dot leader (could be used as lookalike)
        ];

        let command_lower = command.to_lowercase();
        for pattern in &bypass_patterns {
            if command_lower.contains(&pattern.to_lowercase()) {
                return Err(ProcessError::PermissionDenied(
                    "Command contains path traversal attempt".to_string(),
                ));
            }
        }

        // Validate path components by splitting on common delimiters
        // This catches cases where normalization might expose traversal
        for word in command.split_whitespace() {
            // Skip if it's clearly not a path (no slashes)
            if word.contains('/') || word.contains('\\') {
                // Check if normalizing this path component would result in upward traversal
                if Self::contains_path_traversal(word) {
                    return Err(ProcessError::PermissionDenied(
                        "Command contains path traversal pattern".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Apply resource limits in pre-exec hook (Unix only)
    /// This is called AFTER fork() but BEFORE exec(), ensuring limits are atomic
    #[cfg(unix)]
    fn apply_rlimits_preexec(limits: &Limits) -> std::io::Result<()> {
        use nix::libc::{rlimit, setrlimit, RLIMIT_AS, RLIMIT_NPROC, RLIMIT_NOFILE};

        // Memory limit (address space)
        if let Some(memory_bytes) = limits.memory_bytes {
            let rlim = rlimit {
                rlim_cur: memory_bytes,
                rlim_max: memory_bytes,
            };
            unsafe {
                if setrlimit(RLIMIT_AS, &rlim) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
        }

        // CPU time limit (convert from shares to seconds - rough approximation)
        // Note: cpu_shares is for cgroups, not directly mappable to RLIMIT_CPU
        // We skip this for now as it needs different handling via cgroups

        // Process/thread limit
        if let Some(max_pids) = limits.max_pids {
            let rlim = rlimit {
                rlim_cur: max_pids as u64,
                rlim_max: max_pids as u64,
            };
            unsafe {
                if setrlimit(RLIMIT_NPROC, &rlim) != 0 {
                    // Non-fatal, log would go to stderr
                    eprintln!("Warning: Failed to set RLIMIT_NPROC");
                }
            }
        }

        // File descriptor limit
        if let Some(max_files) = limits.max_open_files {
            let rlim = rlimit {
                rlim_cur: max_files as u64,
                rlim_max: max_files as u64,
            };
            unsafe {
                if setrlimit(RLIMIT_NOFILE, &rlim) != 0 {
                    eprintln!("Warning: Failed to set RLIMIT_NOFILE");
                }
            }
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

    /// Validate command argument for security
    fn validate_argument(&self, arg: &str) -> ProcessResult<()> {
        // Check for direct .. usage
        if arg.contains("..") {
            return Err(ProcessError::PermissionDenied(
                "Argument contains path traversal".to_string(),
            ));
        }

        // Check for encoded dots and traversal bypass attempts
        let bypass_patterns = [
            "%2e%2e",  // URL encoded ..
            "%252e",   // Double encoded .
            "..%2f",   // Encoded slash variants
            "%2e.",    // Partial encoding
            ".%2e",    // Partial encoding
            "..\\",    // Windows-style path traversal
            "\\..",    // Windows-style path traversal
            "%5c..",   // Encoded backslash
            "..%5c",   // Encoded backslash
        ];

        let arg_lower = arg.to_lowercase();
        for pattern in &bypass_patterns {
            if arg_lower.contains(&pattern.to_lowercase()) {
                return Err(ProcessError::PermissionDenied(
                    "Argument contains path traversal attempt".to_string(),
                ));
            }
        }

        // Shell injection prevention
        let dangerous_chars = [';', '|', '&', '\n', '\r', '\0', '`', '$'];
        if dangerous_chars.iter().any(|&c| arg.contains(c)) {
            return Err(ProcessError::PermissionDenied(
                "Argument contains shell injection characters".to_string(),
            ));
        }

        // Check for encoded shell metacharacters
        let encoded_dangerous = [
            "%3b",  // ;
            "%7c",  // |
            "%26",  // &
            "%24",  // $
            "%60",  // `
        ];
        for pattern in &encoded_dangerous {
            if arg_lower.contains(pattern) {
                return Err(ProcessError::PermissionDenied(
                    "Argument contains encoded shell metacharacters".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Helper to detect path traversal patterns
    fn contains_path_traversal(path: &str) -> bool {
        // Try to normalize the path and see if it goes upward
        let parts: Vec<&str> = path.split('/').collect();
        let mut depth = 0;

        for part in parts {
            match part {
                ".." => {
                    if depth > 0 {
                        depth -= 1;
                    } else {
                        // Attempting to go above root
                        return true;
                    }
                }
                "." | "" => {
                    // Current dir or empty, no change
                }
                _ => {
                    depth += 1;
                }
            }
        }

        false
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

    #[test]
    fn test_path_traversal_detection() {
        // Test basic .. detection
        assert!(ProcessExecutor::contains_path_traversal("../etc/passwd"));
        assert!(ProcessExecutor::contains_path_traversal("../../etc/passwd"));

        // Test with ./ sequences (the bypass mentioned)
        assert!(ProcessExecutor::contains_path_traversal("./../../etc/passwd"));
        assert!(ProcessExecutor::contains_path_traversal("foo/./bar/../../../etc"));

        // Test absolute paths with traversal
        assert!(ProcessExecutor::contains_path_traversal("/../../../etc/passwd"));

        // Test multiple traversals that escape
        assert!(ProcessExecutor::contains_path_traversal("a/../../../b"));

        // Safe paths - should NOT be detected as traversal
        assert!(!ProcessExecutor::contains_path_traversal("./valid/path.txt"));
        assert!(!ProcessExecutor::contains_path_traversal("./subdir/file.txt"));
        assert!(!ProcessExecutor::contains_path_traversal("dir1/dir2/../file.txt")); // stays within bounds
        assert!(!ProcessExecutor::contains_path_traversal("dir1/dir2/dir3/../../file.txt")); // stays within bounds
        assert!(!ProcessExecutor::contains_path_traversal("/absolute/path/file.txt"));
        assert!(!ProcessExecutor::contains_path_traversal("relative/path/file.txt"));
    }

    #[test]
    fn test_command_validation_path_traversal() {
        let executor = ProcessExecutor::new();

        // Test command with .. in argument
        let config = ExecutionConfig::new("cat".to_string())
            .with_args(vec!["../etc/passwd".to_string()]);
        let result = executor.spawn(1, "test-traversal".to_string(), config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProcessError::PermissionDenied(_)));

        // Test command with /./ and .. combination (the bypass)
        let config = ExecutionConfig::new("cat".to_string())
            .with_args(vec!["./../../etc/passwd".to_string()]);
        let result = executor.spawn(2, "test-bypass".to_string(), config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProcessError::PermissionDenied(_)));

        // Test URL encoded traversal
        let config = ExecutionConfig::new("cat".to_string())
            .with_args(vec!["%2e%2e/etc/passwd".to_string()]);
        let result = executor.spawn(3, "test-encoded".to_string(), config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProcessError::PermissionDenied(_)));

        // Test Windows-style traversal
        let config = ExecutionConfig::new("cat".to_string())
            .with_args(vec!["..\\..\\windows\\system32".to_string()]);
        let result = executor.spawn(4, "test-windows".to_string(), config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProcessError::PermissionDenied(_)));

        // Test safe path - should succeed (or fail for other reasons, but not path validation)
        let config = ExecutionConfig::new("echo".to_string())
            .with_args(vec!["./valid/path.txt".to_string()]);
        let result = executor.spawn(5, "test-safe".to_string(), config);
        // Should not fail due to path validation
        // (might fail if echo doesn't exist, but that's ok)
        if result.is_err() {
            // If it fails, it should NOT be due to permission denied for path traversal
            assert!(!matches!(result.unwrap_err(), ProcessError::PermissionDenied(_)));
        }
    }

    #[test]
    fn test_command_validation_encoding_bypass() {
        let executor = ProcessExecutor::new();

        let bypass_attempts = vec![
            "%2e%2e/etc/passwd",      // URL encoded
            "%252e%252e/etc/passwd",  // Double encoded
            "..%2fetc/passwd",        // Encoded slash
            "%2e./etc/passwd",        // Partial encoding
            ".%2e/etc/passwd",        // Partial encoding
            "..%5cetc",               // Encoded backslash
        ];

        for (i, attempt) in bypass_attempts.iter().enumerate() {
            let config = ExecutionConfig::new("cat".to_string())
                .with_args(vec![attempt.to_string()]);
            let result = executor.spawn(100 + i as u32, format!("bypass-{}", i), config);
            assert!(result.is_err(), "Failed to block bypass attempt: {}", attempt);
            assert!(
                matches!(result.unwrap_err(), ProcessError::PermissionDenied(_)),
                "Wrong error type for bypass attempt: {}", attempt
            );
        }
    }
}
