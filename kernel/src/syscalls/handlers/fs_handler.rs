/*!
 * File System Syscall Handler
 * Handles all file system related syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};
use crate::syscalls::executor::SyscallExecutor;

/// Handler for file system syscalls
pub struct FileSystemHandler {
    executor: SyscallExecutor,
}

impl FileSystemHandler {
    #[inline]
    pub fn new(executor: SyscallExecutor) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for FileSystemHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::ReadFile { ref path } => {
                Some(self.executor.read_file(pid, path))
            }
            Syscall::WriteFile { ref path, ref data } => {
                Some(self.executor.write_file(pid, path, data))
            }
            Syscall::CreateFile { ref path } => {
                Some(self.executor.create_file(pid, path))
            }
            Syscall::DeleteFile { ref path } => {
                Some(self.executor.delete_file(pid, path))
            }
            Syscall::ListDirectory { ref path } => {
                Some(self.executor.list_directory(pid, path))
            }
            Syscall::FileExists { ref path } => {
                Some(self.executor.file_exists(pid, path))
            }
            Syscall::FileStat { ref path } => {
                Some(self.executor.file_stat(pid, path))
            }
            Syscall::MoveFile { ref source, ref destination } => {
                Some(self.executor.move_file(pid, source, destination))
            }
            Syscall::CopyFile { ref source, ref destination } => {
                Some(self.executor.copy_file(pid, source, destination))
            }
            Syscall::CreateDirectory { ref path } => {
                Some(self.executor.create_directory(pid, path))
            }
            Syscall::RemoveDirectory { ref path } => {
                Some(self.executor.remove_directory(pid, path))
            }
            Syscall::GetWorkingDirectory => {
                Some(self.executor.get_working_directory(pid))
            }
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
