/*!
 * Async Syscall Executor with Intelligent Dispatch
 *
 * ## Design Innovation: Dual-Mode Execution
 *
 * This executor automatically chooses between fast-path (synchronous) and
 * slow-path (async) execution based on compile-time classification.
 *
 * ### Fast Path (< 100ns)
 * - Direct synchronous execution
 * - Zero async overhead
 * - For: memory stats, process queries, system info
 *
 * ### Slow Path (1-1000ms)
 * - True async execution
 * - Non-blocking I/O
 * - For: file I/O, network, IPC, sleep
 *
 * ### Benefits
 * 1. **Zero-cost fast operations**: No unnecessary async machinery
 * 2. **Non-blocking slow operations**: True async for I/O
 * 3. **Backward compatible**: Same API surface
 * 4. **Automatic optimization**: Classification is compile-time
 *
 * ## Architecture
 *
 * ```text
 * ┌──────────────┐
 * │   Syscall    │
 * └──────┬───────┘
 *        │
 *        ├─── classify() ───┐
 *        │                  │
 *    ┌───▼────┐      ┌─────▼────┐
 *    │  Fast  │      │ Blocking │
 *    └───┬────┘      └─────┬────┘
 *        │                 │
 *        │ Sync            │ Async
 *        │                 │
 *    ┌───▼────┐      ┌─────▼─────┐
 *    │ Direct │      │tokio::spawn│
 *    │  Call  │      │   or       │
 *    │        │      │  io_uring  │
 *    └───┬────┘      └─────┬─────┘
 *        │                 │
 *        └────┬────────────┘
 *             │
 *        ┌────▼────┐
 *        │ Result  │
 *        └─────────┘
 * ```
 */

use super::classification::SyscallClass;
use super::dispatcher::AdaptiveDispatcher;
use crate::core::types::Pid;
use crate::monitoring::{span_syscall, Collector};
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::types::{Syscall, SyscallResult};
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info};

/// Async-capable syscall executor with intelligent dispatch
///
/// Wraps the existing `SyscallExecutorWithIpc` and adds async execution
/// capabilities while maintaining zero-cost fast-path for synchronous syscalls.
#[derive(Clone)]
pub struct AsyncSyscallExecutor {
    /// Underlying synchronous executor (for fast-path operations)
    sync_executor: SyscallExecutorWithIpc,

    /// Adaptive dispatcher (Phase 2 & 3: tokio::fs + io_uring)
    dispatcher: Option<Arc<AdaptiveDispatcher>>,

    /// Optional observability collector
    collector: Option<Arc<Collector>>,
}

impl AsyncSyscallExecutor {
    /// Create new async executor wrapping a sync executor
    pub fn new(sync_executor: SyscallExecutorWithIpc) -> Self {
        Self {
            collector: sync_executor.optional().collector.clone(),
            dispatcher: None,
            sync_executor,
        }
    }

    /// Create new async executor with adaptive dispatcher
    ///
    /// This enables Phase 2 & 3 features:
    /// - True async I/O with tokio::fs
    /// - io_uring for large file operations
    /// - Native async IPC
    pub fn with_dispatcher(
        sync_executor: SyscallExecutorWithIpc,
        dispatcher: Arc<AdaptiveDispatcher>,
    ) -> Self {
        Self {
            collector: sync_executor.optional().collector.clone(),
            dispatcher: Some(dispatcher),
            sync_executor,
        }
    }

    /// Execute a syscall with intelligent sync/async dispatch
    ///
    /// This is the main entry point. It automatically chooses between:
    /// - Fast-path: Synchronous execution for fast operations
    /// - Slow-path: Async execution for blocking operations
    ///
    /// # Performance
    ///
    /// - Fast syscalls: < 100ns (direct call)
    /// - Blocking syscalls: ~1-10μs dispatch overhead + operation time
    pub async fn execute(&self, pid: Pid, syscall: Syscall) -> SyscallResult {
        // Classify syscall at compile-time (inlined, zero cost)
        match syscall.classify() {
            SyscallClass::Fast => {
                // Fast path: Direct synchronous execution
                self.execute_fast_path(pid, syscall)
            }
            SyscallClass::Blocking => {
                // Slow path: Async execution
                self.execute_async_path(pid, syscall).await
            }
        }
    }

