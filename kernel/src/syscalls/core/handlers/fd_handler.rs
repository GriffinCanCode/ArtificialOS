/*!
 * File Descriptor Syscall Handler
 * Handles file descriptor operations
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for file descriptor syscalls
pub struct FileDescriptorHandler {
    executor: SyscallExecutorWithIpc,
}

impl FileDescriptorHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for FileDescriptorHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::Open {
                ref path,
                flags,
                mode,
            } => Some(self.executor.open(pid, path, *flags, *mode).into()),
            Syscall::Close { fd } => Some(self.executor.close_fd(pid, *fd).into()),
            Syscall::Dup { fd } => Some(self.executor.dup(pid, *fd).into()),
            Syscall::Dup2 { oldfd, newfd } => Some(self.executor.dup2(pid, *oldfd, *newfd).into()),
            Syscall::Lseek { fd, offset, whence } => {
                Some(self.executor.lseek(pid, *fd, *offset, *whence))
            }
            Syscall::Fcntl { fd, cmd, arg } => Some(self.executor.fcntl(pid, *fd, *cmd, *arg).into()),
            _ => None, // Not an fd syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "fd_handler"
    }
}
