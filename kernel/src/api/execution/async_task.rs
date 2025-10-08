/*!
 * Async Task Manager
 * Handles async syscall execution with progress tracking
 *
 * # Graceful-with-Fallback Cleanup Task Management
 *
 * The background cleanup task uses the same pattern as SchedulerTask:
 * - Preferred: Call `shutdown().await` on the last clone for graceful cleanup
 * - Fallback: Drop will abort the cleanup task if not shut down gracefully
 * - The cleanup task runs every 5 minutes to remove expired completed tasks
 */

use crate::core::guard::AsyncTaskGuard;
use crate::core::types::Pid;
use crate::syscalls::{Syscall, SyscallExecutorWithIpc, SyscallResult};
use dashmap::DashMap;
use log::warn;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use uuid::Uuid;

/// Default TTL for completed tasks (1 hour)
const DEFAULT_TASK_TTL: Duration = Duration::from_secs(3600);

/// Cleanup interval for background task (5 minutes)
const CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed(SyscallResult),
    Failed(String),
    Cancelled,
}

struct Task {
    pid: Pid,
    status: TaskStatus,
    progress: f32,
    cancel_tx: Option<oneshot::Sender<()>>,
    /// Timestamp when task completed/failed/cancelled (for TTL-based cleanup)
    completed_at: Option<Instant>,
}

/// Background cleanup task handle
struct CleanupTaskHandle {
    handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    shutdown_initiated: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct AsyncTaskManager {
    tasks: Arc<DashMap<String, Task>>,
    /// Track which tasks belong to which process (HashSet for O(1) removal)
    process_tasks: Arc<DashMap<Pid, HashSet<String>>>,
    executor: SyscallExecutorWithIpc,
    /// TTL for completed tasks before automatic cleanup
    task_ttl: Duration,
    /// Handle to background cleanup task (shared across clones)
    cleanup_task: Arc<Mutex<CleanupTaskHandle>>,
}

impl AsyncTaskManager {
    pub fn new(executor: SyscallExecutorWithIpc) -> Self {
        Self::with_ttl(executor, DEFAULT_TASK_TTL)
    }

    pub fn with_ttl(executor: SyscallExecutorWithIpc, task_ttl: Duration) -> Self {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let shutdown_initiated = Arc::new(AtomicBool::new(false));

        let tasks = Arc::new(DashMap::new());
        let process_tasks = Arc::new(DashMap::new());

        // Start background cleanup task
        let handle = Self::spawn_cleanup_task(
            Arc::clone(&tasks),
            Arc::clone(&process_tasks),
            task_ttl,
            shutdown_rx,
        );

        let cleanup_task = Arc::new(Mutex::new(CleanupTaskHandle {
            handle: Some(handle),
            shutdown_tx: Some(shutdown_tx),
            shutdown_initiated,
        }));

        Self {
            tasks,
            process_tasks,
            executor,
            task_ttl,
            cleanup_task,
        }
    }

