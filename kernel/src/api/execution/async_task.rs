/*!
 * Async Task Manager
 * Handles async syscall execution with progress tracking
 */

use crate::core::types::Pid;
use crate::syscalls::{Syscall, SyscallExecutor, SyscallResult};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::oneshot;
use uuid::Uuid;

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
}

#[derive(Clone)]
pub struct AsyncTaskManager {
    tasks: Arc<DashMap<String, Task>>,
    /// Track which tasks belong to which process for cleanup
    process_tasks: Arc<DashMap<Pid, Vec<String>>>,
    executor: SyscallExecutor,
}

impl AsyncTaskManager {
    pub fn new(executor: SyscallExecutor) -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
            process_tasks: Arc::new(DashMap::new()),
            executor,
        }
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
                task.cancel_tx = None;
            }

            // Execute with cancellation support
            let result = tokio::select! {
                _ = cancel_rx => {
                    if let Some(mut task) = tasks.get_mut(&task_id_clone) {
                        task.status = TaskStatus::Cancelled;
                    }
                    return;
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

    pub fn cleanup_completed(&self) {
        self.tasks.retain(|_, task| {
            !matches!(
                task.status,
                TaskStatus::Completed(_) | TaskStatus::Failed(_) | TaskStatus::Cancelled
            )
        });
    }

    /// Cleanup all tasks for a terminated process
    pub fn cleanup_process_tasks(&self, pid: Pid) -> usize {
        let task_ids = self.process_tasks.remove(&pid)
            .map(|(_, v)| v)
            .unwrap_or_default();

        let mut cleaned_count = 0;

        for task_id in &task_ids {
            if let Some(mut task) = self.tasks.get_mut(task_id) {
                // Try to cancel running tasks
                if let Some(cancel_tx) = task.cancel_tx.take() {
                    let _ = cancel_tx.send(());
                }
                // Mark as cancelled
                task.status = TaskStatus::Cancelled;
            }
            // Remove the task
            self.tasks.remove(task_id);
            cleaned_count += 1;
        }

        if cleaned_count > 0 {
            log::info!("Cleaned {} async tasks for terminated PID {}", cleaned_count, pid);
        }

        cleaned_count
    }

    /// Check if process has any active tasks
    pub fn has_process_tasks(&self, pid: Pid) -> bool {
        self.process_tasks
            .get(&pid)
            .map_or(false, |tasks| !tasks.is_empty())
    }
}
