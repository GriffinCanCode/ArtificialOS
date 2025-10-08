/*!
 * eBPF Event Collector Unit Tests
 * Tests for event collection and distribution
 */

use ai_os_kernel::security::ebpf::{
    EbpfEvent, EventCollector, EventType, SyscallEvent, SyscallEventType,
};
use std::sync::{Arc, Mutex};

#[test]
fn test_event_emit_and_recent() {
    let collector = EventCollector::new();

    let event = EbpfEvent::Syscall(SyscallEvent {
        pid: 100,
        syscall_nr: 0,
        name: Some("read".to_string()),
        args: [0, 0, 0, 0, 0, 0],
        ret: Some(0),
        timestamp_ns: 1234567890,
        event_type: SyscallEventType::Enter,
        tid: 100,
        uid: 1000,
        gid: 1000,
        cpu: 0,
    });

    collector.emit(event);

    let recent = collector.recent(10);
    assert_eq!(recent.len(), 1);
}

#[test]
fn test_event_history_limit() {
    let collector = EventCollector::new();

    // Emit more than MAX_EVENT_HISTORY events
    for i in 0..11000 {
        let event = EbpfEvent::Syscall(SyscallEvent {
            pid: 100,
            syscall_nr: 0,
            name: Some("read".to_string()),
            args: [0, 0, 0, 0, 0, 0],
            ret: Some(0),
            timestamp_ns: i as u64,
            event_type: SyscallEventType::Enter,
            tid: 100,
            uid: 1000,
            gid: 1000,
            cpu: 0,
        });
        collector.emit(event);
    }

    let recent = collector.recent(usize::MAX);
    // Should be capped at MAX_EVENT_HISTORY (10000)
    assert!(recent.len() <= 10000);
}

#[test]
fn test_event_by_pid() {
    let collector = EventCollector::new();

    // Emit events for different PIDs
    for pid in 100..105 {
        for _ in 0..5 {
            let event = EbpfEvent::Syscall(SyscallEvent {
                pid,
                syscall_nr: 0,
                name: Some("read".to_string()),
                args: [0, 0, 0, 0, 0, 0],
                ret: Some(0),
                timestamp_ns: 1234567890,
                event_type: SyscallEventType::Enter,
                tid: pid,
                uid: 1000,
                gid: 1000,
                cpu: 0,
            });
            collector.emit(event);
        }
    }

    // Get events for specific PID
    let pid_events = collector.by_pid(102, 10);
    assert_eq!(pid_events.len(), 5);

    // All events should be for the requested PID
    for event in pid_events {
        assert_eq!(event.pid(), 102);
    }
}

#[test]
fn test_event_subscription_syscall() {
    let collector = EventCollector::new();
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);

    let subscription_id = collector.subscribe(
        EventType::Syscall,
        Box::new(move |event| {
            received_clone.lock().unwrap().push(event);
        }),
    );

    // Emit syscall event
    let event = EbpfEvent::Syscall(SyscallEvent {
        pid: 100,
        syscall_nr: 0,
        name: Some("read".to_string()),
        args: [0, 0, 0, 0, 0, 0],
        ret: Some(0),
        timestamp_ns: 1234567890,
        event_type: SyscallEventType::Enter,
        tid: 100,
        uid: 1000,
        gid: 1000,
        cpu: 0,
    });

    collector.emit(event.clone());

    // Check callback was invoked
    let received_events = received.lock().unwrap();
    assert_eq!(received_events.len(), 1);

    collector.unsubscribe(&subscription_id).unwrap();
}

#[test]
fn test_event_subscription_unsubscribe() {
    let collector = EventCollector::new();
    let received = Arc::new(Mutex::new(0));
    let received_clone = Arc::clone(&received);

    let subscription_id = collector.subscribe(
        EventType::All,
        Box::new(move |_| {
            *received_clone.lock().unwrap() += 1;
        }),
    );

    // Emit event
    let event = EbpfEvent::Syscall(SyscallEvent {
        pid: 100,
        syscall_nr: 0,
        name: Some("read".to_string()),
        args: [0, 0, 0, 0, 0, 0],
        ret: Some(0),
        timestamp_ns: 1234567890,
        event_type: SyscallEventType::Enter,
        tid: 100,
        uid: 1000,
        gid: 1000,
        cpu: 0,
    });

    collector.emit(event.clone());
    assert_eq!(*received.lock().unwrap(), 1);

    // Unsubscribe
    collector.unsubscribe(&subscription_id).unwrap();

    // Emit another event (should not be received)
    collector.emit(event);
    assert_eq!(*received.lock().unwrap(), 1);
}

#[test]
fn test_event_clear() {
    let collector = EventCollector::new();

    // Emit some events
    for _ in 0..10 {
        let event = EbpfEvent::Syscall(SyscallEvent {
            pid: 100,
            syscall_nr: 0,
            name: Some("read".to_string()),
            args: [0, 0, 0, 0, 0, 0],
            ret: Some(0),
            timestamp_ns: 1234567890,
            event_type: SyscallEventType::Enter,
            tid: 100,
            uid: 1000,
            gid: 1000,
            cpu: 0,
        });
        collector.emit(event);
    }

    assert_eq!(collector.recent(100).len(), 10);

    collector.clear();
    assert_eq!(collector.recent(100).len(), 0);
}

#[test]
fn test_event_stats() {
    let collector = EventCollector::new();

    // Emit syscall events
    for _ in 0..5 {
        let event = EbpfEvent::Syscall(SyscallEvent {
            pid: 100,
            syscall_nr: 0,
            name: Some("read".to_string()),
            args: [0, 0, 0, 0, 0, 0],
            ret: Some(0),
            timestamp_ns: 1234567890,
            event_type: SyscallEventType::Enter,
            tid: 100,
            uid: 1000,
            gid: 1000,
            cpu: 0,
        });
        collector.emit(event);
    }

    let (syscall_events, _network_events, _file_events, _events_per_sec) = collector.stats();
    assert_eq!(syscall_events, 5);
}
