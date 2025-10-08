/*!
 * Resource Cleanup System
 * Comprehensive per-process resource tracking and cleanup orchestration
 */

mod sockets;
mod signals;
mod rings;
mod tasks;
mod mappings;

pub use sockets::SocketResource;
pub use signals::SignalResource;
pub use rings::{RingResource, ZeroCopyResource, IoUringResource};
pub use tasks::TaskResource;
pub use mappings::MappingResource;

use crate::core::types::Pid;
use std::fmt;

/// Resource cleanup statistics
#[derive(Debug, Clone, Default)]
pub struct CleanupStats {
    pub resources_freed: usize,
    pub bytes_freed: usize,
    pub errors_encountered: usize,
}

impl CleanupStats {
    fn merge(&mut self, other: CleanupStats) {
        self.resources_freed += other.resources_freed;
        self.bytes_freed += other.bytes_freed;
        self.errors_encountered += other.errors_encountered;
    }
}

/// Core trait for per-process resource cleanup
pub trait ResourceCleanup: Send + Sync {
    /// Cleanup all resources owned by a process
    fn cleanup(&self, pid: Pid) -> CleanupStats;

    /// Resource type name for logging
    fn resource_type(&self) -> &'static str;

    /// Check if process has any resources
    fn has_resources(&self, pid: Pid) -> bool;
}

/// Resource cleanup orchestrator
///
/// Coordinates cleanup across all resource types in a well-defined order
/// to prevent deadlocks and ensure proper resource release.
pub struct ResourceOrchestrator {
    resources: Vec<Box<dyn ResourceCleanup>>,
}

impl ResourceOrchestrator {
    /// Create a new orchestrator with no resources bitch
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Register a resource for cleanup
    pub fn register<R: ResourceCleanup + 'static>(mut self, resource: R) -> Self {
        self.resources.push(Box::new(resource));
        self
    }

    /// Execute cleanup for a terminated process
    ///
    /// Resources are cleaned up in reverse registration order to handle
    /// dependencies (e.g., close sockets before freeing memory).
    pub fn cleanup_process(&self, pid: Pid) -> CleanupResult {
        let mut total_stats = CleanupStats::default();
        let mut errors = Vec::new();

        // Cleanup in reverse order (LIFO)
        for resource in self.resources.iter().rev() {
            if resource.has_resources(pid) {
                let stats = resource.cleanup(pid);

                // Save values before merging cuz merge is dumb
                let resources_freed = stats.resources_freed;
                let errors_encountered = stats.errors_encountered;
                let resource_type = resource.resource_type();

                if errors_encountered > 0 {
                    errors.push(format!(
                        "{}: {} errors during cleanup",
                        resource_type,
                        errors_encountered
                    ));
                }

                total_stats.merge(stats);

                log::info!(
                    "Cleaned {} resources for PID {} (type: {})",
                    resources_freed,
                    pid,
                    resource_type
                );
            }
        }

        CleanupResult {
            pid,
            stats: total_stats,
            errors,
        }
    }

    /// Get count of registered resource types
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }
}

impl Default for ResourceOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a cleanup operation
pub struct CleanupResult {
    pub pid: Pid,
    pub stats: CleanupStats,
    pub errors: Vec<String>,
}

impl CleanupResult {
    /// Check if cleanup was successful
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if cleanup had any effect
    pub fn has_freed_resources(&self) -> bool {
        self.stats.resources_freed > 0
    }
}

impl fmt::Display for CleanupResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PID {} cleanup: {} resources freed, {} bytes freed, {} errors",
            self.pid,
            self.stats.resources_freed,
            self.stats.bytes_freed,
            self.errors.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestResource {
        name: &'static str,
        cleanup_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl ResourceCleanup for TestResource {
        fn cleanup(&self, _pid: Pid) -> CleanupStats {
            self.cleanup_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            CleanupStats {
                resources_freed: 1,
                bytes_freed: 100,
                errors_encountered: 0,
            }
        }

        fn resource_type(&self) -> &'static str {
            self.name
        }

        fn has_resources(&self, _pid: Pid) -> bool {
            true
        }
    }

    #[test]
    fn test_orchestrator_cleanup_order() {
        use std::sync::Arc;
        use std::sync::atomic::AtomicUsize;

        let r1_count = Arc::new(AtomicUsize::new(0));
        let r2_count = Arc::new(AtomicUsize::new(0));

        let orchestrator = ResourceOrchestrator::new()
            .register(TestResource {
                name: "resource1",
                cleanup_count: r1_count.clone(),
            })
            .register(TestResource {
                name: "resource2",
                cleanup_count: r2_count.clone(),
            });

        let result = orchestrator.cleanup_process(1);

        assert!(result.is_success());
        assert_eq!(result.stats.resources_freed, 2);
        assert_eq!(r1_count.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(r2_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
