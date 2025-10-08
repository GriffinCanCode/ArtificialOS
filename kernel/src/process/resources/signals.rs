/*!
 * Signal Callback Resource Cleanup
 * Per-process signal handler tracking and cleanup
 */

use super::{CleanupStats, ResourceCleanup};
use crate::core::types::Pid;
use crate::signals::SignalManagerImpl as SignalManager;

/// Signal callback resource cleanup wrapper
pub struct SignalResource {
    manager: SignalManager,
}

impl SignalResource {
    pub fn new(manager: SignalManager) -> Self {
        Self { manager }
    }
}

impl ResourceCleanup for SignalResource {
    fn cleanup(&self, pid: Pid) -> CleanupStats {
        let count = self.manager.cleanup_process_signals(pid);

        CleanupStats {
            resources_freed: count,
            bytes_freed: 0,
            errors_encountered: 0,
        }
    }

    fn resource_type(&self) -> &'static str {
        "signals"
    }

    fn has_resources(&self, pid: Pid) -> bool {
        self.manager.has_process_signals(pid)
    }
}
