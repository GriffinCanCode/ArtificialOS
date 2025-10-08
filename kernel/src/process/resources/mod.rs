/*!
 * Resource Cleanup System
 * Comprehensive per-process resource tracking and cleanup orchestration
 */

mod fds;
mod ipc;
mod mappings;
mod memory;
mod rings;
mod signals;
mod sockets;
mod tasks;

pub use fds::FdResource;
pub use ipc::IpcResource;
pub use mappings::MappingResource;
pub use memory::MemoryResource;
pub use rings::{IoUringResource, RingResource, ZeroCopyResource};
pub use signals::SignalResource;
pub use sockets::SocketResource;
pub use tasks::TaskResource;

use crate::core::types::Pid;
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

/// Resource cleanup statistics
#[derive(Debug, Clone, Default)]
pub struct CleanupStats {
    pub resources_freed: usize,
    pub bytes_freed: usize,
    pub errors_encountered: usize,
    pub cleanup_duration_micros: u64,
    pub by_type: HashMap<String, usize>,
}

impl CleanupStats {
    /// Create new stats with timing
    #[inline]
    pub fn with_timing<F>(f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        let start = Instant::now();
        let mut stats = f();
        stats.cleanup_duration_micros = start.elapsed().as_micros() as u64;
        stats
    }

    /// Merge another stats into this one
    fn merge(&mut self, other: CleanupStats) {
        self.resources_freed += other.resources_freed;
        self.bytes_freed += other.bytes_freed;
        self.errors_encountered += other.errors_encountered;
        self.cleanup_duration_micros += other.cleanup_duration_micros;

        // Merge per-type counts
        for (type_name, count) in other.by_type {
            *self.by_type.entry(type_name).or_insert(0) += count;
        }
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
///
/// Uses Arc internally for safe sharing across ProcessManager clones.
/// The orchestrator is immutable after construction, making Arc ideal.
pub struct ResourceOrchestrator {
    resources: std::sync::Arc<Vec<Box<dyn ResourceCleanup>>>,
}

impl ResourceOrchestrator {
    /// Create a new orchestrator with no resources
    pub fn new() -> Self {
        Self {
            resources: std::sync::Arc::new(Vec::new().into()),
        }
    }

    /// Register a resource for cleanup (builder pattern)
    ///
    /// Note: This consumes self and returns a new ResourceOrchestrator with
    /// the registered resource. All registration must happen during initialization
    /// before the orchestrator is shared/cloned.
    pub fn register<R: ResourceCleanup + 'static>(self, resource: R) -> Self {
        // During building phase, we should be the only owner of the Arc
        // This is safe because registration happens during initialization
        // before ProcessManager is created/cloned
        let mut resources_vec = std::sync::Arc::try_unwrap(self.resources)
            .unwrap_or_else(|_| {
                panic!("ResourceOrchestrator::register called after being shared - registration must complete during initialization")
            });

        resources_vec.push(Box::new(resource));

        Self {
            resources: std::sync::Arc::new(resources_vec),
        }
    }

    /// Execute cleanup for a terminated process
    ///
    /// Resources are cleaned up in reverse registration order to handle
    /// dependencies (e.g., close sockets before freeing memory).
    pub fn cleanup_process(&self, pid: Pid) -> CleanupResult {
        use crate::core::memory::arena::with_arena;

        let overall_start = Instant::now();
        let mut total_stats = CleanupStats::default();

        let errors = with_arena(|arena| {
            use crate::core::optimization::prefetch_read;

            let mut error_msgs = bumpalo::collections::Vec::new_in(arena);
            let resources_vec: Vec<_> = self.resources.iter().rev().collect();
            let len = resources_vec.len();

            // Cleanup in reverse order (LIFO)
            for (_i, resource) in resources_vec.into_iter().enumerate() {
                // Note: Cannot prefetch trait objects due to unknown size at compile time

                if resource.has_resources(pid) {
                    let start = Instant::now();
                    let mut stats = resource.cleanup(pid);
                    stats.cleanup_duration_micros = start.elapsed().as_micros() as u64;

                    // Track per-type counts
                    let resource_type = resource.resource_type();
                    stats
                        .by_type
                        .insert(resource_type.to_string(), stats.resources_freed);

                    // Save values before merging
                    let resources_freed = stats.resources_freed;
                    let errors_encountered = stats.errors_encountered;
                    let duration_micros = stats.cleanup_duration_micros;

                    if errors_encountered > 0 {
                        error_msgs.push(format!(
                            "{}: {} errors during cleanup",
                            resource_type, errors_encountered
                        ));
                    }

                    total_stats.merge(stats);

                    log::info!(
                        "Cleaned {} resources for PID {} (type: {}, took {}μs)",
                        resources_freed,
                        pid,
                        resource_type,
                        duration_micros
                    );
                }
            }

            error_msgs.into_iter().collect::<Vec<_>>()
        });

        total_stats.cleanup_duration_micros = overall_start.elapsed().as_micros() as u64;

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

    /// Validate that expected resource types are registered
    ///
    /// Warns if critical resource types are missing to detect potential leaks
    pub fn validate_coverage(&self, expected_types: &[&str]) {
        let registered: std::collections::HashSet<_> =
            self.resources.iter().map(|r| r.resource_type()).collect();

        for expected in expected_types {
            if !registered.contains(expected) {
                log::warn!(
                    "Resource type '{}' not registered - potential leak source!",
                    expected
                );
            }
        }
    }

    /// Get list of registered resource types
    pub fn registered_types(&self) -> Vec<&'static str> {
        self.resources.iter().map(|r| r.resource_type()).collect()
    }
}

impl Default for ResourceOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ResourceOrchestrator {
    fn clone(&self) -> Self {
        Self {
            resources: std::sync::Arc::clone(&self.resources),
        }
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
            self.cleanup_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            CleanupStats {
                resources_freed: 1,
                bytes_freed: 100,
                errors_encountered: 0,
                cleanup_duration_micros: 0,
                by_type: HashMap::new(),
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
        use std::sync::atomic::AtomicUsize;
        use std::sync::Arc;

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

    #[test]
    fn test_orchestrator_clone_preserves_resources() {
        use std::sync::atomic::AtomicUsize;
        use std::sync::Arc;

        let r1_count = Arc::new(AtomicUsize::new(0));
        let r2_count = Arc::new(AtomicUsize::new(0));

        // Build orchestrator with resources
        let orchestrator = ResourceOrchestrator::new()
            .register(TestResource {
                name: "resource1",
                cleanup_count: r1_count.clone(),
            })
            .register(TestResource {
                name: "resource2",
                cleanup_count: r2_count.clone(),
            });

        // Clone should preserve registered resources
        let cloned = orchestrator.clone();

        // Original should still work
        let result1 = orchestrator.cleanup_process(1);
        assert!(result1.is_success());
        assert_eq!(result1.stats.resources_freed, 2);
        assert_eq!(r1_count.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(r2_count.load(std::sync::atomic::Ordering::SeqCst), 1);

        // Clone should have same cleanup capabilities
        let result2 = cloned.cleanup_process(2);
        assert!(result2.is_success());
        assert_eq!(result2.stats.resources_freed, 2);
        assert_eq!(r1_count.load(std::sync::atomic::Ordering::SeqCst), 2);
        assert_eq!(r2_count.load(std::sync::atomic::Ordering::SeqCst), 2);

        // Both should have same resource count
        assert_eq!(orchestrator.resource_count(), 2);
        assert_eq!(cloned.resource_count(), 2);
    }
}
