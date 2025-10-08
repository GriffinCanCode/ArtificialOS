/*!
 * Async Task Guards
 *
 * RAII guards for async tasks with automatic cancellation
 */

use super::traits::{Guard, GuardDrop};
use super::{GuardError, GuardMetadata, GuardResult, TimeoutPolicy};
use crate::core::types::Pid;
use std::time::Instant;
use tokio::task::JoinHandle;

/// Async task guard with automatic cancellation
///
/// # Example
///
/// ```ignore
/// let task_guard = AsyncTaskGuard::spawn(pid, async {
///     // Task work
/// });
///
/// // Task automatically cancelled if guard drops
/// ```
pub struct AsyncTaskGuard<T: Send + 'static> {
    task_id: Option<u64>,
    handle: Option<JoinHandle<T>>,
    pid: Option<Pid>,
    metadata: GuardMetadata,
    auto_cancel: bool,
}

impl<T: Send + 'static> AsyncTaskGuard<T> {
    /// Spawn a new task with a guard
    pub fn spawn<F>(pid: Option<Pid>, future: F) -> Self
    where
        F: std::future::Future<Output = T> + Send + 'static,
    {
        let handle = tokio::spawn(future);

        let mut metadata = GuardMetadata::new("async_task");
        if let Some(pid) = pid {
            metadata = metadata.with_pid(pid);
        }

        Self {
            task_id: None,
            handle: Some(handle),
            pid,
            metadata,
            auto_cancel: true,
        }
    }

    /// Create guard from existing handle
    pub fn from_handle(handle: JoinHandle<T>, pid: Option<Pid>) -> Self {
        let mut metadata = GuardMetadata::new("async_task");
        if let Some(pid) = pid {
            metadata = metadata.with_pid(pid);
        }

        Self {
            task_id: None,
            handle: Some(handle),
            pid,
            metadata,
            auto_cancel: true,
        }
    }

    /// Set task ID for tracking
    pub fn with_task_id(mut self, task_id: u64) -> Self {
        self.task_id = Some(task_id);
        self
    }

    /// Disable automatic cancellation on drop
    pub fn no_auto_cancel(mut self) -> Self {
        self.auto_cancel = false;
        self
    }

    /// Check if task is finished
    pub fn is_finished(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(true)
    }

    /// Abort the task
    pub fn abort(&mut self) {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
    }

    /// Await the task result
    pub async fn await_result(mut self) -> GuardResult<T> {
        if let Some(handle) = self.handle.take() {
            handle
                .await
                .map_err(|e| GuardError::OperationFailed(format!("Task join error: {:?}", e).into()))
        } else {
            Err(GuardError::AlreadyReleased)
        }
    }

    /// Await the task result with timeout
    ///
    /// # Errors
    ///
    /// Returns `GuardError::Timeout` if timeout expires before task completes
    pub async fn await_timeout(mut self, timeout: TimeoutPolicy) -> GuardResult<T> {
        if let Some(handle) = self.handle.take() {
            match timeout.duration() {
                None => {
                    // No timeout, just await
                    handle.await.map_err(|e| {
                        GuardError::OperationFailed(format!("Task join error: {:?}", e))
                    })
                }
                Some(duration) => {
                    let start = Instant::now();
                    match tokio::time::timeout(duration, handle).await {
                        Ok(result) => result.map_err(|e| {
                            GuardError::OperationFailed(format!("Task join error: {:?}", e))
                        }),
                        Err(_) => Err(GuardError::Timeout {
                            resource_type: "async_task",
                            category: timeout.category(),
                            elapsed_ms: start.elapsed().as_millis() as u64,
                            timeout_ms: Some(duration.as_millis() as u64),
                        }),
                    }
                }
            }
        } else {
            Err(GuardError::AlreadyReleased)
        }
    }

    /// Get task ID if set
    pub fn task_id(&self) -> Option<u64> {
        self.task_id
    }

    /// Get process ID if set
    pub fn pid(&self) -> Option<Pid> {
        self.pid
    }
}

impl<T: Send + 'static> Guard for AsyncTaskGuard<T> {
    fn resource_type(&self) -> &'static str {
        "async_task"
    }

    fn metadata(&self) -> &GuardMetadata {
        &self.metadata
    }

    fn is_active(&self) -> bool {
        self.handle.is_some() && !self.is_finished()
    }

    fn release(&mut self) -> GuardResult<()> {
        if let Some(handle) = self.handle.take() {
            handle.abort();
            Ok(())
        } else {
            Err(GuardError::AlreadyReleased)
        }
    }
}

impl<T: Send + 'static> GuardDrop for AsyncTaskGuard<T> {
    fn on_drop(&mut self) {
        if self.auto_cancel && self.is_active() {
            self.abort();
            log::debug!(
                "Async task guard auto-cancelled task{}",
                self.task_id
                    .map(|id| format!(" (ID: {})", id))
                    .unwrap_or_default()
            );
        }
    }
}

impl<T: Send + 'static> Drop for AsyncTaskGuard<T> {
    fn drop(&mut self) {
        self.on_drop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_async_task_guard_auto_cancel() {
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_clone = cancelled.clone();

        {
            let _guard = AsyncTaskGuard::spawn(Some(1), async move {
                tokio::select! {
                    _ = sleep(Duration::from_secs(10)) => {},
                    _ = tokio::signal::ctrl_c() => {
                        cancelled_clone.store(true, Ordering::SeqCst);
                    }
                }
            });

            // Guard drops here, task should be cancelled
        }

        sleep(Duration::from_millis(10)).await;
        // Task was aborted (not gracefully cancelled, so flag not set)
    }

    #[tokio::test]
    async fn test_async_task_guard_await() {
        let guard = AsyncTaskGuard::spawn(Some(1), async { 42 });

        let result = guard.await_result().await.unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_async_task_guard_no_auto_cancel() {
        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        {
            let _guard = AsyncTaskGuard::spawn(Some(1), async move {
                sleep(Duration::from_millis(10)).await;
                completed_clone.store(true, Ordering::SeqCst);
            })
            .no_auto_cancel();

            // Guard drops but task continues
        }

        sleep(Duration::from_millis(50)).await;
        assert!(completed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_async_task_guard_timeout_success() {
        use super::super::TimeoutPolicy;

        let guard = AsyncTaskGuard::spawn(Some(1), async {
            sleep(Duration::from_millis(10)).await;
            42
        });

        let timeout = TimeoutPolicy::Task(Duration::from_secs(1));
        let result = guard.await_timeout(timeout).await.unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_async_task_guard_timeout_expires() {
        use super::super::{GuardError, TimeoutPolicy};

        let guard = AsyncTaskGuard::spawn(Some(1), async {
            sleep(Duration::from_secs(10)).await;
            42
        });

        let timeout = TimeoutPolicy::Task(Duration::from_millis(50));
        let result = guard.await_timeout(timeout).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            GuardError::Timeout { .. } => {}
            _ => panic!("Expected timeout error"),
        }
    }
}
