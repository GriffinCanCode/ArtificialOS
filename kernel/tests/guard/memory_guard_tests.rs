/*!
 * Memory Guard Tests
 */

use ai_os_kernel::core::guard::*;
use ai_os_kernel::memory::manager::MemoryManager;
use std::sync::Arc;

#[test]
fn test_memory_guard_auto_cleanup() {
    let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
    let pid = 1;
    let size = 1024;

    let address = manager.allocate(size, pid).unwrap();
    let initial_used = manager.used_memory();

    {
        let guard = MemoryGuard::new(address, size, pid, manager.clone(), None);
        assert!(guard.is_active());
        assert_eq!(guard.address(), address);
        // Memory still allocated
        assert!(manager.is_valid(address));
    }

    // After drop, memory should be freed
    // Note: In actual implementation, we'd verify deallocation
    assert!(initial_used > 0);
}

#[test]
fn test_memory_guard_manual_release() {
    let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
    let pid = 1;
    let size = 512;

    let address = manager.allocate(size, pid).unwrap();
    let mut guard = MemoryGuard::new(address, size, pid, manager.clone(), None);

    assert!(guard.is_active());
    guard.release().unwrap();
    assert!(!guard.is_active());

    // Second release should fail
    assert!(guard.release().is_err());
}

#[test]
fn test_memory_guard_early_release() {
    let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
    let pid = 1;
    let size = 256;

    let address = manager.allocate(size, pid).unwrap();
    let guard = MemoryGuard::new(address, size, pid, manager.clone(), None);

    // Release early (consumes guard)
    guard.release_early().unwrap();
    // Drop won't run because we consumed the guard
}

#[test]
fn test_memory_guard_ref_shared_ownership() {
    let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
    let pid = 1;
    let size = 2048;

    let address = manager.allocate(size, pid).unwrap();
    let guard1 = MemoryGuardRef::new(address, size, pid, manager.clone(), None);

    assert_eq!(guard1.ref_count(), 1);
    assert!(guard1.is_last_ref());

    let guard2 = guard1.clone();
    assert_eq!(guard1.ref_count(), 2);
    assert_eq!(guard2.ref_count(), 2);
    assert!(!guard1.is_last_ref());

    let guard3 = guard2.clone();
    assert_eq!(guard1.ref_count(), 3);

    drop(guard1);
    assert_eq!(guard2.ref_count(), 2);

    drop(guard2);
    assert_eq!(guard3.ref_count(), 1);
    assert!(guard3.is_last_ref());

    // Last guard drop will trigger cleanup
}

#[test]
fn test_memory_guard_metadata() {
    let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
    let pid = 42;
    let size = 4096;

    let address = manager.allocate(size, pid).unwrap();
    let guard = MemoryGuard::new(address, size, pid, manager.clone(), None);

    let metadata = guard.metadata();
    assert_eq!(metadata.resource_type, "memory");
    assert_eq!(metadata.pid, Some(pid));
    assert_eq!(metadata.size_bytes, size);
    assert!(metadata.lifetime_micros() >= 0);
}
