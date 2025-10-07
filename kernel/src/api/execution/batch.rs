/*!
 * Batch Syscall Executor
 * Executes multiple syscalls efficiently in parallel or sequence
 */

use crate::core::types::Pid;
use crate::syscalls::{Syscall, SyscallExecutor, SyscallResult};
use futures::future::join_all;

#[derive(Clone)]
pub struct BatchExecutor {
    executor: SyscallExecutor,
}

impl BatchExecutor {
    pub fn new(executor: SyscallExecutor) -> Self {
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
        let futures: Vec<_> = requests
            .into_iter()
            .map(|(pid, syscall)| {
                let executor = self.executor.clone();
                tokio::task::spawn_blocking(move || executor.execute(pid, syscall))
            })
            .collect();

        let results = join_all(futures).await;
        results
            .into_iter()
            .map(|r| {
                r.unwrap_or_else(|e| SyscallResult::Error {
                    message: format!("Task error: {}", e),
                })
            })
            .collect()
    }

    async fn execute_sequential(&self, requests: Vec<(Pid, Syscall)>) -> Vec<SyscallResult> {
        let mut results = Vec::with_capacity(requests.len());
        for (pid, syscall) in requests {
            let executor = self.executor.clone();
            let result = tokio::task::spawn_blocking(move || executor.execute(pid, syscall))
                .await
                .unwrap_or_else(|e| SyscallResult::Error {
                    message: format!("Task error: {}", e),
                });
            results.push(result);
        }
        results
    }
}
