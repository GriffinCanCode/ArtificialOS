/*!
 * Async Task Manager
 * Handles async syscall execution with progress tracking
 */

use crate::core::types::Pid;
use crate::syscalls::{Syscall, SyscallExecutor, SyscallResult};
use dashmap::DashMap;
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

#[derive(Clone)]
pub struct AsyncTaskManager {
    tasks: Arc<DashMap<String, Task>>,
    /// Track which tasks belong to which process for cleanup
    process_tasks: Arc<DashMap<Pid, Vec<String>>>,
    executor: SyscallExecutor,
    /// TTL for completed tasks before automatic cleanup
    task_ttl: Duration,
}

impl AsyncTaskManager {
    pub fn new(executor: SyscallExecutor) -> Self {
        Self::with_ttl(executor, DEFAULT_TASK_TTL)
    }

    pub fn with_ttl(executor: SyscallExecutor, task_ttl: Duration) -> Self {
        let manager = Self {
            tasks: Arc::new(DashMap::new()),
            process_tasks: Arc::new(DashMap::new()),
            executor,
            task_ttl,
        };

        // Start background cleanup task
        manager.start_cleanup_task();

        manager
    }

    /// Start background task for automatic cleanup of expired tasks
    fn start_cleanup_task(&self) {
        let tasks = Arc::clone(&self.tasks);
        let process_tasks = Arc::clone(&self.process_tasks);
        let ttl = self.task_ttl;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
            loop {
                interval.tick().await;

                let now = Instant::now();
                let mut cleaned_count = 0;
                let mut task_ids_to_remove = Vec::new();

                // First pass: identify expired tasks
                for entry in tasks.iter() {
                    let task_id = entry.key();
                    let task = entry.value();

                    if let Some(completed_at) = task.completed_at {
                        if now.duration_since(completed_at) > ttl {
                            task_ids_to_remove.push((task_id.clone(), task.pid));
                        }
                    }
                }

                // Second pass: remove expired tasks
                for (task_id, pid) in task_ids_to_remove {
                    tasks.remove(&task_id);
                    cleaned_count += 1;

                    // Also remove from process_tasks
                    if let Some(mut task_list) = process_tasks.get_mut(&pid) {
                        task_list.retain(|id| id != &task_id);
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
        });
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

        // Track task for this process
        self.process_tasks
            .entry(pid)
            .or_insert_with(Vec::new)
            .push(task_id.clone());

        // Spawn async execution
        let tasks = Arc::clone(&self.tasks);
        let executor = self.executor.clone();
        let task_id_clone = task_id.clone();

        tokio::spawn(async move {
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
        let mut task_ids_to_remove = Vec::new();

        // Collect tasks to remove
        for entry in self.tasks.iter() {
            let task_id = entry.key();
            let task = entry.value();

            if let Some(completed_at) = task.completed_at {
                if now.duration_since(completed_at) > ttl {
                    task_ids_to_remove.push((task_id.clone(), task.pid));
                }
            }
        }

        // Remove expired tasks
        for (task_id, pid) in task_ids_to_remove {
            self.tasks.remove(&task_id);
            cleaned_count += 1;

            // Also remove from process_tasks
            if let Some(mut task_list) = self.process_tasks.get_mut(&pid) {
                task_list.retain(|id| id != &task_id);
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
        let mut task_ids_to_remove = Vec::new();

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

            // Also remove from process_tasks
            if let Some(mut task_list) = self.process_tasks.get_mut(&pid) {
                task_list.retain(|id| id != &task_id);
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
