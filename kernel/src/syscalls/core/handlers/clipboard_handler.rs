/*!
 * Clipboard Syscall Handler
 * Handles clipboard operations
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for clipboard syscalls
pub struct ClipboardHandler {
    executor: SyscallExecutorWithIpc,
}

impl ClipboardHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for ClipboardHandler {
    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::ClipboardCopy {
                ref data,
                ref format,
                global,
            } => Some(self.executor.clipboard_copy(pid, data, format, *global)),
            Syscall::ClipboardPaste { global } => {
                Some(self.executor.clipboard_paste(pid, *global))
            }
            Syscall::ClipboardHistory { global, limit } => {
                Some(self.executor.clipboard_history(pid, *global, *limit))
            }
            Syscall::ClipboardGetEntry { entry_id } => {
                Some(self.executor.clipboard_get_entry(pid, *entry_id))
            }
            Syscall::ClipboardClear { global } => {
                Some(self.executor.clipboard_clear(pid, *global))
            }
            Syscall::ClipboardSubscribe { ref formats } => {
                Some(self.executor.clipboard_subscribe(pid, formats.clone()))
            }
            Syscall::ClipboardUnsubscribe => Some(self.executor.clipboard_unsubscribe(pid)),
            Syscall::ClipboardStats => Some(self.executor.clipboard_stats(pid)),
            _ => None, // Not a clipboard syscall
        }
    }

    #[inline]
    fn name(&self) -> &'static str {
        "clipboard_handler"
    }
}