    /// Spawn background cleanup task with graceful shutdown support
    fn spawn_cleanup_task(
        tasks: Arc<DashMap<String, Task>>,
        process_tasks: Arc<DashMap<Pid, HashSet<String>>>,
        ttl: Duration,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            log::info!(
                "AsyncTaskManager cleanup task started (interval: {:?}, TTL: {:?})",
                CLEANUP_INTERVAL,
                ttl
            );

            loop {
                tokio::select! {
                    // Shutdown signal received
                    _ = &mut shutdown_rx => {
                        log::info!("AsyncTaskManager cleanup task shutting down gracefully");
                        break;
                    }

                    // Periodic cleanup tick
                    _ = interval.tick() => {
                        let now = Instant::now();
                        let mut cleaned_count = 0;
                        let mut task_ids_to_remove = Vec::with_capacity(16);

                        for entry in tasks.iter() {
                            let task_id = entry.key();
                            let task = entry.value();

                            if let Some(completed_at) = task.completed_at {
                                if now.duration_since(completed_at) > ttl {
                                    task_ids_to_remove.push((task_id.clone(), task.pid));
                                }
                            }
                        }

                        for (task_id, pid) in task_ids_to_remove {
                            tasks.remove(&task_id);
                            cleaned_count += 1;

                            // Also remove from process_tasks (O(1) HashSet removal)
                            if let Some(mut task_set) = process_tasks.get_mut(&pid) {
                                task_set.remove(&task_id);
                            }
                        }

                        if cleaned_count > 0 {
                            log::info!(
                                "Background cleanup: removed {} expired async tasks (TTL: {:?})",
                                cleaned_count,
                                ttl
                            );
                        }
                    }
                }
            }

            log::info!("AsyncTaskManager cleanup task stopped");
        })
    }

    pub fn submit(&self, pid: Pid, syscall: Syscall) -> String {
        let task_id = Uuid::new_v4().to_string();
        let (cancel_tx, cancel_rx) = oneshot::channel();

        // Insert pending task
        self.tasks.insert(
            task_id.clone(),
            Task {
                pid,
                status: TaskStatus::Pending,
                progress: 0.0,
                cancel_tx: Some(cancel_tx),
                completed_at: None,
            },
        );

        // Track task for this process (HashSet for O(1) operations)
        self.process_tasks
            .entry(pid)
            .or_insert_with(HashSet::new)
            .insert(task_id.clone());

        // Spawn async execution with guard for automatic cleanup
        let tasks = Arc::clone(&self.tasks);
        let executor = self.executor.clone();
        let task_id_clone = task_id.clone();

        let handle = tokio::spawn(async move {
            // Update to running
            if let Some(mut task) = tasks.get_mut(&task_id_clone) {
                task.status = TaskStatus::Running;
                // Don't drop cancel_tx yet - we need it to stay alive for cancellation
            }

            // Execute with cancellation support
            let result = tokio::select! {
                result = cancel_rx => {
                    // Check if it was an explicit cancellation (Ok) or channel drop (Err)
                    match result {
                        Ok(_) => {
                            // Explicitly cancelled
                            if let Some(mut task) = tasks.get_mut(&task_id_clone) {
                                task.status = TaskStatus::Cancelled;
                                task.completed_at = Some(Instant::now());
                                task.cancel_tx = None;
                            }
                            return;
                        }
                        Err(_) => {
                            // Cancellation channel was dropped unexpectedly
                            SyscallResult::Error {
                                message: "Cancellation channel dropped unexpectedly".into(),
                            }
                        }
                    }
                }
                result = tokio::task::spawn_blocking(move || executor.execute(pid, syscall)) => {
                    result.unwrap_or_else(|e| SyscallResult::Error {
                        message: format!("Task panic: {}", e),
                    })
                }
            };

            // Update with result
            if let Some(mut task) = tasks.get_mut(&task_id_clone) {
                task.status = TaskStatus::Completed(result);
                task.progress = 1.0;
                task.completed_at = Some(Instant::now());
                task.cancel_tx = None; // Clear cancellation channel after completion
            }
        });

        // Guard ensures task is tracked and can be cancelled on drop
        // Note: We don't store the guard since tasks are tracked in DashMap
        // The guard is useful for external callers who want auto-cancellation
        let _guard = AsyncTaskGuard::from_handle(handle, Some(pid)).no_auto_cancel();

        task_id
    }

    pub fn get_status(&self, task_id: &str) -> Option<(TaskStatus, f32)> {
        self.tasks
            .get(task_id)
            .map(|task| (task.status.clone(), task.progress))
    }

    pub fn cancel(&self, task_id: &str) -> bool {
        if let Some(mut task) = self.tasks.get_mut(task_id) {
            if let Some(cancel_tx) = task.cancel_tx.take() {
                let _ = cancel_tx.send(());
                return true;
            }
        }
        false
    }

    /// Cleanup completed tasks that have exceeded their TTL
    /// Can be called manually by gRPC handlers for immediate cleanup
    pub fn cleanup_completed(&self) -> usize {
        let now = Instant::now();
        let ttl = self.task_ttl;
        let mut cleaned_count = 0;
        let mut task_ids_to_remove = Vec::with_capacity(16);

        for entry in self.tasks.iter() {
            let task_id = entry.key();
            let task = entry.value();

            if let Some(completed_at) = task.completed_at {
                if now.duration_since(completed_at) > ttl {
                    task_ids_to_remove.push((task_id.clone(), task.pid));
                }
            }
        }

        for (task_id, pid) in task_ids_to_remove {
            self.tasks.remove(&task_id);
            cleaned_count += 1;

            // Also remove from process_tasks (O(1) HashSet removal)
            if let Some(mut task_set) = self.process_tasks.get_mut(&pid) {
                task_set.remove(&task_id);
            }
        }

        if cleaned_count > 0 {
            log::debug!(
                "Manual cleanup: removed {} expired async tasks",
                cleaned_count
            );
        }

        cleaned_count
    }

    /// Cleanup all completed tasks regardless of TTL
    /// Useful for testing or forced cleanup scenarios
    pub fn cleanup_completed_immediate(&self) -> usize {
        let mut cleaned_count = 0;
        let mut task_ids_to_remove = Vec::with_capacity(32);

        // Collect all completed tasks
        for entry in self.tasks.iter() {
            let task_id = entry.key();
            let task = entry.value();

            if matches!(
                task.status,
                TaskStatus::Completed(_) | TaskStatus::Failed(_) | TaskStatus::Cancelled
            ) {
                task_ids_to_remove.push((task_id.clone(), task.pid));
            }
        }

        // Remove all completed tasks
        for (task_id, pid) in task_ids_to_remove {
            self.tasks.remove(&task_id);
            cleaned_count += 1;

            // Also remove from process_tasks (O(1) HashSet removal)
            if let Some(mut task_set) = self.process_tasks.get_mut(&pid) {
                task_set.remove(&task_id);
            }
        }

        cleaned_count
    }

    /// Cleanup all tasks for a terminated process
    pub fn cleanup_process_tasks(&self, pid: Pid) -> usize {
        let task_ids = self
            .process_tasks
            .remove(&pid)
            .map(|(_, v)| v)
            .unwrap_or_default();

        let mut cleaned_count = 0;
        let now = Instant::now();

        for task_id in &task_ids {
            if let Some(mut task) = self.tasks.get_mut(task_id) {
                // Try to cancel running tasks
                if let Some(cancel_tx) = task.cancel_tx.take() {
                    let _ = cancel_tx.send(());
                }
                // Mark as cancelled with timestamp
                task.status = TaskStatus::Cancelled;
                task.completed_at = Some(now);
            }
            // Remove the task
            self.tasks.remove(task_id);
            cleaned_count += 1;
        }

        if cleaned_count > 0 {
            log::info!(
                "Cleaned {} async tasks for terminated PID {}",
                cleaned_count,
                pid
            );
        }

        cleaned_count
    }

    /// Check if process has any active tasks
    pub fn has_process_tasks(&self, pid: Pid) -> bool {
        self.process_tasks
            .get(&pid)
            .map_or(false, |tasks| !tasks.is_empty())
    }

    /// Get total number of tasks (all states)
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Submit task and return a guard for automatic cancellation
    ///
    /// The guard will cancel the task when dropped unless explicitly disabled.
    /// Useful for callers who want RAII semantics for task lifecycle.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let guard = manager.submit_with_guard(pid, syscall);
    /// // Task automatically cancelled if guard drops before completion
    /// ```
    pub fn submit_with_guard(
        &self,
        pid: Pid,
        syscall: Syscall,
    ) -> (String, AsyncTaskGuard<SyscallResult>) {
        let task_id = Uuid::new_v4().to_string();
        let (cancel_tx, cancel_rx) = oneshot::channel();

        // Insert pending task
        self.tasks.insert(
            task_id.clone(),
            Task {
                pid,
                status: TaskStatus::Pending,
                progress: 0.0,
                cancel_tx: Some(cancel_tx),
                completed_at: None,
            },
        );

        // Track task for this process
        self.process_tasks
            .entry(pid)
            .or_insert_with(HashSet::new)
            .insert(task_id.clone());

        // Spawn async execution
        let tasks = Arc::clone(&self.tasks);
        let executor = self.executor.clone();
        let task_id_clone = task_id.clone();

        let handle = tokio::spawn(async move {
            // Update to running
            if let Some(mut task) = tasks.get_mut(&task_id_clone) {
                task.status = TaskStatus::Running;
            }

            // Execute with cancellation support
            let result = tokio::select! {
                result = cancel_rx => {
                    match result {
                        Ok(_) => {
                            if let Some(mut task) = tasks.get_mut(&task_id_clone) {
                                task.status = TaskStatus::Cancelled;
                                task.completed_at = Some(Instant::now());
                                task.cancel_tx = None;
                            }
                            return SyscallResult::Error {
                                message: "Task cancelled".into(),
                            };
                        }
                        Err(_) => {
                            SyscallResult::Error {
                                message: "Cancellation channel dropped unexpectedly".into(),
                            }
                        }
                    }
                }
                result = tokio::task::spawn_blocking(move || executor.execute(pid, syscall)) => {
                    result.unwrap_or_else(|e| SyscallResult::Error {
                        message: format!("Task panic: {}", e),
                    })
                }
            };

            // Update with result
            if let Some(mut task) = tasks.get_mut(&task_id_clone) {
                task.status = TaskStatus::Completed(result.clone());
                task.progress = 1.0;
                task.completed_at = Some(Instant::now());
                task.cancel_tx = None;
            }

            result
        });

        // Create guard with auto-cancel enabled
        let guard = AsyncTaskGuard::from_handle(handle, Some(pid));

        (task_id, guard)
    }

    /// Get count of tasks by state
    pub fn task_stats(&self) -> TaskStats {
        let mut stats = TaskStats::default();

        for entry in self.tasks.iter() {
            match &entry.value().status {
                TaskStatus::Pending => stats.pending += 1,
                TaskStatus::Running => stats.running += 1,
                TaskStatus::Completed(_) => stats.completed += 1,
                TaskStatus::Failed(_) => stats.failed += 1,
                TaskStatus::Cancelled => stats.cancelled += 1,
            }
        }

        stats.total = self.tasks.len();
        stats
    }

    /// Shutdown the background cleanup task gracefully
    ///
    /// **Preferred shutdown method** - Call this before dropping the manager
    /// to ensure the cleanup task terminates cleanly. This is especially
    /// important in tests and during graceful kernel shutdown.
    ///
    /// Note: Because AsyncTaskManager is Clone, all clones share the same
    /// cleanup task. Calling shutdown on any clone will stop the task for all.
    ///
    /// # Example
    /// ```no_run
    /// # use ai_os_kernel::api::execution::async_task::AsyncTaskManager;
    /// # use ai_os_kernel::syscalls::SyscallExecutor;
    /// # async fn example(executor: SyscallExecutorWithIpc) {
    /// let manager = AsyncTaskManager::new(executor);
    /// // ... use manager ...
    /// manager.shutdown().await;
    /// # }
    /// ```
    pub async fn shutdown(&self) {
        let mut cleanup_handle = self.cleanup_task.lock();

        // Check if already shut down
        if cleanup_handle.shutdown_initiated.load(Ordering::SeqCst) {
            log::debug!("AsyncTaskManager cleanup task already shut down");
            return;
        }

        // Mark as initiated
        cleanup_handle
            .shutdown_initiated
            .store(true, Ordering::SeqCst);

        // Send shutdown signal
        if let Some(tx) = cleanup_handle.shutdown_tx.take() {
            let _ = tx.send(());
        }

        // Wait for cleanup task to complete
        if let Some(handle) = cleanup_handle.handle.take() {
            match handle.await {
                Ok(_) => log::info!("AsyncTaskManager cleanup task shutdown complete"),
                Err(e) => warn!("AsyncTaskManager cleanup task shutdown error: {}", e),
            }
        }
    }
}

impl Drop for CleanupTaskHandle {
    fn drop(&mut self) {
        // Check if graceful shutdown was initiated
        if self.shutdown_initiated.load(Ordering::SeqCst) {
            // Graceful shutdown path was used - nothing to do
            return;
        }

        // Fallback path: graceful shutdown wasn't called
        if let Some(handle) = self.handle.take() {
            warn!(
                "AsyncTaskManager cleanup task dropped without calling shutdown() - aborting task. \
                 Use `manager.shutdown().await` for graceful cleanup."
            );

            // Abort the cleanup task immediately (non-blocking but forceful)
            handle.abort();
        }
    }
}

/// Statistics about async tasks
#[derive(Debug, Default, Clone)]
pub struct TaskStats {
    pub total: usize,
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
}
