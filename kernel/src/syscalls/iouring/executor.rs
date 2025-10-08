/*!
 * io_uring Executor
 * Executes async syscall operations from submission queues
 */

use super::completion::SyscallCompletionStatus;
use super::ring::SyscallCompletionRing;
use super::submission::SyscallOpType;
use crate::syscalls::SyscallExecutorWithIpc;
use std::sync::Arc;
use tracing::{debug, error};

/// Executor for io_uring-style syscall operations
pub struct IoUringExecutor {
    syscall_executor: SyscallExecutorWithIpc,
}

impl IoUringExecutor {
    /// Create a new executor
    pub fn new(syscall_executor: SyscallExecutorWithIpc) -> Self {
        Self { syscall_executor }
    }

    /// Execute pending operations from a ring (single)
    pub async fn execute_async(&self, ring: Arc<SyscallCompletionRing>) {
        if let Some(entry) = ring.pop_submission() {
            let result = self.execute_operation(&entry.op, entry.pid).await;

            let status = match &result {
                crate::syscalls::types::SyscallResult::Success { .. } => {
                    SyscallCompletionStatus::Success
                }
                crate::syscalls::types::SyscallResult::Error { message } => {
                    SyscallCompletionStatus::Error(message.clone())
                }
                crate::syscalls::types::SyscallResult::PermissionDenied { reason } => {
                    SyscallCompletionStatus::Error(format!("Permission denied: {}", reason))
                }
            };

            ring.complete(entry.seq, status, result, entry.user_data);
        }
    }

    /// Execute pending operations from a ring (batch)
    pub async fn execute_batch_async(&self, ring: Arc<SyscallCompletionRing>) {
        const BATCH_SIZE: usize = crate::core::limits::IOURING_BATCH_SIZE;
        let entries = ring.pop_submissions(BATCH_SIZE);

        if entries.is_empty() {
            return;
        }

        debug!(
            pid = ring.pid(),
            count = entries.len(),
            "Executing batch of io_uring operations"
        );

        // Execute all operations concurrently
        let futures: Vec<_> = entries
            .into_iter()
            .map(|entry| {
                let ring = ring.clone();
                async move {
                    let result = self.execute_operation(&entry.op, entry.pid).await;

                    let status = match &result {
                        crate::syscalls::types::SyscallResult::Success { .. } => {
                            SyscallCompletionStatus::Success
                        }
                        crate::syscalls::types::SyscallResult::Error { message } => {
                            SyscallCompletionStatus::Error(message.clone())
                        }
                        crate::syscalls::types::SyscallResult::PermissionDenied { reason } => {
                            SyscallCompletionStatus::Error(format!("Permission denied: {}", reason))
                        }
                    };

                    ring.complete(entry.seq, status, result, entry.user_data);
                }
            })
            .collect();

        // Wait for all operations to complete
        futures::future::join_all(futures).await;
    }

    /// Execute a single operation
    async fn execute_operation(
        &self,
        op: &SyscallOpType,
        pid: crate::core::types::Pid,
    ) -> crate::syscalls::types::SyscallResult {
        use tokio::task;

        // Most operations are blocking, so run them on blocking thread pool
        let executor = self.syscall_executor.clone();
        let op = op.clone();

        task::spawn_blocking(move || {
            match op {
                // File I/O operations
                SyscallOpType::ReadFile { path } => executor.read_file(pid, &path),
                SyscallOpType::WriteFile { path, data } => executor.write_file(pid, &path, &data),
                SyscallOpType::Open { path, flags, mode } => executor.open(pid, &path, flags, mode),
                SyscallOpType::Close { fd } => executor.close_fd(pid, fd),
                SyscallOpType::Fsync { fd } => executor.fsync_fd(pid, fd),
                SyscallOpType::Lseek { fd, offset, whence } => {
                    executor.lseek(pid, fd, offset, whence)
                }

                // Network I/O operations
                SyscallOpType::Send {
                    sockfd,
                    data,
                    flags,
                } => executor.send(pid, sockfd, &data, flags),
                SyscallOpType::Recv {
                    sockfd,
                    size,
                    flags,
                } => executor.recv(pid, sockfd, size, flags),
                SyscallOpType::Accept { sockfd } => executor.accept(pid, sockfd),
                SyscallOpType::Connect { sockfd, address } => {
                    executor.connect(pid, sockfd, &address)
                }
                SyscallOpType::SendTo {
                    sockfd,
                    data,
                    address,
                    flags,
                } => executor.sendto(pid, sockfd, &data, &address, flags),
                SyscallOpType::RecvFrom {
                    sockfd,
                    size,
                    flags,
                } => executor.recvfrom(pid, sockfd, size, flags),

                // IPC operations (using queues)
                SyscallOpType::IpcSend {
                    target_pid: _,
                    data: _,
                } => {
                    // IPC via SendQueue - would need a queue_id mapping
                    // For now, return error indicating need for explicit queue usage
                    crate::syscalls::types::SyscallResult::Error {
                        message:
                            "Direct IPC send not supported via io_uring, use SendQueue instead"
                                .to_string(),
                    }
                }
                SyscallOpType::IpcRecv { size: _ } => {
                    // IPC via ReceiveQueue - would need a queue_id mapping
                    // For now, return error indicating need for explicit queue usage
                    crate::syscalls::types::SyscallResult::Error {
                        message:
                            "Direct IPC recv not supported via io_uring, use ReceiveQueue instead"
                                .to_string(),
                    }
                }
            }
        })
        .await
        .unwrap_or_else(|e| {
            error!("Task execution panic: {}", e);
            crate::syscalls::types::SyscallResult::Error {
                message: format!("Execution panic: {}", e),
            }
        })
    }
}
