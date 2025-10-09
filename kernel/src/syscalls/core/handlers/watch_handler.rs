/*!
 * Watch Handler - File system event subscriptions
 */

use crate::core::types::Pid;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::core::handler::SyscallHandler;
use crate::syscalls::types::{Syscall, SyscallResult};

/// Handler for file watching syscalls
pub struct WatchHandler {
    executor: SyscallExecutorWithIpc,
}

impl WatchHandler {
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }
}

impl SyscallHandler for WatchHandler {
    fn handle(&self, pid: Pid, syscall: &Syscall) -> Option<SyscallResult> {
        match syscall {
            Syscall::WatchFiles { pattern } => {
                Some(tokio::runtime::Handle::current().block_on(
                    self.executor.watch_files(pid, pattern.clone())
                ))
            }
            Syscall::UnwatchFiles { watch_id } => {
                Some(tokio::runtime::Handle::current().block_on(
                    self.executor.unwatch_files(pid, watch_id.clone())
                ))
            }
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        "watch"
    }
}

