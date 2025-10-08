/*!
 * Async Task Resource Cleanup
 * Per-process async task tracking and cleanup
 */

use super::{CleanupStats, ResourceCleanup};
use crate::api::execution::AsyncTaskManager;
use crate::core::types::Pid;

/// Async task resource cleanup wrapper
pub struct TaskResource {
    manager: AsyncTaskManager,
}

impl TaskResource {
    pub fn new(manager: AsyncTaskManager) -> Self {
        Self { manager }
    }
}

impl ResourceCleanup for TaskResource {
    fn cleanup(&self, pid: Pid) -> CleanupStats {
        let count = self.manager.cleanup_process_tasks(pid);

        CleanupStats {
            resources_freed: count,
            bytes_freed: 0,
            errors_encountered: 0,
        }
    }

    fn resource_type(&self) -> &'static str {
        "async_tasks"
    }

    fn has_resources(&self, pid: Pid) -> bool {
        self.manager.has_process_tasks(pid)
    }
}
