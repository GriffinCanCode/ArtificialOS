/*!
 * Typed Guard Tests
 */

use ai_os_kernel::core::guard::*;

struct TestResource {
    value: usize,
    state: String,
}

#[test]
fn test_typed_guard_basic() {
    let resource = TestResource {
        value: 42,
        state: "uninitialized".to_string(),
    };

    let guard: TypedGuard<TestResource, Uninitialized> =
        TypedGuard::new(resource, "test");

    assert_eq!(guard.state_name(), "uninitialized");
    assert_eq!(guard.resource().value, 42);
}

#[test]
fn test_typed_guard_state_transitions() {
    let resource = TestResource {
        value: 0,
        state: "uninitialized".to_string(),
    };

    let guard: TypedGuard<TestResource, Uninitialized> =
        TypedGuard::new(resource, "test");
    assert_eq!(guard.state_name(), "uninitialized");

    let guard: TypedGuard<TestResource, Initialized> = guard.transition();
    assert_eq!(guard.state_name(), "initialized");

    let guard: TypedGuard<TestResource, Running> = guard.transition();
    assert_eq!(guard.state_name(), "running");

    let guard: TypedGuard<TestResource, Stopped> = guard.transition();
    assert_eq!(guard.state_name(), "stopped");

    let guard: TypedGuard<TestResource, Terminated> = guard.transition();
    assert_eq!(guard.state_name(), "terminated");
}

#[test]
fn test_typed_guard_with_transition() {
    let resource = TestResource {
        value: 0,
        state: "init".to_string(),
    };

    let guard: TypedGuard<TestResource, Initialized> =
        TypedGuard::new(resource, "test");

    let guard: TypedGuard<TestResource, Running> = guard
        .with_transition(|r| {
            r.value = 100;
            r.state = "running".to_string();
            Ok(())
        })
        .unwrap();

    assert_eq!(guard.resource().value, 100);
    assert_eq!(guard.resource().state, "running");
    assert_eq!(guard.state_name(), "running");
}

#[test]
fn test_typed_guard_mutation() {
    let resource = TestResource {
        value: 10,
        state: "test".to_string(),
    };

    let mut guard: TypedGuard<TestResource, Initialized> =
        TypedGuard::new(resource, "test");

    guard.resource_mut().value += 5;
    assert_eq!(guard.resource().value, 15);

    guard.resource_mut().state = "modified".to_string();
    assert_eq!(guard.resource().state, "modified");
}

#[test]
fn test_typed_guard_into_resource() {
    let resource = TestResource {
        value: 999,
        state: "final".to_string(),
    };

    let guard: TypedGuard<TestResource, Initialized> =
        TypedGuard::new(resource, "test");

    let extracted = guard.into_resource();
    assert_eq!(extracted.value, 999);
    assert_eq!(extracted.state, "final");
}

#[test]
fn test_typed_guard_try_transition_valid() {
    let resource = TestResource {
        value: 0,
        state: "uninit".to_string(),
    };

    let guard: TypedGuard<TestResource, Uninitialized> =
        TypedGuard::new(resource, "test");

    let result = guard.try_transition::<Initialized>();
    assert!(result.is_ok());
}

#[test]
fn test_typed_guard_try_transition_invalid() {
    let resource = TestResource {
        value: 0,
        state: "uninit".to_string(),
    };

    let guard: TypedGuard<TestResource, Uninitialized> =
        TypedGuard::new(resource, "test");

    // Try to transition directly to Running (invalid)
    let result = guard.try_transition::<Running>();
    assert!(result.is_err());
}

#[test]
fn test_typed_guard_custom_states() {
    struct Idle;
    impl TypedState for Idle {
        fn state_name() -> &'static str {
            "idle"
        }
    }

    struct Active;
    impl TypedState for Active {
        fn state_name() -> &'static str {
            "active"
        }
    }

    let resource = TestResource {
        value: 0,
        state: "idle".to_string(),
    };

    let guard: TypedGuard<TestResource, Idle> = TypedGuard::new(resource, "test");
    assert_eq!(guard.state_name(), "idle");

    let guard: TypedGuard<TestResource, Active> = guard.transition();
    assert_eq!(guard.state_name(), "active");
}
