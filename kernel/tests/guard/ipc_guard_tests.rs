/*!
 * IPC Guard Tests
 */

use ai_os_kernel::core::guard::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_ipc_guard_cleanup() {
    let cleaned = Arc::new(AtomicBool::new(false));
    let cleaned_clone = cleaned.clone();

    let cleanup = move |id: u64| {
        assert_eq!(id, 42);
        cleaned_clone.store(true, Ordering::SeqCst);
        Ok(())
    };

    {
        let guard = IpcGuard::new(42, IpcResourceType::Pipe, 1, cleanup, None);
        assert!(guard.is_active());
        assert_eq!(guard.resource_id(), 42);
        assert_eq!(guard.resource_type_kind(), IpcResourceType::Pipe);
        assert!(!cleaned.load(Ordering::SeqCst));
    }

    // Should be cleaned after drop
    assert!(cleaned.load(Ordering::SeqCst));
}

#[test]
fn test_ipc_guard_all_types() {
    let types = vec![
        IpcResourceType::Pipe,
        IpcResourceType::Queue,
        IpcResourceType::SharedMemory,
        IpcResourceType::ZeroCopyRing,
    ];

    for resource_type in types {
        let cleanup = |_: u64| Ok(());
        let guard = IpcGuard::new(1, resource_type, 1, cleanup, None);
        assert_eq!(guard.resource_type(), resource_type.as_str());
    }
}

#[test]
fn test_ipc_guard_ref_shared() {
    let count = Arc::new(AtomicUsize::new(0));
    let count_clone = count.clone();

    let cleanup = move |_: u64| {
        count_clone.fetch_add(1, Ordering::SeqCst);
        Ok(())
    };

    let guard1 = IpcGuardRef::new(100, IpcResourceType::Queue, 1, cleanup, None);
    assert_eq!(guard1.ref_count(), 1);

    let guard2 = guard1.clone();
    let guard3 = guard2.clone();

    assert_eq!(guard1.ref_count(), 3);
    assert_eq!(guard2.ref_count(), 3);
    assert_eq!(guard3.ref_count(), 3);

    drop(guard1);
    drop(guard2);

    assert_eq!(count.load(Ordering::SeqCst), 0); // Not yet cleaned

    drop(guard3); // Last reference

    assert_eq!(count.load(Ordering::SeqCst), 1); // Now cleaned
}

#[test]
fn test_ipc_guard_early_release() {
    let cleaned = Arc::new(AtomicBool::new(false));
    let cleaned_clone = cleaned.clone();

    let cleanup = move |_: u64| {
        cleaned_clone.store(true, Ordering::SeqCst);
        Ok(())
    };

    let guard = IpcGuard::new(
        999,
        IpcResourceType::SharedMemory,
        1,
        cleanup,
        None,
    );

    guard.release_early().unwrap();
    assert!(cleaned.load(Ordering::SeqCst));
}

#[test]
fn test_ipc_guard_cleanup_error() {
    let cleanup = |_: u64| Err("Cleanup failed".to_string());

    let mut guard = IpcGuard::new(1, IpcResourceType::Pipe, 1, cleanup, None);

    let result = guard.release();
    assert!(result.is_err());
    match result {
        Err(GuardError::OperationFailed(msg)) => {
            assert_eq!(msg, "Cleanup failed");
        }
        _ => panic!("Wrong error type"),
    }
}
