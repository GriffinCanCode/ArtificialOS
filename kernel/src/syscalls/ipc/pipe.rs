/*!
 * Pipe Syscall Operations
 * Handle pipe creation, read, write, and lifecycle
 */

use crate::core::types::Pid;
use crate::core::{bincode, json};
use crate::{IpcGuard, IpcResourceType, SyscallGuard};
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};
use crate::syscalls::executor::SyscallExecutor;
use crate::syscalls::types::SyscallResult;
use log::{error, info};

impl SyscallExecutor {
    pub(crate) fn create_pipe(
        &self,
        pid: Pid,
        reader_pid: Pid,
        writer_pid: Pid,
        capacity: Option<usize>,
    ) -> SyscallResult {
        let mut syscall_guard = match &self.collector {
            Some(collector) => Some(SyscallGuard::new("pipe_create", pid, collector.clone())),
            None => None,
        };

        // Check permission using centralized manager
        let request =
            PermissionRequest::new(pid, Resource::IpcChannel { channel_id: 0 }, Action::Create);
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            let err = SyscallResult::permission_denied(response.reason());
            if let Some(ref mut g) = syscall_guard {
                let result: Result<(), _> = Err(response.reason());
                g.record_result(&result);
            }
            return err;
        }

        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => {
                let err = SyscallResult::error("Pipe manager not available");
                if let Some(ref mut g) = syscall_guard {
                    let result: Result<(), _> = Err("Pipe manager not available");
                    g.record_result(&result);
                }
                return err;
            }
        };

        let result = match pipe_manager.create(reader_pid, writer_pid, capacity) {
            Ok(pipe_id) => {
                info!("PID {} created pipe {}", pid, pipe_id);

                // Create IPC guard for automatic tracking and cleanup
                let _ipc_guard = match &self.collector {
                    Some(collector) => Some(IpcGuard::new(
                        pipe_id.into(),
                        IpcResourceType::Pipe,
                        pid,
                        |_| Ok(()), // No-op cleanup - pipe lifecycle managed separately
                        Some(collector.clone()),
                    )),
                    None => None,
                };

                match json::to_vec(&pipe_id) {
                    Ok(data) => Ok(SyscallResult::success_with_data(data)),
                    Err(e) => {
                        error!("Failed to serialize pipe ID: {}", e);
                        Err("Serialization failed".to_string())
                    }
                }
            }
            Err(e) => {
                error!("Failed to create pipe: {}", e);
                Err(format!("Pipe creation failed: {}", e))
            }
        };

        // Record result in syscall guard
        if let Some(ref mut g) = syscall_guard {
            g.record_result(&result);
        }

        result.unwrap_or_else(|e| SyscallResult::error(e))
    }

    pub(crate) fn write_pipe(&self, pid: Pid, pipe_id: u32, data: &[u8]) -> SyscallResult {
        let mut syscall_guard = match &self.collector {
            Some(collector) => Some(SyscallGuard::new("pipe_write", pid, collector.clone())),
            None => None,
        };

        let request = PermissionRequest::new(
            pid,
            Resource::IpcChannel {
                channel_id: pipe_id,
            },
            Action::Send,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            let err = SyscallResult::permission_denied(response.reason());
            if let Some(ref mut g) = syscall_guard {
                let result: Result<(), _> = Err(response.reason());
                g.record_result(&result);
            }
            return err;
        }

        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => {
                let err = SyscallResult::error("Pipe manager not available");
                if let Some(ref mut g) = syscall_guard {
                    let result: Result<(), _> = Err("Pipe manager not available");
                    g.record_result(&result);
                }
                return err;
            }
        };

        // Use generic timeout executor for all blocking operations
        use crate::ipc::pipe::PipeError;
        let result = self.timeout_executor.execute_with_retry(
            || pipe_manager.write(pipe_id, pid, data),
            |e| matches!(e, PipeError::WouldBlock(_)),
            self.timeout_config.pipe_write,
            "pipe_write",
        );

        let result = match result {
            Ok(written) => {
                info!("PID {} wrote {} bytes to pipe {}", pid, written, pipe_id);
                match json::to_vec(&written) {
                    Ok(data) => Ok(SyscallResult::success_with_data(data)),
                    Err(e) => {
                        error!("Failed to serialize write result: {}", e);
                        Err("Serialization failed".to_string())
                    }
                }
            }
            Err(super::super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("Pipe write timed out for PID {}, pipe {} after {}ms", pid, pipe_id, elapsed_ms);
                Err("Pipe write timed out".to_string())
            }
            Err(super::super::TimeoutError::Operation(e)) => {
                error!("Pipe write failed: {}", e);
                Err(format!("Pipe write failed: {}", e))
            }
        };

        // Record result in syscall guard
        if let Some(ref mut g) = syscall_guard {
            g.record_result(&result);
        }

        result.unwrap_or_else(|e| SyscallResult::error(e))
    }

    pub(crate) fn read_pipe(&self, pid: Pid, pipe_id: u32, size: usize) -> SyscallResult {
        let mut syscall_guard = match &self.collector {
            Some(collector) => Some(SyscallGuard::new("pipe_read", pid, collector.clone())),
            None => None,
        };

        let request = PermissionRequest::new(
            pid,
            Resource::IpcChannel {
                channel_id: pipe_id,
            },
            Action::Receive,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            let err = SyscallResult::permission_denied(response.reason());
            if let Some(ref mut g) = syscall_guard {
                let result: Result<(), _> = Err(response.reason());
                g.record_result(&result);
            }
            return err;
        }

        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => {
                let err = SyscallResult::error("Pipe manager not available");
                if let Some(ref mut g) = syscall_guard {
                    let result: Result<(), _> = Err("Pipe manager not available");
                    g.record_result(&result);
                }
                return err;
            }
        };

        // Use generic timeout executor for all blocking operations
        use crate::ipc::pipe::PipeError;
        let result = self.timeout_executor.execute_with_retry(
            || pipe_manager.read(pipe_id, pid, size),
            |e| matches!(e, PipeError::WouldBlock(_)),
            self.timeout_config.pipe_read,
            "pipe_read",
        );

        let result = match result {
            Ok(data) => {
                info!("PID {} read {} bytes from pipe {}", pid, data.len(), pipe_id);
                Ok(SyscallResult::success_with_data(data))
            }
            Err(super::super::TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!("Pipe read timed out for PID {}, pipe {} after {}ms", pid, pipe_id, elapsed_ms);
                Err("Pipe read timed out".to_string())
            }
            Err(super::super::TimeoutError::Operation(e)) => {
                error!("Pipe read failed: {}", e);
                Err(format!("Pipe read failed: {}", e))
            }
        };

        // Record result in syscall guard
        if let Some(ref mut g) = syscall_guard {
            g.record_result(&result);
        }

        result.unwrap_or_else(|e| SyscallResult::error(e))
    }

    pub(crate) fn close_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult {
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

    pub(crate) fn destroy_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult {
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

    pub(crate) fn pipe_stats(&self, pid: Pid, pipe_id: u32) -> SyscallResult {
        let pipe_manager = match &self.pipe_manager {
            Some(pm) => pm,
            None => return SyscallResult::error("Pipe manager not available"),
        };

        match pipe_manager.stats(pipe_id) {
            Ok(stats) => match bincode::to_vec(&stats) {
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
}
