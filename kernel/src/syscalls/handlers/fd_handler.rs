/*!
 * File Descriptor Syscall Handler
 * Handles file descriptor operations
 */

use crate::core::types::Pid;
use crate::syscalls::executor::SyscallExecutorWithIpc;
use crate::syscalls::handler::SyscallHandler;
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
            } => Some(self.executor.open(pid, path, *flags, *mode)),
            Syscall::Close { fd } => Some(self.executor.close_fd(pid, *fd)),
            Syscall::Dup { fd } => Some(self.executor.dup(pid, *fd)),
            Syscall::Dup2 { oldfd, newfd } => Some(self.executor.dup2(pid, *oldfd, *newfd)),
            Syscall::Lseek { fd, offset, whence } => {
                Some(self.executor.lseek(pid, *fd, *offset, *whence))
            }
            Syscall::Fcntl { fd, cmd, arg } => Some(self.executor.fcntl(pid, *fd, *cmd, *arg)),
            _ => None, // Not an fd syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "fd_handler"
    }
}
