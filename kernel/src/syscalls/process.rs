/*!
 * Process Syscalls
 * Process management and control
 */

use log::{error, info, warn};
use std::process::Command;

use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::{ProcessOutput, SyscallResult};

impl SyscallExecutor {
    pub(super) fn spawn_process(&self, pid: u32, command: &str, args: &[String]) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SpawnProcess)
        {
            return SyscallResult::permission_denied("Missing SpawnProcess capability");
        }

        if command.is_empty() || command.contains([';', '|', '&', '\n', '\0']) {
            error!("Invalid command attempted: {:?}", command);
            return SyscallResult::error("Invalid command: contains shell metacharacters");
        }

        for arg in args {
            if arg.contains('\0') {
                error!("Invalid argument attempted: contains null byte");
                return SyscallResult::error("Invalid argument: contains null byte");
            }
        }

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

        match Command::new(command).args(args).output() {
            Ok(output) => {
                self.sandbox_manager.record_spawn(pid);

                info!("PID {} spawned process: {} {:?}", pid, command, args);
                let process_output = ProcessOutput {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code().unwrap_or(-1),
                };

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

    pub(super) fn kill_process(&self, pid: u32, target_pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::KillProcess)
        {
            return SyscallResult::permission_denied("Missing KillProcess capability");
        }

        self.sandbox_manager.remove_sandbox(target_pid);

        info!(
            "PID {} terminated PID {} and cleaned up sandbox",
            pid, target_pid
        );
        SyscallResult::success()
    }

    pub(super) fn get_process_info(&self, pid: u32, target_pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_process(target_pid) {
            Some(process) => match serde_json::to_vec(&process) {
                Ok(data) => {
                    info!("PID {} retrieved info for PID {}", pid, target_pid);
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize process info: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            None => SyscallResult::error(format!("Process {} not found", target_pid)),
        }
    }

    pub(super) fn get_process_list(&self, pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        let processes = process_manager.list_processes();
        match serde_json::to_vec(&processes) {
            Ok(data) => {
                info!("PID {} listed {} processes", pid, processes.len());
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Failed to serialize process list: {}", e);
                SyscallResult::error("Serialization failed")
            }
        }
    }

    pub(super) fn set_process_priority(&self, pid: u32, target_pid: u32, priority: u8) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SpawnProcess)
        {
            return SyscallResult::permission_denied("Missing SpawnProcess capability");
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        // Get process, modify priority, update scheduler
        match process_manager.get_process(target_pid) {
            Some(_) => {
                // Priority is part of Process struct, scheduler tracks it
                // In full implementation would update the process struct
                info!("PID {} set priority of PID {} to {}", pid, target_pid, priority);
                warn!("SetProcessPriority not fully implemented - requires mutable process access");
                SyscallResult::success()
            }
            None => SyscallResult::error(format!("Process {} not found", target_pid))
        }
    }

    pub(super) fn get_process_state(&self, pid: u32, target_pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_process(target_pid) {
            Some(process) => match serde_json::to_vec(&process.state) {
                Ok(data) => {
                    info!("PID {} retrieved state for PID {}", pid, target_pid);
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize process state: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            None => SyscallResult::error(format!("Process {} not found", target_pid)),
        }
    }

    pub(super) fn get_process_stats_call(&self, pid: u32, target_pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let process_manager = match &self.process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_process_stats(target_pid) {
            Some(stats) => match serde_json::to_vec(&stats) {
                Ok(data) => {
                    info!("PID {} retrieved stats for PID {}", pid, target_pid);
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize process stats: {}", e);
                    SyscallResult::error("Serialization failed")
                }
            },
            None => SyscallResult::error(format!("No stats available for process {}", target_pid)),
        }
    }

    pub(super) fn wait_process(&self, pid: u32, target_pid: u32, _timeout_ms: Option<u64>) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SpawnProcess)
        {
            return SyscallResult::permission_denied("Missing SpawnProcess capability");
        }

        warn!("WaitProcess not fully implemented, returning immediately");
        info!("PID {} waiting for PID {}", pid, target_pid);
        SyscallResult::success()
    }
}
