/*!
 * Guard Integration Tests
 *
 * Real-world scenarios combining multiple guard types
 */

use ai_os_kernel::core::guard::*;
use ai_os_kernel::memory::manager::MemoryManager;
use ai_os_kernel::monitoring::Collector;
use std::sync::Arc;

#[test]
fn test_combined_memory_and_transaction() {
    let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
    let pid = 1;

    let commit_fn = |_: &[Operation]| {
        println!("Transaction committed");
        Ok(())
    };

    let rollback_fn = |_: &[Operation]| {
        println!("Transaction rolled back");
        Ok(())
    };

    let mut tx = TransactionGuard::new(Some(pid), commit_fn, rollback_fn);

    // Allocate memory within transaction
    let address = manager.allocate(1024, pid).unwrap();
    let memory_guard = MemoryGuard::new(address, 1024, pid, manager.clone(), None);

    tx.add_operation(Operation::new("alloc_memory", vec![]))
        .unwrap();

    // Use memory...
    assert!(memory_guard.is_active());

    // Commit transaction
    tx.commit().unwrap();

    // Memory guard still active and will clean up on drop
    drop(memory_guard);
}

#[test]
fn test_composite_with_multiple_resource_types() {
    let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
    let pid = 1;

    // Allocate memory
    let addr1 = manager.allocate(512, pid).unwrap();
    let mem_guard1 = MemoryGuard::new(addr1, 512, pid, manager.clone(), None);

    let addr2 = manager.allocate(1024, pid).unwrap();
    let mem_guard2 = MemoryGuard::new(addr2, 1024, pid, manager.clone(), None);

    // Create IPC resource
    use std::sync::atomic::{AtomicBool, Ordering};
    let ipc_cleaned = Arc::new(AtomicBool::new(false));
    let ipc_cleaned_clone = ipc_cleaned.clone();

    let ipc_guard = IpcGuard::new(
        42,
        IpcResourceType::Pipe,
        pid,
        move |_| {
            ipc_cleaned_clone.store(true, Ordering::SeqCst);
            Ok(())
        },
        None,
    );

    // Combine all guards
    {
        let _composite = CompositeGuardBuilder::new()
            .with("memory1", mem_guard1)
            .with("memory2", mem_guard2)
            .with("ipc", ipc_guard)
            .build();

        // All resources active
    }

    // All resources should be cleaned up
    assert!(ipc_cleaned.load(Ordering::SeqCst));
}

#[test]
fn test_observable_composite_guard() {
    let collector = Arc::new(Collector::new(1000));
    let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
    let pid = 1;

    let addr = manager.allocate(256, pid).unwrap();
    let mem_guard = MemoryGuard::new(addr, 256, pid, manager, Some(collector.clone()));

    // Wrap in observable
    let observable = ObservableGuard::wrap(mem_guard, collector);

    // Create composite with observable guard
    let _composite = CompositeGuard::new().add(observable);

    // Events should be emitted throughout lifecycle
}

#[test]
fn test_typed_guard_with_transaction() {
    struct ProcessState {
        pid: u32,
        status: String,
    }

    let process = ProcessState {
        pid: 1,
        status: "initializing".to_string(),
    };

    let guard: TypedGuard<ProcessState, Uninitialized> =
        TypedGuard::new(process, "process");

    let commit_fn = |_: &[Operation]| Ok(());
    let rollback_fn = |_: &[Operation]| Ok(());

    let mut tx = TransactionGuard::new(None, commit_fn, rollback_fn);

    // Transition with transaction
    let guard: TypedGuard<ProcessState, Initialized> = guard
        .with_transition(|p| {
            p.status = "initialized".to_string();
            tx.add_operation(Operation::new("init_process", vec![]))?;
            Ok(())
        })
        .unwrap();

    let guard: TypedGuard<ProcessState, Running> = guard.transition();

    tx.commit().unwrap();

    assert_eq!(guard.resource().status, "initialized");
    assert_eq!(guard.state_name(), "running");
}

#[test]
fn test_lock_guard_with_memory() {
    let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
    let pid = 1;

    let addr = manager.allocate(1024, pid).unwrap();
    let mem_guard = MemoryGuard::new(addr, 1024, pid, manager, None);

    // Wrap memory guard in lock guard for thread-safe access
    let unlocked = LockGuard::new(mem_guard);
    let locked = unlocked.lock().unwrap();

    // Access memory guard
    let mem_guard_ref = locked.access();
    assert!(mem_guard_ref.is_active());

    let unlocked = locked.unlock();
    let locked = unlocked.lock().unwrap();

    // Still active
    assert!(locked.access().is_active());
}
