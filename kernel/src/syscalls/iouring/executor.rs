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
                    SyscallCompletionStatus::Error(message.to_string())
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
        use crate::core::optimization::prefetch_read;

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
        let len = entries.len();
        let futures: Vec<_> = entries
            .into_iter()
            .enumerate()
            .map(|(i, entry)| {
                if i + 2 < len {
                    prefetch_read(&entry as *const _);
                }

                let ring = ring.clone();
                async move {
                    let result = self.execute_operation(&entry.op, entry.pid).await;

                    let status = match &result {
                        crate::syscalls::types::SyscallResult::Success { .. } => {
                            SyscallCompletionStatus::Success
                        }
                        crate::syscalls::types::SyscallResult::Error { message } => {
                            SyscallCompletionStatus::Error(message.to_string())
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
    ///
    /// Executes syscalls directly without spawn_blocking for better performance.
    /// io_uring already handles async I/O, so we don't need additional threading.
    async fn execute_operation(
        &self,
        op: &SyscallOpType,
        pid: crate::core::types::Pid,
    ) -> crate::syscalls::types::SyscallResult {
        // Direct execution - io_uring handles the async I/O
        // No spawn_blocking needed since this is already off the main thread
        match op {
            // File I/O operations
            SyscallOpType::ReadFile { path } => self.syscall_executor.read_file(pid, path),
            SyscallOpType::WriteFile { path, data } => {
                self.syscall_executor.write_file(pid, path, data)
            }
            SyscallOpType::Open { path, flags, mode } => {
                self.syscall_executor.open(pid, path, *flags, *mode)
            }
            SyscallOpType::Close { fd } => self.syscall_executor.close_fd(pid, *fd),
            SyscallOpType::Fsync { fd } => self.syscall_executor.fsync_fd(pid, *fd),
            SyscallOpType::Lseek { fd, offset, whence } => {
                self.syscall_executor.lseek(pid, *fd, *offset, *whence)
            }

            // Network I/O operations
            SyscallOpType::Send {
                sockfd,
                data,
                flags,
            } => self.syscall_executor.send(pid, *sockfd, data, *flags),
            SyscallOpType::Recv {
                sockfd,
                size,
                flags,
            } => self.syscall_executor.recv(pid, *sockfd, *size, *flags),
            SyscallOpType::Accept { sockfd } => self.syscall_executor.accept(pid, *sockfd),
            SyscallOpType::Connect { sockfd, address } => {
                self.syscall_executor.connect(pid, *sockfd, address)
            }
            SyscallOpType::SendTo {
                sockfd,
                data,
                address,
                flags,
            } => self
                .syscall_executor
                .sendto(pid, *sockfd, data, address, *flags),
            SyscallOpType::RecvFrom {
                sockfd,
                size,
                flags,
            } => self.syscall_executor.recvfrom(pid, *sockfd, *size, *flags),

            // IPC operations (using queues)
            SyscallOpType::IpcSend {
                target_pid: _,
                data: _,
            } => {
                // IPC via SendQueue - would need a queue_id mapping
                // For now, return error indicating need for explicit queue usage
                crate::syscalls::types::SyscallResult::Error {
                    message: "Direct IPC send not supported via io_uring, use SendQueue instead"
                        .into(),
                }
            }
            SyscallOpType::IpcRecv { size: _ } => {
                // IPC via ReceiveQueue - would need a queue_id mapping
                // For now, return error indicating need for explicit queue usage
                crate::syscalls::types::SyscallResult::Error {
                    message: "Direct IPC recv not supported via io_uring, use ReceiveQueue instead"
                        .into(),
                }
            }
        }
    }
}
