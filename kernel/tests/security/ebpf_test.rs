/*!
 * eBPF Module Tests
 */

use ai_os_kernel::security::ebpf::*;

#[test]
fn test_ebpf_manager_init() {
    let manager = EbpfManagerImpl::with_simulation();
    assert!(manager.is_supported());
    assert_eq!(manager.platform(), EbpfPlatform::Simulation);
}

#[test]
fn test_program_lifecycle() {
    let manager = EbpfManagerImpl::with_simulation();

    let config = ProgramConfig {
        name: "test_prog".to_string(),
        program_type: ProgramType::SyscallEntry,
        auto_attach: false,
        enabled: true,
    };

    // Load program
    assert!(manager.load_program(config).is_ok());
    assert_eq!(manager.list_programs().len(), 1);

    // Get info
    let info = manager.get_program_info("test_prog");
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.name, "test_prog");
    assert!(!info.attached);

    // Attach program
    assert!(manager.attach_program("test_prog").is_ok());
    let info = manager.get_program_info("test_prog").unwrap();
    assert!(info.attached);

    // Detach program
    assert!(manager.detach_program("test_prog").is_ok());
    let info = manager.get_program_info("test_prog").unwrap();
    assert!(!info.attached);

    // Unload program
    assert!(manager.unload_program("test_prog").is_ok());
    assert_eq!(manager.list_programs().len(), 0);
}

#[test]
fn test_multiple_programs() {
    let manager = EbpfManagerImpl::with_simulation();

    let programs = vec![
        ("syscall_entry", ProgramType::SyscallEntry),
        ("syscall_exit", ProgramType::SyscallExit),
        ("network", ProgramType::NetworkSocket),
        ("file_ops", ProgramType::FileOps),
    ];

    for (name, ptype) in programs {
        let config = ProgramConfig {
            name: name.to_string(),
            program_type: ptype,
            auto_attach: true,
            enabled: true,
        };
        assert!(manager.load_program(config).is_ok());
    }

    assert_eq!(manager.list_programs().len(), 4);

    let stats = manager.stats();
    assert_eq!(stats.programs_loaded, 4);
    assert_eq!(stats.programs_attached, 4);
}

#[test]
fn test_filter_management() {
    let manager = EbpfManagerImpl::with_simulation();

    // Add filter
    let filter = SyscallFilter {
        id: "test_filter".to_string(),
        pid: Some(100),
        syscall_nrs: Some(vec![1, 2, 3]),
        action: FilterAction::Deny,
        priority: 100,
    };
    assert!(manager.add_filter(filter).is_ok());

    // Get filters
    let filters = manager.get_filters();
    assert_eq!(filters.len(), 1);
    assert_eq!(filters[0].id, "test_filter");

    // Remove filter
    assert!(manager.remove_filter("test_filter").is_ok());
    assert_eq!(manager.get_filters().len(), 0);
}

#[test]
fn test_filter_priority() {
    let manager = EbpfManagerImpl::with_simulation();

    // Add filters with different priorities
    for i in 0..3 {
        let filter = SyscallFilter {
            id: format!("filter_{}", i),
            pid: None,
            syscall_nrs: None,
            action: FilterAction::Allow,
            priority: i * 10,
        };
        manager.add_filter(filter).unwrap();
    }

    let filters = manager.get_filters();
    assert_eq!(filters.len(), 3);

    // Should be sorted by priority (highest first)
    assert!(filters[0].priority >= filters[1].priority);
    assert!(filters[1].priority >= filters[2].priority);
}

#[test]
fn test_syscall_checking() {
    let manager = EbpfManagerImpl::with_simulation();

    // No filters - should allow everything
    assert!(manager.check_syscall(100, 1));
    assert!(manager.check_syscall(100, 2));

    // Add deny filter for specific syscall and PID
    let filter = SyscallFilter {
        id: "deny_write".to_string(),
        pid: Some(100),
        syscall_nrs: Some(vec![1]),
        action: FilterAction::Deny,
        priority: 100,
    };
    manager.add_filter(filter).unwrap();

    // Should deny
    assert!(!manager.check_syscall(100, 1));

    // Different PID should allow
    assert!(manager.check_syscall(200, 1));

    // Different syscall should allow
    assert!(manager.check_syscall(100, 2));
}

#[test]
fn test_filter_actions() {
    let manager = EbpfManagerImpl::with_simulation();

    // Test allow action
    let filter = SyscallFilter {
        id: "allow".to_string(),
        pid: Some(100),
        syscall_nrs: Some(vec![1]),
        action: FilterAction::Allow,
        priority: 100,
    };
    manager.add_filter(filter).unwrap();
    assert!(manager.check_syscall(100, 1));

    manager.clear_filters().unwrap();

    // Test deny action
    let filter = SyscallFilter {
        id: "deny".to_string(),
        pid: Some(100),
        syscall_nrs: Some(vec![1]),
        action: FilterAction::Deny,
        priority: 100,
    };
    manager.add_filter(filter).unwrap();
    assert!(!manager.check_syscall(100, 1));

    manager.clear_filters().unwrap();

    // Test log action (allows but logs)
    let filter = SyscallFilter {
        id: "log".to_string(),
        pid: Some(100),
        syscall_nrs: Some(vec![1]),
        action: FilterAction::Log,
        priority: 100,
    };
    manager.add_filter(filter).unwrap();
    assert!(manager.check_syscall(100, 1));
}

