/*!
 * Async Task Manager
 * Handles async syscall execution with progress tracking
 */

use crate::core::types::Pid;
use crate::syscalls::{Syscall, SyscallExecutor, SyscallResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
    status: TaskStatus,
    progress: f32,
    cancel_tx: Option<oneshot::Sender<()>>,
}

#[derive(Clone)]
pub struct AsyncTaskManager {
    tasks: Arc<Mutex<HashMap<String, Task>>>,
    executor: SyscallExecutor,
}

impl AsyncTaskManager {
    pub fn new(executor: SyscallExecutor) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            executor,
        }
    }

    pub fn submit(&self, pid: Pid, syscall: Syscall) -> String {
        let task_id = Uuid::new_v4().to_string();
        let (cancel_tx, cancel_rx) = oneshot::channel();

        // Insert pending task
        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(
                task_id.clone(),
                Task {
                    status: TaskStatus::Pending,
                    progress: 0.0,
                    cancel_tx: Some(cancel_tx),
                },
            );
        }

        // Spawn async execution
        let tasks = Arc::clone(&self.tasks);
        let executor = self.executor.clone();
        let task_id_clone = task_id.clone();

        tokio::spawn(async move {
            // Update to running
            {
                let mut tasks = tasks.lock().unwrap();
                if let Some(task) = tasks.get_mut(&task_id_clone) {
                    task.status = TaskStatus::Running;
                    task.cancel_tx = None;
                }
            }

            // Execute with cancellation support
            let result = tokio::select! {
                _ = cancel_rx => {
                    let mut tasks = tasks.lock().unwrap();
                    if let Some(task) = tasks.get_mut(&task_id_clone) {
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
            let mut tasks = tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(&task_id_clone) {
                task.status = TaskStatus::Completed(result);
                task.progress = 1.0;
            }
        });

        task_id
    }

    pub fn get_status(&self, task_id: &str) -> Option<(TaskStatus, f32)> {
        let tasks = self.tasks.lock().unwrap();
        tasks
            .get(task_id)
            .map(|task| (task.status.clone(), task.progress))
    }

    pub fn cancel(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(task_id) {
            if let Some(cancel_tx) = task.cancel_tx.take() {
                let _ = cancel_tx.send(());
                return true;
            }
        }
        false
    }

    pub fn cleanup_completed(&self) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.retain(|_, task| {
            !matches!(
                task.status,
                TaskStatus::Completed(_) | TaskStatus::Failed(_) | TaskStatus::Cancelled
            )
        });
    }
}