    /// Execute fast-path syscalls synchronously (zero async overhead)
    ///
    /// Fast syscalls complete in < 100ns and never block, so we execute
    /// them directly without any async machinery. This is just a wrapper
    /// around the existing synchronous executor.
    ///
    /// # Performance
    ///
    /// - Inlined by compiler
    /// - Zero overhead vs direct sync call
    /// - No future allocation
    #[inline]
    fn execute_fast_path(&self, pid: Pid, syscall: Syscall) -> SyscallResult {
        // Direct synchronous execution via existing executor
        self.sync_executor.execute(pid, syscall)
    }

    /// Execute blocking syscalls asynchronously
    ///
    /// Phase 2 & 3 Implementation:
    /// 1. **Adaptive dispatcher** chooses between tokio::fs and io_uring
    /// 2. **True async I/O** with tokio::fs (no spawn_blocking)
    /// 3. **io_uring** for large files and batched operations
    /// 4. **Fallback** to spawn_blocking for unsupported operations
    ///
    /// # Execution Paths
    ///
    /// - **With dispatcher**: Uses adaptive path (tokio::fs or io_uring)
    /// - **Without dispatcher**: Falls back to spawn_blocking (backward compat)
    async fn execute_async_path(&self, pid: Pid, syscall: Syscall) -> SyscallResult {
        let syscall_name = syscall.name();

        // Create observability span
        let span = span_syscall(syscall_name, pid);
        let _guard = span.enter();

        let start = Instant::now();

        // Phase 2 & 3: Use adaptive dispatcher if available
        let result = if let Some(ref dispatcher) = self.dispatcher {
            info!(
                pid = pid,
                syscall = syscall_name,
                trace_id = %span.trace_id(),
                execution_mode = "adaptive",
                "Executing syscall (adaptive async path)"
            );

            // True async I/O (tokio::fs or io_uring)
            dispatcher.execute(pid, syscall).await
        } else {
            // Fallback: spawn_blocking for backward compatibility
            info!(
                pid = pid,
                syscall = syscall_name,
                trace_id = %span.trace_id(),
                execution_mode = "spawn_blocking",
                "Executing syscall (fallback async path)"
            );

            let executor = self.sync_executor.clone();
            let result = tokio::task::spawn_blocking(move || executor.execute(pid, syscall)).await;

            match result {
                Ok(res) => res,
                Err(e) => {
                    error!("Async syscall execution failed: {}", e);
                    SyscallResult::error(format!("Async execution error: {}", e))
                }
            }
        };

        // Emit observability event
        if let Some(ref collector) = self.collector {
            let duration_us = start.elapsed().as_micros() as u64;
            let success = matches!(result, SyscallResult::Success { .. });
            collector.syscall_exit(pid, syscall_name.to_string(), duration_us, success);
        }

        // Record result in span
        match &result {
            SyscallResult::Success { data } => {
                span.record_result(true);
                if let Some(d) = data {
                    span.record("data_size", d.len());
                }
            }
            SyscallResult::Error { message } => {
                span.record_error(message);
            }
            SyscallResult::PermissionDenied { reason } => {
                span.record_error(&format!("Permission denied: {}", reason));
            }
        }

        result
    }

    /// Execute batch of syscalls concurrently
    ///
    /// Phase 2 & 3 Enhancement:
    /// - With dispatcher: Uses adaptive batch execution (io_uring for large batches)
    /// - Without dispatcher: Concurrent futures (tokio concurrency)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = executor.execute_batch(pid, vec![
    ///     Syscall::ReadFile { path: "file1.txt".into() },
    ///     Syscall::ReadFile { path: "file2.txt".into() },
    ///     Syscall::ReadFile { path: "file3.txt".into() },
    /// ]).await;
    /// ```
    ///
    /// # Performance
    ///
    /// - Fast syscalls execute synchronously (no concurrency benefit)
    /// - Blocking syscalls with dispatcher: io_uring batching (best throughput)
    /// - Blocking syscalls without dispatcher: concurrent futures (good latency)
    /// - Mixed batches get best of both worlds
    pub async fn execute_batch(&self, pid: Pid, syscalls: Vec<Syscall>) -> Vec<SyscallResult> {
        // Phase 2 & 3: Use dispatcher's batch execution if available
        if let Some(ref dispatcher) = self.dispatcher {
            return dispatcher.execute_batch(pid, syscalls).await;
        }

        // Fallback: Execute all syscalls concurrently using futures
        let futures: Vec<_> = syscalls
            .into_iter()
            .map(|syscall| self.execute(pid, syscall))
            .collect();

        futures::future::join_all(futures).await
    }

