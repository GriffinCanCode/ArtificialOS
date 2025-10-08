/*!
 * Memory Resource Cleanup
 * Per-process memory tracking and cleanup
 */

use super::{CleanupStats, ResourceCleanup};
use crate::core::types::Pid;
use crate::memory::MemoryManager;

/// Memory resource cleanup wrapper
pub struct MemoryResource {
    manager: MemoryManager,
}

impl MemoryResource {
    pub fn new(manager: MemoryManager) -> Self {
        Self { manager }
    }
}

impl ResourceCleanup for MemoryResource {
    fn cleanup(&self, pid: Pid) -> CleanupStats {
        let bytes = self.manager.free_process_memory(pid);

        CleanupStats {
            resources_freed: if bytes > 0 { 1 } else { 0 },
            bytes_freed: bytes,
            errors_encountered: 0,
            cleanup_duration_micros: 0,
            by_type: std::collections::HashMap::new(),
        }
    }

    fn resource_type(&self) -> &'static str {
        "memory"
    }

    fn has_resources(&self, pid: Pid) -> bool {
        self.manager.process_memory(pid) > 0
    }
}
