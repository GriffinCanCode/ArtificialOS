/*!
 * Memory Syscall Handler
 * Handles memory management syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::executor::SyscallExecutor;
use crate::syscalls::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for memory syscalls
pub struct MemoryHandler {
    executor: SyscallExecutor,
}

impl MemoryHandler {
    #[inline]
    pub fn new(executor: SyscallExecutor) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for MemoryHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::GetMemoryStats => Some(self.executor.get_memory_stats(pid)),
            Syscall::GetProcessMemoryStats { target_pid } => {
                Some(self.executor.get_process_memory_stats(pid, *target_pid))
            }
            Syscall::TriggerGC { target_pid } => Some(self.executor.trigger_gc(pid, *target_pid)),
            _ => None, // Not a memory syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "memory_handler"
    }
}
