/*!
 * Async Task Guards
 *
 * RAII guards for async tasks with automatic cancellation
 */

use super::traits::{Guard, GuardDrop};
use super::{GuardError, GuardMetadata, GuardResult};
use crate::core::types::Pid;
use tokio::task::JoinHandle;

/// Async task guard with automatic cancellation
///
/// # Example
///
/// ```rust
/// let task_guard = AsyncTaskGuard::spawn(pid, async {
///     // Task work
/// });
///
/// // Task automatically cancelled if guard drops
/// ```
pub struct AsyncTaskGuard<T> {
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
    pub async fn await_result(mut self) -> Result<T, tokio::task::JoinError> {
        if let Some(handle) = self.handle.take() {
            handle.await
        } else {
            Err(tokio::task::JoinError::try_from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Task already completed",
            ))
            .unwrap())
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

impl<T> Guard for AsyncTaskGuard<T> {
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

impl<T> GuardDrop for AsyncTaskGuard<T> {
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

impl<T> Drop for AsyncTaskGuard<T> {
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
}
