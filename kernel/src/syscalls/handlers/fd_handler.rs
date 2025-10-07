/*!
 * File Descriptor Syscall Handler
 * Handles file descriptor operations
 */

use crate::core::types::Pid;
use crate::syscalls::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};
use crate::syscalls::executor::SyscallExecutor;

/// Handler for file descriptor syscalls
pub struct FileDescriptorHandler {
    executor: SyscallExecutor,
}

impl FileDescriptorHandler {
    pub fn new(executor: SyscallExecutor) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for FileDescriptorHandler {
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::Open { ref path, flags, mode } => {
                Some(self.executor.open(pid, path, *flags, *mode))
            }
            Syscall::Close { fd } => {
                Some(self.executor.close_fd(pid, *fd))
            }
            Syscall::Dup { fd } => {
                Some(self.executor.dup(pid, *fd))
            }
            Syscall::Dup2 { oldfd, newfd } => {
                Some(self.executor.dup2(pid, *oldfd, *newfd))
            }
            Syscall::Lseek { fd, offset, whence } => {
                Some(self.executor.lseek(pid, *fd, *offset, *whence))
            }
            Syscall::Fcntl { fd, cmd, arg } => {
                Some(self.executor.fcntl(pid, *fd, *cmd, *arg))
            }
            _ => None, // Not an fd syscall
        }
    }

    fn name(&self) -> &'static str {
        "fd_handler"
    }
}
