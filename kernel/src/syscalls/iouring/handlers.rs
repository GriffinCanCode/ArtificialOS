/*!
 * io_uring Handler Integration
 * Integrates io_uring-style completion with existing syscall handlers
 */

use super::{IoUringManager, SyscallOpType, SyscallSubmissionEntry};
use crate::core::types::Pid;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};
use std::sync::Arc;

/// Handler that routes I/O-bound syscalls to io_uring-style completion
///
/// This handler identifies syscalls that benefit from async completion
/// and routes them through the io_uring manager. Other syscalls are
/// passed through to the next handler.
pub struct IoUringHandler {
    manager: Arc<IoUringManager>,
    /// Whether to use blocking wait or return immediately with task ID
    blocking_mode: bool,
}

impl IoUringHandler {
    /// Create a new io_uring handler
    pub fn new(manager: Arc<IoUringManager>, blocking_mode: bool) -> Self {
        Self {
            manager,
            blocking_mode,
        }
    }

    /// Try to convert a syscall to an io_uring operation
    fn try_convert_to_iouring(&self, syscall: &Syscall) -> Option<SyscallOpType> {
        match syscall {
            // File I/O
            Syscall::ReadFile { path } => Some(SyscallOpType::ReadFile { path: path.clone() }),
            Syscall::WriteFile { path, data } => Some(SyscallOpType::WriteFile {
                path: path.clone(),
                data: data.clone(),
            }),
            Syscall::Open { path, flags, mode } => Some(SyscallOpType::Open {
                path: path.clone(),
                flags: *flags,
                mode: *mode,
            }),
            Syscall::Close { fd } => Some(SyscallOpType::Close { fd: *fd }),

            // Network I/O
            Syscall::Send {
                sockfd,
                data,
                flags,
            } => Some(SyscallOpType::Send {
                sockfd: *sockfd,
                data: data.clone(),
                flags: *flags,
            }),
            Syscall::Recv {
                sockfd,
                size,
                flags,
            } => Some(SyscallOpType::Recv {
                sockfd: *sockfd,
                size: *size,
                flags: *flags,
            }),
            Syscall::Accept { sockfd } => Some(SyscallOpType::Accept { sockfd: *sockfd }),
            Syscall::Connect { sockfd, address } => Some(SyscallOpType::Connect {
                sockfd: *sockfd,
                address: address.clone(),
            }),
            Syscall::SendTo {
                sockfd,
                data,
                address,
                flags,
            } => Some(SyscallOpType::SendTo {
                sockfd: *sockfd,
                data: data.clone(),
                address: address.clone(),
                flags: *flags,
            }),
            Syscall::RecvFrom {
                sockfd,
                size,
                flags,
            } => Some(SyscallOpType::RecvFrom {
                sockfd: *sockfd,
                size: *size,
                flags: *flags,
            }),

            // IPC - Note: SendQueue/ReceiveQueue would need queue_id, so not included here
            // Direct IPC messaging syscalls don't exist in current implementation

            // Not an I/O-bound operation
            _ => None,
        }
    }
}

impl SyscallHandler for IoUringHandler {
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        // Try to convert to io_uring operation
        let op = match self.try_convert_to_iouring(syscall) {
            Some(op) => op,
            None => return None, // Not an I/O operation, pass to next handler
        };

        // Check if this operation should use io_uring
        if !op.is_io_bound() {
            return None;
        }

        // Create submission entry
        let entry = SyscallSubmissionEntry::new(pid, op, 0);

        // Submit to io_uring
        let seq = match self.manager.submit(pid, entry) {
            Ok(seq) => seq,
            Err(e) => {
                return Some(SyscallResult::Error {
                    message: format!("io_uring submission failed: {}", e),
                });
            }
        };

        // If blocking mode, wait for completion
        if self.blocking_mode {
            match self.manager.wait_completion(pid, seq) {
                Ok(completion) => Some(completion.result),
                Err(e) => Some(SyscallResult::Error {
                    message: format!("io_uring completion failed: {}", e),
                }),
            }
        } else {
            // Non-blocking: return task ID for polling
            Some(SyscallResult::Success {
                data: Some(format!("io_uring_seq_{}", seq).into_bytes().into()),
            })
        }
    }

    fn name(&self) -> &'static str {
        "io_uring_handler"
    }
}

/// Async handler that uses io_uring for async operations
///
/// This can be used with the AsyncSyscallHandlerRegistry for true async
/// completion without blocking.
pub struct IoUringAsyncHandler {
    #[allow(dead_code)]
    manager: Arc<IoUringManager>,
}

impl IoUringAsyncHandler {
    /// Create a new async io_uring handler
    pub fn new(manager: Arc<IoUringManager>) -> Self {
        Self { manager }
    }
}

impl crate::syscalls::core::handlers::AsyncSyscallHandler for IoUringAsyncHandler {
    fn handle_async(
        &self,
        pid: Pid,
        syscall: &Syscall,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<SyscallResult>> + Send + '_>>
    {
        // Clone syscall to move into async block
        let syscall = syscall.clone();
        let manager = self.manager.clone();

        Box::pin(async move {
            // Create an io_uring handler in blocking mode
            let handler = IoUringHandler::new(manager, true);

            // Handle the syscall (this will submit and wait)
            handler.handle(pid, &syscall)
        })
    }

    fn name(&self) -> &'static str {
        "io_uring_async_handler"
    }
}
