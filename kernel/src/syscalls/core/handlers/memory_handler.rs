/*!
 * Memory Syscall Handler
 * Handles memory management syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for memory syscalls
pub struct MemoryHandler {
    executor: SyscallExecutorWithIpc,
}

impl MemoryHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for MemoryHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::GetMemoryStats => Some(self.executor.get_memory_stats(pid).into()),
            Syscall::GetProcessMemoryStats { target_pid } => {
                Some(self.executor.get_process_memory_stats(pid, *target_pid))
            }
            Syscall::TriggerGC { target_pid } => Some(self.executor.trigger_gc(pid, *target_pid).into()),
            _ => None, // Not a memory syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "memory_handler"
    }
}
