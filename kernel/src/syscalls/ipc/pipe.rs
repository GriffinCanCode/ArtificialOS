/*!
 * Pipe Syscall Operations
 * Handle pipe creation, read, write, and lifecycle
 */

use crate::core::serialization::{bincode, json};
use crate::core::types::Pid;
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::timeout::executor::TimeoutError;
use crate::syscalls::types::SyscallResult;
use log::{error, info};

impl SyscallExecutorWithIpc {
    pub(in crate::syscalls) fn create_pipe(
        &self,
        pid: Pid,
        reader_pid: Pid,
        writer_pid: Pid,
        capacity: Option<usize>,
    ) -> SyscallResult {
        // NOTE: SyscallGuard not needed - executor already provides syscall tracing
        // IpcGuard not needed here - pipe lifecycle managed by PipeManager

        // Check permission using centralized manager
        let request =
            PermissionRequest::new(pid, Resource::IpcChannel { channel_id: 0 }, Action::Create);
        let response = self.permission_manager().check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // Direct access - no Option check! Guaranteed by typestate
        let pipe_manager = &self.ipc().pipe_manager();

        match pipe_manager.create(reader_pid, writer_pid, capacity) {
            Ok(pipe_id) => {
                info!("PID {} created pipe {}", pid, pipe_id);

                match json::to_vec(&pipe_id) {
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

    pub(in crate::syscalls) fn write_pipe(
        &self,
        pid: Pid,
        pipe_id: u32,
        data: &[u8],
    ) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::IpcChannel {
                channel_id: pipe_id,
            },
            Action::Send,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // Direct access - no Option check!
        let pipe_manager = &self.ipc().pipe_manager();

        // Use generic timeout executor for all blocking operations
        use crate::ipc::pipe::PipeError;
        let result = self.timeout_executor().execute_with_retry(
            || pipe_manager.write(pipe_id, pid, data),
            |e| matches!(e, PipeError::WouldBlock(_)),
            self.timeout_config().pipe_write,
            "pipe_write",
        );

        match result {
            Ok(written) => {
                info!("PID {} wrote {} bytes to pipe {}", pid, written, pipe_id);
                match json::to_vec(&written) {
                    Ok(data) => SyscallResult::success_with_data(data),
                    Err(e) => {
                        error!("Failed to serialize write result: {}", e);
                        SyscallResult::error("Serialization failed")
                    }
                }
            }
            Err(TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!(
                    "Pipe write timed out for PID {}, pipe {} after {}ms",
                    pid, pipe_id, elapsed_ms
                );
                SyscallResult::error("Pipe write timed out")
            }
            Err(TimeoutError::Operation(e)) => {
                error!("Pipe write failed: {}", e);
                SyscallResult::error(format!("Pipe write failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn read_pipe(
        &self,
        pid: Pid,
        pipe_id: u32,
        size: usize,
    ) -> SyscallResult {
        let request = PermissionRequest::new(
            pid,
            Resource::IpcChannel {
                channel_id: pipe_id,
            },
            Action::Receive,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // Direct access - no Option check!
        let pipe_manager = &self.ipc().pipe_manager();

        // Use generic timeout executor for all blocking operations
        use crate::ipc::pipe::PipeError;
        let result = self.timeout_executor().execute_with_retry(
            || pipe_manager.read(pipe_id, pid, size),
            |e| matches!(e, PipeError::WouldBlock(_)),
            self.timeout_config().pipe_read,
            "pipe_read",
        );

        match result {
            Ok(data) => {
                info!(
                    "PID {} read {} bytes from pipe {}",
                    pid,
                    data.len(),
                    pipe_id
                );
                SyscallResult::success_with_data(data)
            }
            Err(TimeoutError::Timeout { elapsed_ms, .. }) => {
                error!(
                    "Pipe read timed out for PID {}, pipe {} after {}ms",
                    pid, pipe_id, elapsed_ms
                );
                SyscallResult::error("Pipe read timed out")
            }
            Err(TimeoutError::Operation(e)) => {
                error!("Pipe read failed: {}", e);
                SyscallResult::error(format!("Pipe read failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn close_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult {
        // Direct access - no Option check!
        let pipe_manager = &self.ipc().pipe_manager();

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

    pub(in crate::syscalls) fn destroy_pipe(&self, pid: Pid, pipe_id: u32) -> SyscallResult {
        // Direct access - no Option check!
        let pipe_manager = &self.ipc().pipe_manager();

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

    pub(in crate::syscalls) fn pipe_stats(&self, pid: Pid, pipe_id: u32) -> SyscallResult {
        // Direct access - no Option check!
        let pipe_manager = &self.ipc().pipe_manager();

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
