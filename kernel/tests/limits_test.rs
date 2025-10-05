/*!
 * Resource Limits Tests
 * Tests for OS-level resource limit enforcement
 */

use ai_os_kernel::limits::{LimitManager, Limits};

#[test]
fn test_limits_builder() {
    let limits = Limits::new()
        .with_memory(256 * 1024 * 1024)
        .with_cpu_shares(150)
        .with_max_pids(20);

    assert_eq!(limits.memory_bytes, Some(256 * 1024 * 1024));
    assert_eq!(limits.cpu_shares, Some(150));
    assert_eq!(limits.max_pids, Some(20));
}

#[test]
fn test_default_limits() {
    let limits = Limits::default();

    assert_eq!(limits.memory_bytes, Some(512 * 1024 * 1024));
    assert_eq!(limits.cpu_shares, Some(100));
    assert_eq!(limits.max_pids, Some(10));
}

#[test]
fn test_limit_manager_creation() {
    let manager = LimitManager::new();
    assert!(manager.is_ok());
}

#[test]
#[cfg(not(target_os = "linux"))]
fn test_apply_limits_non_linux() {
    let manager = LimitManager::new().unwrap();
    let limits = Limits::default();

    // Should not fail on non-Linux platforms
    let result = manager.apply(12345, &limits);
    assert!(result.is_ok());
}

#[test]
#[cfg(not(target_os = "linux"))]
fn test_remove_limits_non_linux() {
    let manager = LimitManager::new().unwrap();

    // Should not fail on non-Linux platforms
    let result = manager.remove(12345);
    assert!(result.is_ok());
}

#[test]
fn test_custom_memory_limits() {
    let limits = Limits::new().with_memory(1024 * 1024 * 1024); // 1 GB

    assert_eq!(limits.memory_bytes, Some(1024 * 1024 * 1024));
}

#[test]
fn test_cpu_shares_range() {
    let low_priority = Limits::new().with_cpu_shares(50);
    let high_priority = Limits::new().with_cpu_shares(500);

    assert_eq!(low_priority.cpu_shares, Some(50));
    assert_eq!(high_priority.cpu_shares, Some(500));
}
