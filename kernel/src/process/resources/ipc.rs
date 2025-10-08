/*!
 * IPC Resource Cleanup
 * Per-process IPC tracking and cleanup
 */

use super::{CleanupStats, ResourceCleanup};
use crate::core::types::Pid;
use crate::ipc::IPCManager;

/// IPC resource cleanup wrapper
pub struct IpcResource {
    manager: IPCManager,
}

impl IpcResource {
    pub fn new(manager: IPCManager) -> Self {
        Self { manager }
    }
}

impl ResourceCleanup for IpcResource {
    fn cleanup(&self, pid: Pid) -> CleanupStats {
        let count = self.manager.clear_process_queue(pid);

        CleanupStats {
            resources_freed: count,
            bytes_freed: 0,
            errors_encountered: 0,
            cleanup_duration_micros: 0,
            by_type: std::collections::HashMap::new(),
        }
    }

    fn resource_type(&self) -> &'static str {
        "ipc"
    }

    fn has_resources(&self, pid: Pid) -> bool {
        self.manager.has_messages(pid)
    }
}
