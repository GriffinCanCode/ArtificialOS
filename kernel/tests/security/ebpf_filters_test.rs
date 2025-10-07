/*!
 * eBPF Filter Manager Unit Tests
 * Tests for syscall filtering functionality
 */

use ai_os_kernel::security::ebpf::{FilterAction, FilterManager, SyscallFilter};

#[test]
fn test_filter_add_and_list() {
    let manager = FilterManager::new();

    let filter = SyscallFilter {
        id: "test_filter_1".to_string(),
        pid: Some(1234),
        syscall_nrs: Some(vec![0, 1, 2]), // read, write, open
        action: FilterAction::Allow,
        priority: 100,
    };

    assert!(manager.add(filter.clone()).is_ok());

    let filters = manager.list();
    assert_eq!(filters.len(), 1);
    assert_eq!(filters[0].id, "test_filter_1");
}

#[test]
fn test_filter_duplicate_id() {
    let manager = FilterManager::new();

    let filter = SyscallFilter {
        id: "duplicate".to_string(),
        pid: None,
        syscall_nrs: None,
        action: FilterAction::Allow,
        priority: 50,
    };

    assert!(manager.add(filter.clone()).is_ok());
    // Adding same ID should fail
    assert!(manager.add(filter).is_err());
}

#[test]
fn test_filter_priority_ordering() {
    let manager = FilterManager::new();

    let high_priority = SyscallFilter {
        id: "high".to_string(),
        pid: None,
        syscall_nrs: None,
        action: FilterAction::Deny,
        priority: 200,
    };

    let low_priority = SyscallFilter {
        id: "low".to_string(),
        pid: None,
        syscall_nrs: None,
        action: FilterAction::Allow,
        priority: 10,
    };

    assert!(manager.add(low_priority).is_ok());
    assert!(manager.add(high_priority).is_ok());

    let filters = manager.list();
    // Should be sorted by priority (highest first)
    assert_eq!(filters[0].id, "high");
    assert_eq!(filters[1].id, "low");
}

#[test]
fn test_filter_check_allow() {
    let manager = FilterManager::new();

    let filter = SyscallFilter {
        id: "allow_read".to_string(),
        pid: Some(100),
        syscall_nrs: Some(vec![0]), // read
        action: FilterAction::Allow,
        priority: 100,
    };

    assert!(manager.add(filter).is_ok());

    // Should allow read for pid 100
    assert!(manager.check(100, 0));

    // Should allow other syscalls (default allow)
    assert!(manager.check(100, 1));

    // Should allow for other pids (default allow)
    assert!(manager.check(200, 0));
}

#[test]
fn test_filter_check_deny() {
    let manager = FilterManager::new();

    let filter = SyscallFilter {
        id: "deny_write".to_string(),
        pid: Some(100),
        syscall_nrs: Some(vec![1]), // write
        action: FilterAction::Deny,
        priority: 100,
    };

    assert!(manager.add(filter).is_ok());

    // Should deny write for pid 100
    assert!(!manager.check(100, 1));

    // Should allow other syscalls
    assert!(manager.check(100, 0));
}

#[test]
fn test_filter_remove() {
    let manager = FilterManager::new();

    let filter = SyscallFilter {
        id: "to_remove".to_string(),
        pid: None,
        syscall_nrs: None,
        action: FilterAction::Allow,
        priority: 50,
    };

    assert!(manager.add(filter).is_ok());
    assert_eq!(manager.list().len(), 1);

    assert!(manager.remove("to_remove").is_ok());
    assert_eq!(manager.list().len(), 0);

    // Removing non-existent filter should fail
    assert!(manager.remove("to_remove").is_err());
}

#[test]
fn test_filter_clear() {
    let manager = FilterManager::new();

    for i in 0..5 {
        let filter = SyscallFilter {
            id: format!("filter_{}", i),
            pid: None,
            syscall_nrs: None,
            action: FilterAction::Allow,
            priority: i * 10,
        };
        assert!(manager.add(filter).is_ok());
    }

    assert_eq!(manager.list().len(), 5);

    assert!(manager.clear().is_ok());
    assert_eq!(manager.list().len(), 0);
}

#[test]
fn test_filter_caching() {
    let manager = FilterManager::new();

    let filter = SyscallFilter {
        id: "cached".to_string(),
        pid: Some(100),
        syscall_nrs: Some(vec![0]),
        action: FilterAction::Allow,
        priority: 100,
    };

    assert!(manager.add(filter).is_ok());

    // First check (should cache)
    assert!(manager.check(100, 0));

    // Second check (should use cache)
    assert!(manager.check(100, 0));

    // Clear should invalidate cache
    assert!(manager.clear().is_ok());

    // After clear, default is allow
    assert!(manager.check(100, 0));
}
