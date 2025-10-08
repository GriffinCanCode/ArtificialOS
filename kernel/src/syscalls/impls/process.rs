/*!

* Process Syscalls
* Process management and control
*/

use crate::syscalls::timeout::executor::TimeoutError;

use crate::core::serialization::json;
use crate::core::types::{Pid, Priority};
use crate::monitoring::span_operation;
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};
use log::{error, info, warn};
use std::process::Command;

use crate::security::{ResourceLimitProvider, SandboxProvider};

use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::types::{ProcessOutput, SyscallResult};

impl SyscallExecutorWithIpc {
    pub(in crate::syscalls) fn spawn_process(
        &self,
        pid: Pid,
        command: &str,
        args: &[String],
    ) -> SyscallResult {
        let span = span_operation("process_spawn");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("command", command);
        span.record("args_count", &format!("{}", args.len()));

        let request = PermissionRequest::new(pid, Resource::Process { pid: 0 }, Action::Create);
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        if command.is_empty() || command.contains([';', '|', '&', '\n', '\0']) {
            error!("Invalid command attempted: {:?}", command);
            span.record_error("Invalid command: contains shell metacharacters");
            return SyscallResult::error("Invalid command: contains shell metacharacters");
        }

        for arg in args {
            if arg.contains('\0') {
                error!("Invalid argument attempted: contains null byte");
                span.record_error("Invalid argument: contains null byte");
                return SyscallResult::error("Invalid argument: contains null byte");
            }
        }

        if let Some(limits) = self.sandbox_manager().get_limits(pid) {
            if !self.sandbox_manager().can_spawn_process(pid) {
                let current = self.sandbox_manager().get_spawn_count(pid);
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
                self.sandbox_manager().record_spawn(pid);

                info!("PID {} spawned process: {} {:?}", pid, command, args);
                let process_output = ProcessOutput {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code().unwrap_or(-1),
                };

                use crate::core::memory::arena::with_arena;

                with_arena(|arena| {
                    self.sandbox_manager().record_termination(pid);
                    let exit_code_str = arena.alloc_str(&process_output.exit_code.to_string());
                    span.record("exit_code", exit_code_str);
                    span.record_result(true);

                    match json::to_vec(&process_output) {
                        Ok(result) => SyscallResult::success_with_data(result),
                        Err(e) => {
                            error!("Failed to serialize process output: {}", e);
                            span.record_error("Failed to serialize process output");
                            SyscallResult::error("Failed to serialize process output")
                        }
                    }
                })
            }
            Err(e) => {
                error!("Failed to spawn process: {}", e);
                span.record_error(&format!("Spawn failed: {}", e));
                SyscallResult::error(format!("Spawn failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn kill_process(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        let span = span_operation("process_kill");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("target_pid", &format!("{}", target_pid));

        let request = PermissionRequest::proc_kill(pid, target_pid);
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        self.sandbox_manager().remove_sandbox(target_pid);

        info!(
            "PID {} terminated PID {} and cleaned up sandbox",
            pid, target_pid
        );
        span.record_result(true);
        SyscallResult::success()
    }

    pub(in crate::syscalls) fn get_process_info(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        let span = span_operation("process_get_info");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("target_pid", &format!("{}", target_pid));

        let request =
            PermissionRequest::new(pid, Resource::Process { pid: target_pid }, Action::Inspect);
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => {
                span.record_error("Process manager not available");
                return SyscallResult::error("Process manager not available");
            }
        };

        match process_manager.get_process(target_pid) {
            Some(process) => match json::to_vec(&process) {
                Ok(data) => {
                    info!("PID {} retrieved info for PID {}", pid, target_pid);
                    span.record_result(true);
                    SyscallResult::success_with_data(data)
                }
                Err(e) => {
                    error!("Failed to serialize process info: {}", e);
                    span.record_error("Serialization failed");
                    SyscallResult::error("Serialization failed")
                }
            },
            None => {
                span.record_error(&format!("Process {} not found", target_pid));
                SyscallResult::error(format!("Process {} not found", target_pid))
            }
        }
    }

    pub(in crate::syscalls) fn get_process_list(&self, pid: Pid) -> SyscallResult {
        let span = span_operation("process_list");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));

        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "processes".to_string(),
            },
            Action::List,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => {
                span.record_error("Process manager not available");
                return SyscallResult::error("Process manager not available");
            }
        };

