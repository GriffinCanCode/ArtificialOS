/*!
 * Collector Tests
 */

use ai_os_kernel::monitoring::{
    Category, Collector, CommonQueries, Event, Payload, Query, Severity,
};

#[test]
fn test_collector_basic() {
    let collector = Collector::new();

    // Emit some events
    collector.process_created(100, "test_proc".to_string(), 5);
    collector.process_terminated(100, Some(0));

    let stats = collector.stream_stats();
    assert!(stats.events_produced >= 2);
}

#[test]
fn test_collector_subscribe() {
    let collector = Collector::new();

    collector.process_created(123, "test".to_string(), 5);

    let mut sub = collector.subscribe();

    // Collect events
    let mut count = 0;
    while let Some(_event) = sub.next() {
        count += 1;
    }

    assert!(count > 0);
}

#[test]
fn test_collector_causality() {
    let collector = Collector::new();

    // Create causality chain
    let event1 = Event::new(
        Severity::Info,
        Category::Process,
        Payload::ProcessCreated {
            name: "parent".to_string(),
            priority: 5,
        },
    );

    let causality_id = collector.emit_causal(event1);

    // Emit related event in chain
    let event2 = Event::new(
        Severity::Info,
        Category::Memory,
        Payload::MemoryAllocated {
            size: 4096,
            region_id: 1,
        },
    );

    collector.emit_in_chain(event2, causality_id);

    // Verify chain exists
    let mut sub = collector.subscribe();
    let mut found_chain = 0;

    while let Some(event) = sub.next() {
        if event.causality_id == Some(causality_id) {
            found_chain += 1;
        }
    }

    assert_eq!(found_chain, 2);
}

#[test]
fn test_collector_query() {
    let collector = Collector::new();

    // Emit various events
    collector.process_created(100, "test1".to_string(), 5);
    collector.process_created(200, "test2".to_string(), 3);
    collector.memory_pressure(85, 512);

    let mut sub = collector.subscribe();

    // Query for process events only
    let query = Query::new().category(Category::Process);
    let result = collector.query(query, &mut sub);

    assert_eq!(result.count, 2);
}

#[test]
fn test_collector_metrics_integration() {
    let collector = Collector::new();

    // Emit syscall event (updates both events and metrics)
    collector.syscall_exit(123, "read".to_string(), 1500, true);

    let metrics = collector.metrics();
    assert!(metrics.counters.contains_key("syscall.total"));
}

#[test]
fn test_collector_convenience_methods() {
    let collector = Collector::new();

    // Test all convenience methods
    collector.process_created(100, "app".to_string(), 5);
    collector.process_terminated(100, Some(0));
    collector.syscall_exit(100, "write".to_string(), 200, true);
    collector.memory_pressure(75, 1024);
    collector.slow_operation("database_query".to_string(), 150, 100);

    let stats = collector.stream_stats();
    assert!(stats.events_produced >= 5);
}

#[test]
fn test_collector_sampling_rate() {
    let collector = Collector::new();

    let initial_rate = collector.sampling_rate();
    assert_eq!(initial_rate, 100); // Starts at 100%

    // Simulate high overhead
    collector.update_overhead(10);

    // Rate should decrease
    let new_rate = collector.sampling_rate();
    assert!(new_rate < initial_rate || new_rate == 1); // Min is 1%
}

#[test]
fn test_collector_clone() {
    let collector1 = Collector::new();
    collector1.process_created(100, "test".to_string(), 5);

    let collector2 = collector1.clone();
    collector2.process_created(200, "test2".to_string(), 3);

    // Both should share the same stream
    let stats = collector1.stream_stats();
    assert!(stats.events_produced >= 2);
}

#[test]
fn test_collector_reset() {
    let collector = Collector::new();

    collector.process_created(100, "test".to_string(), 5);

    collector.reset();

    let metrics = collector.metrics();
    assert_eq!(metrics.counters.len(), 0);
}

#[test]
fn test_common_queries() {
    let collector = Collector::new();

    // Create various events
    collector.slow_operation("slow_op".to_string(), 200, 100);
    collector.memory_pressure(85, 256);

    let mut sub = collector.subscribe();

    // Test slow operations query
    let query = CommonQueries::slow_operations(100);
    let result = collector.query(query, &mut sub);
    assert!(result.count > 0);
}

#[test]
fn test_collector_resource_cleanup() {
    let collector = Collector::new();

    let by_type = std::collections::HashMap::new();
    collector.resource_cleanup(123, 10, 4096, 500, by_type, vec![]);

    let stats = collector.stream_stats();
    assert!(stats.events_produced >= 1);

    // Verify event was emitted and metrics were updated
    let metrics = collector.metrics();
    // The cleanup method updates these specific metrics:
    assert!(
        metrics.counters.contains_key("resource.unified.freed")
            || metrics
                .histograms
                .contains_key("resource.cleanup_duration_ms")
    );
}

#[test]
fn test_collector_with_errors() {
    let collector = Collector::new();

    let by_type = std::collections::HashMap::new();
    let errors = vec!["Failed to close fd".to_string()];

    collector.resource_cleanup(123, 5, 1024, 300, by_type, errors);

    // Should emit both reclaim and error events
    let mut sub = collector.subscribe();
    let mut event_count = 0;
    while let Some(_) = sub.next() {
        event_count += 1;
    }

    assert!(event_count >= 2); // At least reclaim + error event
}
