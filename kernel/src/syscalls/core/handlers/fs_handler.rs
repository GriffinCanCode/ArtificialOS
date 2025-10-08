/*!
 * File System Syscall Handler
 * Handles all file system related syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for file system syscalls
pub struct FileSystemHandler {
    executor: SyscallExecutorWithIpc,
}

impl FileSystemHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for FileSystemHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::ReadFile { ref path } => Some(self.executor.read_file(pid, path).into()),
            Syscall::WriteFile { ref path, ref data } => {
                Some(self.executor.write_file(pid, path, data))
            }
            Syscall::CreateFile { ref path } => Some(self.executor.create_file(pid, path).into()),
            Syscall::DeleteFile { ref path } => Some(self.executor.delete_file(pid, path).into()),
            Syscall::ListDirectory { ref path } => Some(self.executor.list_directory(pid, path).into()),
            Syscall::FileExists { ref path } => Some(self.executor.file_exists(pid, path).into()),
            Syscall::FileStat { ref path } => Some(self.executor.file_stat(pid, path).into()),
            Syscall::MoveFile {
                ref source,
                ref destination,
            } => Some(self.executor.move_file(pid, source, destination).into()),
            Syscall::CopyFile {
                ref source,
                ref destination,
            } => Some(self.executor.copy_file(pid, source, destination).into()),
            Syscall::CreateDirectory { ref path } => {
                Some(self.executor.create_directory(pid, path))
            }
            Syscall::RemoveDirectory { ref path } => {
                Some(self.executor.remove_directory(pid, path))
            }
            Syscall::GetWorkingDirectory => Some(self.executor.get_working_directory(pid).into()),
            Syscall::SetWorkingDirectory { ref path } => {
                Some(self.executor.set_working_directory(pid, path))
            }
            Syscall::TruncateFile { ref path, size } => {
                Some(self.executor.truncate_file(pid, path, *size))
            }
            _ => None, // Not a file system syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "fs_handler"
    }
}
