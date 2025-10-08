/*!
 * Batch Syscall Executor
 * Executes multiple syscalls efficiently in parallel or sequence
 */

use crate::core::types::Pid;
use crate::syscalls::{Syscall, SyscallExecutorWithIpc, SyscallResult};
use futures::future::join_all;

#[derive(Clone)]
pub struct BatchExecutor {
    executor: SyscallExecutorWithIpc,
}

impl BatchExecutor {
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self { executor }
    }

    pub async fn execute_batch(
        &self,
        requests: Vec<(Pid, Syscall)>,
        parallel: bool,
    ) -> Vec<SyscallResult> {
        if parallel {
            self.execute_parallel(requests).await
        } else {
            self.execute_sequential(requests).await
        }
    }

    async fn execute_parallel(&self, requests: Vec<(Pid, Syscall)>) -> Vec<SyscallResult> {
        let count = requests.len();
        let futures: Vec<_> = requests
            .into_iter()
            .map(|(pid, syscall)| {
                let executor = self.executor.clone();
                tokio::task::spawn_blocking(move || executor.execute(pid, syscall))
            })
            .collect();

        let results = join_all(futures).await;
        let mut output = Vec::with_capacity(count);
        for r in results {
            output.push(r.unwrap_or_else(|e| SyscallResult::Error {
                message: format!("Task error: {}", e).into(),
            }));
        }
        output
    }

    async fn execute_sequential(&self, requests: Vec<(Pid, Syscall)>) -> Vec<SyscallResult> {
        use crate::core::optimization::prefetch_read;

        let mut results = Vec::with_capacity(requests.len());
        let len = requests.len();

        for (i, (pid, syscall)) in requests.into_iter().enumerate() {
            if i + 2 < len {
                prefetch_read(&pid as *const _);
            }

            let executor = self.executor.clone();
            let result = tokio::task::spawn_blocking(move || executor.execute(pid, syscall))
                .await
                .unwrap_or_else(|e| SyscallResult::Error {
                    message: format!("Task error: {}", e).into(),
                });
            results.push(result);
        }
        results
    }
}
