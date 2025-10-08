/*!
 * Memory Mapping Resource Cleanup
 * Per-process mmap tracking and cleanup
 */

use super::{CleanupStats, ResourceCleanup};
use crate::core::types::Pid;
use crate::ipc::MmapManager;

/// Memory mapping resource cleanup wrapper
pub struct MappingResource {
    manager: MmapManager,
}

impl MappingResource {
    pub fn new(manager: MmapManager) -> Self {
        Self { manager }
    }
}

impl ResourceCleanup for MappingResource {
    fn cleanup(&self, pid: Pid) -> CleanupStats {
        let (count, bytes) = self.manager.cleanup_process_mappings(pid);

        CleanupStats {
            resources_freed: count,
            bytes_freed: bytes,
            errors_encountered: 0,
            cleanup_duration_micros: 0,
            by_type: std::collections::HashMap::new(),
        }
    }

    fn resource_type(&self) -> &'static str {
        "mappings"
    }

    fn has_resources(&self, pid: Pid) -> bool {
        self.manager.has_process_mappings(pid)
    }
}
