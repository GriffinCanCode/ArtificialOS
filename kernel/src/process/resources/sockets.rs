/*!
 * Socket Resource Cleanup
 * Per-process network socket tracking and cleanup
 */

use super::{CleanupStats, ResourceCleanup};
use crate::core::types::Pid;
use crate::syscalls::SocketManager;

/// Socket resource cleanup wrapper
pub struct SocketResource {
    manager: SocketManager,
}

impl SocketResource {
    pub fn new(manager: SocketManager) -> Self {
        Self { manager }
    }
}

impl ResourceCleanup for SocketResource {
    fn cleanup(&self, pid: Pid) -> CleanupStats {
        let count = self.manager.cleanup_process_sockets(pid);

        CleanupStats {
            resources_freed: count,
            bytes_freed: 0,
            errors_encountered: 0,
            cleanup_duration_micros: 0,
            by_type: std::collections::HashMap::new(),
        }
    }

    fn resource_type(&self) -> &'static str {
        "sockets"
    }

    fn has_resources(&self, pid: Pid) -> bool {
        self.manager.has_process_sockets(pid)
    }
}