#[test]
fn test_process_monitoring() {
    let manager = EbpfManagerImpl::with_simulation();

    // Monitor processes
    assert!(manager.monitor_process(100).is_ok());
    assert!(manager.monitor_process(200).is_ok());
    assert!(manager.monitor_process(300).is_ok());

    let pids = manager.get_monitored_pids();
    assert_eq!(pids.len(), 3);
    assert!(pids.contains(&100));
    assert!(pids.contains(&200));
    assert!(pids.contains(&300));

    // Unmonitor a process
    assert!(manager.unmonitor_process(200).is_ok());
    let pids = manager.get_monitored_pids();
    assert_eq!(pids.len(), 2);
    assert!(!pids.contains(&200));
}

#[test]
fn test_syscall_counting() {
    let manager = EbpfManagerImpl::with_simulation();

    manager.monitor_process(100).unwrap();

    // Initially zero
    assert_eq!(manager.get_syscall_count(100), 0);

    // Unmonitored process should return zero
    assert_eq!(manager.get_syscall_count(999), 0);
}

#[test]
fn test_event_retrieval() {
    let manager = EbpfManagerImpl::with_simulation();

    // Get recent events (empty initially in simulation)
    let events = manager.get_recent_events(10);
    assert!(events.is_empty() || events.len() <= 10);

    // Get events by PID (empty initially in simulation)
    let events = manager.get_events_by_pid(100, 10);
    assert!(events.is_empty() || events.len() <= 10);
}

#[test]
fn test_subscription() {
    let manager = EbpfManagerImpl::with_simulation();

    // Subscribe to syscall events
    let sub_id = manager.subscribe_syscall(Box::new(|_event| {
        // Callback
    }));
    assert!(sub_id.is_ok());

    // Subscribe to network events
    let sub_id = manager.subscribe_network(Box::new(|_event| {
        // Callback
    }));
    assert!(sub_id.is_ok());

    // Subscribe to file events
    let sub_id = manager.subscribe_file(Box::new(|_event| {
        // Callback
    }));
    assert!(sub_id.is_ok());

    // Subscribe to all events
    let sub_id = manager.subscribe_all(Box::new(|_event| {
        // Callback
    }));
    assert!(sub_id.is_ok());
    let sub_id = sub_id.unwrap();

    // Unsubscribe
    assert!(manager.unsubscribe(&sub_id).is_ok());
}

#[test]
fn test_statistics() {
    let manager = EbpfManagerImpl::with_simulation();

    // Load some programs
    for i in 0..3 {
        let config = ProgramConfig {
            name: format!("prog_{}", i),
            program_type: ProgramType::SyscallEntry,
            auto_attach: true,
            enabled: true,
        };
        manager.load_program(config).unwrap();
    }

    // Add some filters
    for i in 0..2 {
        let filter = SyscallFilter {
            id: format!("filter_{}", i),
            pid: None,
            syscall_nrs: None,
            action: FilterAction::Allow,
            priority: 100,
        };
        manager.add_filter(filter).unwrap();
    }

    let stats = manager.stats();
    assert_eq!(stats.programs_loaded, 3);
    assert_eq!(stats.programs_attached, 3);
    assert_eq!(stats.active_filters, 2);
    assert_eq!(stats.platform, EbpfPlatform::Simulation);
}

#[test]
fn test_manager_lifecycle() {
    let manager = EbpfManagerImpl::with_simulation();

    // Initialize
    assert!(manager.init().is_ok());
    assert!(manager.health_check());

    // Shutdown
    assert!(manager.shutdown().is_ok());
}

#[test]
fn test_clear_filters() {
    let manager = EbpfManagerImpl::with_simulation();

    // Add multiple filters
    for i in 0..5 {
        let filter = SyscallFilter {
            id: format!("filter_{}", i),
            pid: None,
            syscall_nrs: None,
            action: FilterAction::Allow,
            priority: 100,
        };
        manager.add_filter(filter).unwrap();
    }

    assert_eq!(manager.get_filters().len(), 5);

    // Clear all
    assert!(manager.clear_filters().is_ok());
    assert_eq!(manager.get_filters().len(), 0);
}

#[test]
fn test_syscall_name_mapping() {
    use ai_os_kernel::security::ebpf::syscall_name;

    assert_eq!(syscall_name(0), Some("read"));
    assert_eq!(syscall_name(1), Some("write"));
    assert_eq!(syscall_name(2), Some("open"));
    assert_eq!(syscall_name(41), Some("socket"));
    assert_eq!(syscall_name(42), Some("connect"));
    assert_eq!(syscall_name(999), None);
}

#[test]
fn test_integrated_monitor() {
    let ebpf = EbpfManagerImpl::with_simulation();
    let monitor = IntegratedEbpfMonitor::new(ebpf);

    assert!(monitor.init().is_ok());

    // Monitor a process
    assert!(monitor.monitor_process(123).is_ok());

    // Get stats
    let stats = monitor.get_process_stats(123);
    assert_eq!(stats.pid, 123);

    // Unmonitor
    assert!(monitor.unmonitor_process(123).is_ok());

    assert!(monitor.shutdown().is_ok());
}
