/*!
 * Event System Tests
 */

use ai_os_kernel::monitoring::{Category, Event, EventFilter, Payload, Severity};
use std::time::Duration;

#[test]
fn test_event_creation() {
    let event = Event::new(
        Severity::Info,
        Category::Process,
        Payload::ProcessCreated {
            name: "test_process".to_string(),
            priority: 5,
        },
    );

    assert_eq!(event.severity, Severity::Info);
    assert_eq!(event.category, Category::Process);
    assert!(event.causality_id.is_none());
    assert!(event.pid.is_none());
}

#[test]
fn test_event_with_context() {
    let event = Event::new(
        Severity::Warn,
        Category::Memory,
        Payload::MemoryPressure {
            usage_pct: 85,
            available_mb: 512,
        },
    )
    .with_pid(123)
    .with_causality(456);

    assert_eq!(event.pid, Some(123));
    assert_eq!(event.causality_id, Some(456));
}

#[test]
fn test_event_filter_severity() {
    let event = Event::new(
        Severity::Warn,
        Category::Process,
        Payload::ProcessCreated {
            name: "test".to_string(),
            priority: 5,
        },
    );

    let filter_pass = EventFilter::new().severity(Severity::Info);
    assert!(event.matches(&filter_pass));

    let filter_fail = EventFilter::new().severity(Severity::Error);
    assert!(!event.matches(&filter_fail));
}

#[test]
fn test_event_filter_category() {
    let event = Event::new(
        Severity::Info,
        Category::Memory,
        Payload::MemoryAllocated {
            size: 1024,
            region_id: 1,
        },
    );

    let filter_pass = EventFilter::new().category(Category::Memory);
    assert!(event.matches(&filter_pass));

    let filter_fail = EventFilter::new().category(Category::Process);
    assert!(!event.matches(&filter_fail));
}

#[test]
fn test_event_filter_pid() {
    let event = Event::new(
        Severity::Info,
        Category::Process,
        Payload::ProcessCreated {
            name: "test".to_string(),
            priority: 5,
        },
    )
    .with_pid(100);

    let filter_pass = EventFilter::new().pid(100);
    assert!(event.matches(&filter_pass));

    let filter_fail = EventFilter::new().pid(200);
    assert!(!event.matches(&filter_fail));
}

#[test]
fn test_event_filter_combined() {
    let event = Event::new(
        Severity::Error,
        Category::Syscall,
        Payload::SyscallExit {
            name: "read".to_string(),
            duration_us: 1000,
            result: ai_os_kernel::monitoring::SyscallResult::Error,
        },
    )
    .with_pid(123);

    let filter = EventFilter::new()
        .severity(Severity::Error)
        .category(Category::Syscall)
        .pid(123);

    assert!(event.matches(&filter));
}

#[test]
fn test_event_age() {
    let event = Event::new(
        Severity::Info,
        Category::Process,
        Payload::ProcessCreated {
            name: "test".to_string(),
            priority: 5,
        },
    );

    std::thread::sleep(Duration::from_millis(10));

    let age = event.age();
    assert!(age.as_millis() >= 10);
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Critical > Severity::Error);
    assert!(Severity::Error > Severity::Warn);
    assert!(Severity::Warn > Severity::Info);
    assert!(Severity::Info > Severity::Debug);
    assert!(Severity::Debug > Severity::Trace);
}

#[test]
fn test_syscall_event_variants() {
    let enter = Payload::SyscallEnter {
        name: "write".to_string(),
        args_hash: 12345,
    };

    let exit = Payload::SyscallExit {
        name: "write".to_string(),
        duration_us: 150,
        result: ai_os_kernel::monitoring::SyscallResult::Success,
    };

    let slow = Payload::SyscallSlow {
        name: "write".to_string(),
        duration_ms: 50,
        threshold_ms: 10,
    };

    // Ensure they can be created
    let _e1 = Event::new(Severity::Debug, Category::Syscall, enter);
    let _e2 = Event::new(Severity::Debug, Category::Syscall, exit);
    let _e3 = Event::new(Severity::Warn, Category::Syscall, slow);
}

#[test]
fn test_memory_event_variants() {
    let allocated = Payload::MemoryAllocated {
        size: 4096,
        region_id: 42,
    };

    let freed = Payload::MemoryFreed {
        size: 4096,
        region_id: 42,
    };

    let pressure = Payload::MemoryPressure {
        usage_pct: 90,
        available_mb: 128,
    };

    let _e1 = Event::new(Severity::Debug, Category::Memory, allocated);
    let _e2 = Event::new(Severity::Debug, Category::Memory, freed);
    let _e3 = Event::new(Severity::Warn, Category::Memory, pressure);
}
