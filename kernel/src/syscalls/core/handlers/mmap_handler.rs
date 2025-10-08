/*!
 * Memory-Mapped File Syscall Handler
 * Handles memory-mapped file operations
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for mmap syscalls
pub struct MmapHandler {
    executor: SyscallExecutorWithIpc,
}

impl MmapHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for MmapHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::Mmap {
                ref path,
                offset,
                length,
                prot,
                shared,
            } => Some(
                self.executor
                    .mmap(pid, path, *offset, *length, *prot, *shared),
            ),
            Syscall::MmapRead {
                mmap_id,
                offset,
                length,
            } => Some(self.executor.mmap_read(pid, *mmap_id, *offset, *length).into()),
            Syscall::MmapWrite {
                mmap_id,
                offset,
                ref data,
            } => Some(self.executor.mmap_write(pid, *mmap_id, *offset, data).into()),
            Syscall::Msync { mmap_id } => Some(self.executor.msync(pid, *mmap_id).into()),
            Syscall::Munmap { mmap_id } => Some(self.executor.munmap(pid, *mmap_id).into()),
            Syscall::MmapStats { mmap_id } => Some(self.executor.mmap_stats(pid, *mmap_id).into()),
            _ => None, // Not an mmap syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "mmap_handler"
    }
}
