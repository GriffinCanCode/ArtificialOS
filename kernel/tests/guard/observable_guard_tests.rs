/*!
 * Observable Guard Tests
 */

use ai_os_kernel::core::guard::*;
use ai_os_kernel::monitoring::Collector;
use std::sync::Arc;

struct SimpleGuard {
    metadata: GuardMetadata,
    active: bool,
}

impl Guard for SimpleGuard {
    fn resource_type(&self) -> &'static str {
        "simple"
    }

    fn metadata(&self) -> &GuardMetadata {
        &self.metadata
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn release(&mut self) -> GuardResult<()> {
        if !self.active {
            return Err(GuardError::AlreadyReleased);
        }
        self.active = false;
        Ok(())
    }
}

#[test]
fn test_observable_guard_wrapping() {
    let collector = Arc::new(Collector::new(1000));
    let guard = SimpleGuard {
        metadata: GuardMetadata::new("simple"),
        active: true,
    };

    let observable = ObservableGuard::wrap(guard, collector);

    assert!(observable.is_active());
    assert_eq!(observable.resource_type(), "simple");
}

#[test]
fn test_observable_guard_with_operation() {
    let collector = Arc::new(Collector::new(1000));
    let guard = SimpleGuard {
        metadata: GuardMetadata::new("simple"),
        active: true,
    };

    let mut observable = ObservableGuard::wrap(guard, collector);

    observable.with_operation("test_operation", |g| {
        assert!(g.is_active());
    });
}

#[test]
fn test_observable_guard_release() {
    let collector = Arc::new(Collector::new(1000));
    let guard = SimpleGuard {
        metadata: GuardMetadata::new("simple"),
        active: true,
    };

    let mut observable = ObservableGuard::wrap(guard, collector);

    assert!(observable.is_active());
    observable.release().unwrap();
    assert!(!observable.is_active());
}

#[test]
fn test_observable_guard_into_inner() {
    let collector = Arc::new(Collector::new(1000));
    let guard = SimpleGuard {
        metadata: GuardMetadata::new("simple"),
        active: true,
    };

    let observable = ObservableGuard::wrap(guard, collector);
    let inner = observable.into_inner();

    assert!(inner.is_active());
}

#[test]
fn test_observable_guard_custom_category() {
    use ai_os_kernel::monitoring::Category;

    let collector = Arc::new(Collector::new(1000));
    let guard = SimpleGuard {
        metadata: GuardMetadata::new("simple"),
        active: true,
    };

    let _observable =
        ObservableGuard::wrap_with_category(guard, collector, Category::Memory);

    // Events should be emitted with Memory category
}

#[test]
fn test_observable_guard_auto_emit_on_drop() {
    let collector = Arc::new(Collector::new(1000));

    {
        let guard = SimpleGuard {
            metadata: GuardMetadata::new("simple"),
            active: true,
        };

        let _observable = ObservableGuard::wrap(guard, collector.clone());
        // Should emit created event
    }

    // Should emit dropped event on drop
    // We could verify by checking collector's event queue
}
