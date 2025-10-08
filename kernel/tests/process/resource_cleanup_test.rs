/*!
 * Resource Cleanup Tests
 * Comprehensive tests for per-process resource tracking and cleanup
 */

use ai_os_kernel::core::types::Pid;
use ai_os_kernel::process::resources::{
    CleanupStats, ResourceCleanup, ResourceOrchestrator,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Mock resource for testing
struct MockResource {
    name: &'static str,
    cleanup_counter: Arc<AtomicUsize>,
    has_resources_value: bool,
}

impl MockResource {
    fn new(name: &'static str, has_resources: bool) -> Self {
        Self {
            name,
            cleanup_counter: Arc::new(AtomicUsize::new(0)),
            has_resources_value: has_resources,
        }
    }

    fn cleanup_count(&self) -> usize {
        self.cleanup_counter.load(Ordering::SeqCst)
    }
}

impl ResourceCleanup for MockResource {
    fn cleanup(&self, _pid: Pid) -> CleanupStats {
        self.cleanup_counter.fetch_add(1, Ordering::SeqCst);
        CleanupStats {
            resources_freed: 5,
            bytes_freed: 1024,
            errors_encountered: 0,
        }
    }

    fn resource_type(&self) -> &'static str {
        self.name
    }

    fn has_resources(&self, _pid: Pid) -> bool {
        self.has_resources_value
    }
}

#[test]
fn test_orchestrator_basic_cleanup() {
    let resource1 = Arc::new(MockResource::new("resource1", true));
    let resource2 = Arc::new(MockResource::new("resource2", true));

    let orchestrator = ResourceOrchestrator::new()
        .register(resource1.clone())
        .register(resource2.clone());

    let result = orchestrator.cleanup_process(1);

    // Both resources should be cleaned
    assert_eq!(resource1.cleanup_count(), 1);
    assert_eq!(resource2.cleanup_count(), 1);

    // Stats should be aggregated
    assert_eq!(result.stats.resources_freed, 10); // 5 + 5
    assert_eq!(result.stats.bytes_freed, 2048); // 1024 + 1024

    assert!(result.is_success());
    assert!(result.has_freed_resources());
}

#[test]
fn test_orchestrator_lifo_order() {
    let mut cleanup_order = Vec::new();
    let order_tracker = Arc::new(std::sync::Mutex::new(&mut cleanup_order));

    struct OrderedResource {
        name: String,
        order: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl ResourceCleanup for OrderedResource {
        fn cleanup(&self, _pid: Pid) -> CleanupStats {
            let mut order = self.order.lock().unwrap();
            order.push(self.name.clone());
            CleanupStats::default()
        }

        fn resource_type(&self) -> &'static str {
            "ordered"
        }

        fn has_resources(&self, _pid: Pid) -> bool {
            true
        }
    }

    let order = Arc::new(std::sync::Mutex::new(Vec::new()));

    let orchestrator = ResourceOrchestrator::new()
        .register(OrderedResource {
            name: "first".to_string(),
            order: order.clone(),
        })
        .register(OrderedResource {
            name: "second".to_string(),
            order: order.clone(),
        })
        .register(OrderedResource {
            name: "third".to_string(),
            order: order.clone(),
        });

    orchestrator.cleanup_process(1);

    // Should cleanup in LIFO order (reverse of registration)
    let final_order = order.lock().unwrap();
    assert_eq!(*final_order, vec!["third", "second", "first"]);
}

#[test]
fn test_orchestrator_skips_empty_resources() {
    let resource_with = Arc::new(MockResource::new("with_resources", true));
    let resource_without = Arc::new(MockResource::new("without_resources", false));

    let orchestrator = ResourceOrchestrator::new()
        .register(resource_with.clone())
        .register(resource_without.clone());

    orchestrator.cleanup_process(1);

    // Only resource with resources should be cleaned
    assert_eq!(resource_with.cleanup_count(), 1);
    assert_eq!(resource_without.cleanup_count(), 0);
}

#[test]
fn test_orchestrator_handles_errors() {
    struct ErrorResource;

    impl ResourceCleanup for ErrorResource {
        fn cleanup(&self, _pid: Pid) -> CleanupStats {
            CleanupStats {
                resources_freed: 0,
                bytes_freed: 0,
                errors_encountered: 3,
            }
        }

        fn resource_type(&self) -> &'static str {
            "error_resource"
        }

        fn has_resources(&self, _pid: Pid) -> bool {
            true
        }
    }

    let orchestrator = ResourceOrchestrator::new().register(ErrorResource);

    let result = orchestrator.cleanup_process(1);

    assert!(!result.is_success()); // Has errors
    assert_eq!(result.stats.errors_encountered, 3);
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn test_orchestrator_multiple_processes() {
    let resource = Arc::new(MockResource::new("multi_process", true));

    let orchestrator = ResourceOrchestrator::new().register(resource.clone());

    // Cleanup multiple processes
    orchestrator.cleanup_process(1);
    orchestrator.cleanup_process(2);
    orchestrator.cleanup_process(3);

    // Should have been called 3 times
    assert_eq!(resource.cleanup_count(), 3);
}

#[test]
fn test_cleanup_result_display() {
    let orchestrator = ResourceOrchestrator::new()
        .register(MockResource::new("test", true));

    let result = orchestrator.cleanup_process(42);

    let display = format!("{}", result);
    assert!(display.contains("PID 42"));
    assert!(display.contains("5 resources freed"));
    assert!(display.contains("1024 bytes freed"));
}

#[test]
fn test_orchestrator_empty() {
    let orchestrator = ResourceOrchestrator::new();

    let result = orchestrator.cleanup_process(1);

    assert!(result.is_success());
    assert!(!result.has_freed_resources());
    assert_eq!(result.stats.resources_freed, 0);
}

#[test]
fn test_cleanup_stats_aggregation() {
    let mut stats = CleanupStats::default();

    stats.merge(CleanupStats {
        resources_freed: 10,
        bytes_freed: 2048,
        errors_encountered: 2,
    });

    stats.merge(CleanupStats {
        resources_freed: 5,
        bytes_freed: 1024,
        errors_encountered: 1,
    });

    assert_eq!(stats.resources_freed, 15);
    assert_eq!(stats.bytes_freed, 3072);
    assert_eq!(stats.errors_encountered, 3);
}

#[test]
fn test_resource_count() {
    let orchestrator = ResourceOrchestrator::new()
        .register(MockResource::new("r1", true))
        .register(MockResource::new("r2", true))
        .register(MockResource::new("r3", true));

    assert_eq!(orchestrator.resource_count(), 3);
}
