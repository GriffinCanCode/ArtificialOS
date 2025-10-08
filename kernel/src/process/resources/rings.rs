/*!
 * Ring Resource Cleanup
 * Zero-copy and io_uring ring cleanup
 */

use super::{CleanupStats, ResourceCleanup};
use crate::core::types::Pid;
use crate::ipc::zerocopy::ZeroCopyIpc;
use crate::syscalls::iouring::IoUringManager;

/// Zero-copy ring resource cleanup
pub struct ZeroCopyResource {
    manager: ZeroCopyIpc,
}

impl ZeroCopyResource {
    pub fn new(manager: ZeroCopyIpc) -> Self {
        Self { manager }
    }
}

impl ResourceCleanup for ZeroCopyResource {
    fn cleanup(&self, pid: Pid) -> CleanupStats {
        let (count, bytes) = self.manager.cleanup_process_rings(pid);

        CleanupStats {
            resources_freed: count,
            bytes_freed: bytes,
            errors_encountered: 0,
            cleanup_duration_micros: 0,
            by_type: std::collections::HashMap::new(),
        }
    }

    fn resource_type(&self) -> &'static str {
        "zerocopy_rings"
    }

    fn has_resources(&self, pid: Pid) -> bool {
        self.manager.has_process_rings(pid)
    }
}

/// io_uring ring resource cleanup
pub struct IoUringResource {
    manager: IoUringManager,
}

impl IoUringResource {
    pub fn new(manager: IoUringManager) -> Self {
        Self { manager }
    }
}

impl ResourceCleanup for IoUringResource {
    fn cleanup(&self, pid: Pid) -> CleanupStats {
        let count = self.manager.cleanup_process_rings(pid);

        CleanupStats {
            resources_freed: count,
            bytes_freed: 0,
            errors_encountered: 0,
            cleanup_duration_micros: 0,
            by_type: std::collections::HashMap::new(),
        }
    }

    fn resource_type(&self) -> &'static str {
        "iouring_rings"
    }

    fn has_resources(&self, pid: Pid) -> bool {
        self.manager.has_process_rings(pid)
    }
}

/// Combined ring resource (for convenience)
pub struct RingResource {
    zerocopy: Option<ZeroCopyResource>,
    iouring: Option<IoUringResource>,
}

impl RingResource {
    pub fn new() -> Self {
        Self {
            zerocopy: None,
            iouring: None,
        }
    }

    pub fn with_zerocopy(mut self, manager: ZeroCopyIpc) -> Self {
        self.zerocopy = Some(ZeroCopyResource::new(manager));
        self
    }

    pub fn with_iouring(mut self, manager: IoUringManager) -> Self {
        self.iouring = Some(IoUringResource::new(manager));
        self
    }
}

impl Default for RingResource {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceCleanup for RingResource {
    fn cleanup(&self, pid: Pid) -> CleanupStats {
        let mut stats = CleanupStats::default();

        if let Some(ref zc) = self.zerocopy {
            let zc_stats = zc.cleanup(pid);
            stats.resources_freed += zc_stats.resources_freed;
            stats.bytes_freed += zc_stats.bytes_freed;
            stats.errors_encountered += zc_stats.errors_encountered;
        }

        if let Some(ref io) = self.iouring {
            let io_stats = io.cleanup(pid);
            stats.resources_freed += io_stats.resources_freed;
            stats.bytes_freed += io_stats.bytes_freed;
            stats.errors_encountered += io_stats.errors_encountered;
        }

        stats
    }

    fn resource_type(&self) -> &'static str {
        "rings"
    }

    fn has_resources(&self, pid: Pid) -> bool {
        self.zerocopy
            .as_ref()
            .map_or(false, |zc| zc.has_resources(pid))
            || self
                .iouring
                .as_ref()
                .map_or(false, |io| io.has_resources(pid))
    }
}