        let processes = process_manager.list_processes();
        span.record("process_count", &format!("{}", processes.len()));
        match json::to_vec(&processes) {
            Ok(data) => {
                info!("PID {} listed {} processes", pid, processes.len());
                span.record_result(true);
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Failed to serialize process list: {}", e);
                span.record_error("Serialization failed");
                SyscallResult::error("Serialization failed")
            }
        }
    }

    pub(in crate::syscalls) fn set_process_priority(
        &self,
        pid: Pid,
        target_pid: Pid,
        priority: Priority,
    ) -> SyscallResult {
        let request =
            PermissionRequest::new(pid, Resource::Process { pid: target_pid }, Action::Write);
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        // Update process priority in process manager, scheduler, and resource limits
        if process_manager.set_process_priority(target_pid, priority) {
            info!(
                "PID {} successfully set priority of PID {} to {}",
                pid, target_pid, priority
            );
            SyscallResult::success()
        } else {
            SyscallResult::error(format!("Process {} not found", target_pid))
        }
    }

    pub(in crate::syscalls) fn get_process_state(
        &self,
        pid: Pid,
        target_pid: Pid,
    ) -> SyscallResult {
        let request =
            PermissionRequest::new(pid, Resource::Process { pid: target_pid }, Action::Inspect);
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_process(target_pid) {
            Some(process) => match json::to_vec(&process.state) {
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

    pub(in crate::syscalls) fn get_process_stats_call(
        &self,
        pid: Pid,
        target_pid: Pid,
    ) -> SyscallResult {
        let request =
            PermissionRequest::new(pid, Resource::Process { pid: target_pid }, Action::Inspect);
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Process manager not available"),
        };

        match process_manager.get_process_stats(target_pid) {
            Some(stats) => match json::to_vec(&stats) {
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

    pub(in crate::syscalls) fn wait_process(
        &self,
        pid: Pid,
        target_pid: Pid,
        timeout_ms: Option<u64>,
    ) -> SyscallResult {
        let span = span_operation("process_wait");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("target_pid", &format!("{}", target_pid));

        let request =
            PermissionRequest::new(pid, Resource::Process { pid: target_pid }, Action::Inspect);
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        let process_manager = match &self.optional().process_manager {
            Some(pm) => pm,
            None => {
                span.record_error("Process manager not available");
                return SyscallResult::error("Process manager not available");
            }
        };

        // Determine timeout policy: use custom timeout if provided, otherwise use default
        use crate::core::guard::TimeoutPolicy;
        use std::time::Duration;
        let timeout = if let Some(ms) = timeout_ms {
            TimeoutPolicy::Io(Duration::from_millis(ms))
        } else {
            self.timeout_config().process_wait
        };

        span.record(
            "timeout_ms",
            &format!("{:?}", timeout.duration().map(|d| d.as_millis())),
        );

        // Define error type for process wait
        #[derive(Debug)]
        enum WaitError {
            StillRunning,
            NotFound,
        }

        // Use timeout executor to wait for process completion
        let result = self.timeout_executor().execute_with_retry(
            || {
                // Check if process still exists and is running
                match process_manager.get_process(target_pid) {
                    Some(process) => {
                        // Check if process has terminated
                        if process.state == crate::process::types::ProcessState::Terminated {
                            Ok(())
                        } else {
                            // Process still running - retry
                            Err(WaitError::StillRunning)
                        }
                    }
                    None => {
                        // Process not found - either never existed or already cleaned up
                        // Consider this success (process is "done")
                        Ok(())
                    }
                }
            },
            |e| matches!(e, WaitError::StillRunning),
            timeout,
            "process_wait",
        );

        match result {
            Ok(()) => {
                info!(
                    "PID {} successfully waited for PID {} completion",
                    pid, target_pid
                );
                span.record_result(true);
                SyscallResult::success()
            }
            Err(TimeoutError::Timeout { elapsed_ms, .. }) => {
                warn!(
                    "Process wait timed out: PID {} waiting for PID {} after {}ms",
                    pid, target_pid, elapsed_ms
                );
                span.record_error(&format!("Timeout after {}ms", elapsed_ms));
                SyscallResult::error(format!("Process wait timed out after {}ms", elapsed_ms))
            }
            Err(TimeoutError::Operation(WaitError::NotFound)) => {
                span.record_error(&format!("Process {} not found", target_pid));
                SyscallResult::error(format!("Process {} not found", target_pid))
            }
            Err(TimeoutError::Operation(WaitError::StillRunning)) => {
                // This shouldn't happen (retry should handle this)
                span.record_error("Unexpected still running state");
                SyscallResult::error("Process still running")
            }
        }
    }
}
