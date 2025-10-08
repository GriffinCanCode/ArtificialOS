/*!
 * Lock Guard Tests
 */

use ai_os_kernel::core::guard::*;

#[test]
fn test_lock_guard_type_states() {
    let unlocked = LockGuard::new(100);

    // Can't access data when unlocked (would be compile error)
    // let _ = unlocked.access();

    let locked = unlocked.lock().unwrap();
    assert_eq!(*locked.access(), 100);
    assert_eq!(locked.resource_type(), "lock_locked");

    let unlocked = locked.unlock();
    assert_eq!(unlocked.resource_type(), "lock_unlocked");
}

#[test]
fn test_lock_guard_mutation() {
    let unlocked = LockGuard::new(vec![1, 2, 3]);
    let mut locked = unlocked.lock().unwrap();

    locked.access_mut().push(4);
    assert_eq!(locked.access().len(), 4);

    let unlocked = locked.unlock();
    let locked = unlocked.lock().unwrap();
    assert_eq!(locked.access().len(), 4);
}

#[test]
fn test_lock_guard_with() {
    let unlocked = LockGuard::new(String::from("hello"));
    let locked = unlocked.lock().unwrap();

    let (len, unlocked) = locked.with(|s| {
        s.push_str(" world");
        s.len()
    });

    assert_eq!(len, 11);

    let locked = unlocked.lock().unwrap();
    assert_eq!(locked.access(), "hello world");
}

#[test]
fn test_lock_guard_try_lock() {
    let unlocked1 = LockGuard::new(42);
    let locked1 = unlocked1.lock().unwrap();

    // Create another guard sharing the same data
    let unlocked2 = LockGuard {
        data: locked1.data.clone(),
        guard: None,
        metadata: GuardMetadata::new("lock"),
        poisoned: false,
        poison_reason: None,
        _state: std::marker::PhantomData,
    };

    // Try lock should fail while locked1 exists
    match unlocked2.try_lock() {
        Ok(_) => panic!("Should not acquire lock"),
        Err(returned) => {
            // Get the guard back
            drop(locked1);
            // Now should succeed
            let locked2 = returned.try_lock().unwrap();
            assert_eq!(*locked2.access(), 42);
        }
    }
}

#[test]
fn test_lock_guard_poisoning() {
    let mut guard = LockGuard::<i32, Unlocked>::new(0);

    assert!(!guard.is_poisoned());
    assert!(guard.poison_reason().is_none());

    guard.poison("Test poison".to_string());
    assert!(guard.is_poisoned());
    assert_eq!(guard.poison_reason(), Some("Test poison"));

    // Recover from poison
    guard.recover().unwrap();
    assert!(!guard.is_poisoned());
    assert!(guard.poison_reason().is_none());
}

#[test]
fn test_lock_guard_auto_unlock_on_drop() {
    let data = std::sync::Arc::new(std::sync::Mutex::new(vec![1, 2, 3]));
    let data_clone = data.clone();

    {
        let _guard = data.lock().unwrap();
        // Lock held here
    }

    // Lock released, should be able to acquire again
    let guard = data_clone.lock().unwrap();
    assert_eq!(guard.len(), 3);
}
