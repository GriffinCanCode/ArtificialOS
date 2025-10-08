/*!
 * Search Syscall Handler
 * Handles all search-related syscalls
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for search syscalls
pub struct SearchHandler {
    executor: SyscallExecutorWithIpc,
}

impl SearchHandler {
    #[inline]
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for SearchHandler {
    fn name(&self) -> &'static str {
        "search"
    }

    #[inline]
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::SearchFiles {
                ref path,
                ref query,
                limit,
                recursive,
                case_sensitive,
                threshold,
            } => Some(self.executor.search_files(
                pid,
                path,
                query,
                *limit,
                *recursive,
                *case_sensitive,
                *threshold,
            )),
            Syscall::SearchContent {
                ref path,
                ref query,
                limit,
                recursive,
                case_sensitive,
                include_path,
            } => Some(self.executor.search_content(
                pid,
                path,
                query,
                *limit,
                *recursive,
                *case_sensitive,
                *include_path,
            )),
            _ => None,
        }
    }
}