    /// Execute syscalls in pipeline (output of one feeds into next)
    ///
    /// This demonstrates composable async operations. Results can flow
    /// through a pipeline of transformations.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Read file -> Process -> Write result
    /// let result = executor.execute_pipeline(pid, vec![
    ///     Syscall::ReadFile { path: "input.txt".into() },
    ///     // ... processing syscalls ...
    ///     Syscall::WriteFile { path: "output.txt".into(), data: processed },
    /// ]).await;
    /// ```
    pub async fn execute_pipeline(&self, pid: Pid, syscalls: Vec<Syscall>) -> SyscallResult {
        let mut last_result = SyscallResult::Success { data: None };

        for syscall in syscalls {
            // Only continue if last operation succeeded
            if !matches!(last_result, SyscallResult::Success { .. }) {
                return last_result;
            }

            last_result = self.execute(pid, syscall).await;
        }

        last_result
    }

    /// Get underlying sync executor
    ///
    /// Allows access to the underlying synchronous executor for
    /// compatibility with existing code.
    pub fn sync_executor(&self) -> &SyscallExecutorWithIpc {
        &self.sync_executor
    }

    /// Check if adaptive dispatcher is enabled
    pub fn has_dispatcher(&self) -> bool {
        self.dispatcher.is_some()
    }
}

// ============================================================================
// Performance Monitoring
// ============================================================================

/// Statistics for async executor performance
#[derive(Debug, Clone, Default)]
pub struct AsyncExecutorStats {
    /// Number of fast-path syscalls (synchronous)
    pub fast_path_count: u64,

    /// Number of slow-path syscalls (async)
    pub slow_path_count: u64,

    /// Total fast-path execution time (nanoseconds)
    pub fast_path_time_ns: u64,

    /// Total slow-path execution time (nanoseconds)
    pub slow_path_time_ns: u64,
}

impl AsyncExecutorStats {
    /// Get average fast-path latency
    pub fn avg_fast_path_ns(&self) -> Option<f64> {
        if self.fast_path_count == 0 {
            None
        } else {
            Some(self.fast_path_time_ns as f64 / self.fast_path_count as f64)
        }
    }

    /// Get average slow-path latency
    pub fn avg_slow_path_ns(&self) -> Option<f64> {
        if self.slow_path_count == 0 {
            None
        } else {
            Some(self.slow_path_time_ns as f64 / self.slow_path_count as f64)
        }
    }

    /// Get fast-path ratio
    pub fn fast_path_ratio(&self) -> f64 {
        let total = self.fast_path_count + self.slow_path_count;
        if total == 0 {
            0.0
        } else {
            self.fast_path_count as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SandboxManager;

    fn create_test_executor() -> AsyncSyscallExecutor {
        let sandbox = SandboxManager::new();
        let memory_manager = crate::memory::MemoryManager::new();
        let pipe_manager = crate::ipc::PipeManager::new(memory_manager.clone());
        let shm_manager = crate::ipc::ShmManager::new(memory_manager);

        let sync_executor =
            SyscallExecutorWithIpc::with_ipc_direct(sandbox, pipe_manager, shm_manager);

        AsyncSyscallExecutor::new(sync_executor)
    }

    #[tokio::test]
    #[ignore] // Requires process manager setup
    async fn test_fast_path_execution() {
        let executor = create_test_executor();
        let pid = 1;

        // Fast syscall should execute synchronously
        let syscall = Syscall::GetMemoryStats;
        assert!(syscall.is_fast());

        let result = executor.execute(pid, syscall).await;
        assert!(matches!(result, SyscallResult::Success { .. }));
    }

    #[tokio::test]
    #[ignore] // Requires process manager setup
    async fn test_blocking_path_execution() {
        let executor = create_test_executor();
        let pid = 1;

        // Blocking syscall should execute asynchronously
        let syscall = Syscall::Sleep { duration_ms: 10 };
        assert!(syscall.is_blocking());

        let start = Instant::now();
        let result = executor.execute(pid, syscall).await;
        let elapsed = start.elapsed();

        // Should have actually waited
        assert!(elapsed.as_millis() >= 10);
        assert!(matches!(result, SyscallResult::Success { .. }));
    }

    #[tokio::test]
    #[ignore] // Requires process manager setup
    async fn test_batch_execution() {
        let executor = create_test_executor();
        let pid = 1;

        let syscalls = vec![
            Syscall::GetMemoryStats,
            Syscall::GetSystemInfo,
            Syscall::GetUptime,
        ];

        let results = executor.execute_batch(pid, syscalls).await;

        assert_eq!(results.len(), 3);
        for result in results {
            assert!(matches!(result, SyscallResult::Success { .. }));
        }
    }
}
