/*!
 * Query System Tests
 */

use ai_os_kernel::monitoring::{
    AggregationType, CausalityTracer, Category, CommonQueries, Event, Payload, Query, Severity, SyscallResult,
};
use std::time::Duration;

fn create_test_events() -> Vec<Event> {
    vec![
        Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "app1".to_string(),
                priority: 5,
            },
        )
        .with_pid(100),
        Event::new(
            Severity::Warn,
            Category::Memory,
            Payload::MemoryPressure {
                usage_pct: 85,
                available_mb: 256,
            },
        )
        .with_pid(100),
        Event::new(
            Severity::Error,
            Category::Syscall,
            Payload::SyscallExit {
                name: "read".to_string(),
                duration_us: 5000,
                result: SyscallResult::Error,
            },
        )
        .with_pid(200),
    ]
}

#[test]
fn test_query_basic() {
    let events = create_test_events();

    let result = Query::new().execute(&events);

    assert_eq!(result.count, 3);
    assert_eq!(result.events.len(), 3);
}

#[test]
fn test_query_filter_severity() {
    let events = create_test_events();

    let result = Query::new().severity(Severity::Warn).execute(&events);

    // Should match Warn and Error (2 events)
    assert_eq!(result.count, 2);
}

#[test]
fn test_query_filter_category() {
    let events = create_test_events();

    let result = Query::new().category(Category::Memory).execute(&events);

    assert_eq!(result.count, 1);
}

#[test]
fn test_query_filter_pid() {
    let events = create_test_events();

    let result = Query::new().pid(100).execute(&events);

    assert_eq!(result.count, 2);
}

#[test]
fn test_query_combined_filters() {
    let events = create_test_events();

    let result = Query::new()
        .severity(Severity::Warn)
        .category(Category::Memory)
        .pid(100)
        .execute(&events);

    assert_eq!(result.count, 1);
}

#[test]
fn test_query_limit() {
    let events = create_test_events();

    let result = Query::new().limit(2).execute(&events);

    assert_eq!(result.count, 2);
    assert_eq!(result.events.len(), 2);
}

#[test]
fn test_query_aggregation_by_category() {
    let events = create_test_events();

    let result = Query::new()
        .aggregate(AggregationType::CountByCategory)
        .execute(&events);

    assert!(result.aggregations.contains_key("by_category"));
}

#[test]
fn test_query_aggregation_by_severity() {
    let events = create_test_events();

    let result = Query::new()
        .aggregate(AggregationType::CountBySeverity)
        .execute(&events);

    assert!(result.aggregations.contains_key("by_severity"));
}

#[test]
fn test_query_aggregation_by_pid() {
    let events = create_test_events();

    let result = Query::new()
        .aggregate(AggregationType::CountByPid)
        .execute(&events);

    assert!(result.aggregations.contains_key("by_pid"));
}

#[test]
fn test_query_duration_stats() {
    let events = vec![
        Event::new(
            Severity::Debug,
            Category::Syscall,
            Payload::SyscallExit {
                name: "read".to_string(),
                duration_us: 100,
                result: SyscallResult::Success,
            },
        ),
        Event::new(
            Severity::Debug,
            Category::Syscall,
            Payload::SyscallExit {
                name: "write".to_string(),
                duration_us: 200,
                result: SyscallResult::Success,
            },
        ),
        Event::new(
            Severity::Warn,
            Category::Syscall,
            Payload::SyscallSlow {
                name: "fsync".to_string(),
                duration_ms: 50,
                threshold_ms: 10,
            },
        ),
    ];

    let result = Query::new()
        .aggregate(AggregationType::DurationStats)
        .execute(&events);

    assert!(result.aggregations.contains_key("duration_stats"));
}

#[test]
fn test_causality_tracing() {
    let events = vec![
        Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "app".to_string(),
                priority: 5,
            },
        )
        .with_causality(123),
        Event::new(
            Severity::Info,
            Category::Memory,
            Payload::MemoryAllocated {
                size: 4096,
                region_id: 1,
            },
        )
        .with_causality(123),
        Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "other".to_string(),
                priority: 3,
            },
        )
        .with_causality(456),
    ];

    let chain = CausalityTracer::trace(&events, 123);
    assert_eq!(chain.len(), 2);
}

#[test]
fn test_causality_root_cause() {
    let events = vec![
        Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "root".to_string(),
                priority: 5,
            },
        )
        .with_causality(789),
        Event::new(
            Severity::Warn,
            Category::Memory,
            Payload::MemoryPressure {
                usage_pct: 90,
                available_mb: 128,
            },
        )
        .with_causality(789),
    ];

    let root = CausalityTracer::root_cause(&events, 789);
    assert!(root.is_some());

    let root_event = root.unwrap();
    assert_eq!(root_event.category, Category::Process);
}

#[test]
fn test_causality_timeline() {
    let mut events = vec![
        Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "app".to_string(),
                priority: 5,
            },
        )
        .with_causality(999),
    ];

    std::thread::sleep(Duration::from_millis(10));

    events.push(
        Event::new(
            Severity::Info,
            Category::Memory,
            Payload::MemoryAllocated {
                size: 1024,
                region_id: 1,
            },
        )
        .with_causality(999),
    );

    let timeline = CausalityTracer::timeline(&events, 999);
    assert_eq!(timeline.len(), 2);

    // First event should be at offset 0
    assert_eq!(timeline[0].0, Duration::ZERO);

    // Second event should have non-zero offset
    assert!(timeline[1].0 > Duration::ZERO);
}

#[test]
fn test_common_queries_creation() {
    // Just verify they can be created
    let _q1 = CommonQueries::slow_operations(100);
    let _q2 = CommonQueries::process_errors(123);
    let _q3 = CommonQueries::health_check();
    let _q4 = CommonQueries::memory_pressure();
    let _q5 = CommonQueries::syscall_performance();
    let _q6 = CommonQueries::security_events();
}

#[test]
fn test_query_since() {
    let events = create_test_events();

    // Query for recent events (should match all in test)
    let result = Query::new().since(Duration::from_secs(1)).execute(&events);

    assert!(result.count > 0);
}
