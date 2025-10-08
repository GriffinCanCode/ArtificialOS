/*!
 * Composite Guard Tests
 */

use ai_os_kernel::core::guard::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct CountingGuard {
    id: usize,
    metadata: GuardMetadata,
    active: bool,
    release_count: Arc<AtomicUsize>,
}

impl Guard for CountingGuard {
    fn resource_type(&self) -> &'static str {
        "counting"
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
        self.release_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn test_composite_guard_basic() {
    let count = Arc::new(AtomicUsize::new(0));

    let guard1 = CountingGuard {
        id: 1,
        metadata: GuardMetadata::new("test"),
        active: true,
        release_count: count.clone(),
    };

    let guard2 = CountingGuard {
        id: 2,
        metadata: GuardMetadata::new("test"),
        active: true,
        release_count: count.clone(),
    };

    {
        let composite = CompositeGuard::new().add(guard1).add(guard2);

        assert_eq!(composite.len(), 2);
        assert!(composite.all_active());
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    // Both should be released
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[test]
fn test_composite_guard_builder() {
    let count = Arc::new(AtomicUsize::new(0));

    let guard1 = CountingGuard {
        id: 1,
        metadata: GuardMetadata::new("test"),
        active: true,
        release_count: count.clone(),
    };

    let guard2 = CountingGuard {
        id: 2,
        metadata: GuardMetadata::new("test"),
        active: true,
        release_count: count.clone(),
    };

    let guard3 = CountingGuard {
        id: 3,
        metadata: GuardMetadata::new("test"),
        active: true,
        release_count: count.clone(),
    };

    {
        let _composite = CompositeGuardBuilder::new()
            .with("memory", guard1)
            .with("ipc", guard2)
            .with("socket", guard3)
            .build();
    }

    assert_eq!(count.load(Ordering::SeqCst), 3);
}

#[test]
fn test_composite_guard_manual_release() {
    let count = Arc::new(AtomicUsize::new(0));

    let guard1 = CountingGuard {
        id: 1,
        metadata: GuardMetadata::new("test"),
        active: true,
        release_count: count.clone(),
    };

    let mut composite = CompositeGuard::new().add(guard1);

    assert!(composite.is_active());
    composite.release().unwrap();
    assert!(!composite.is_active());

    // Already released
    assert!(composite.release().is_err());
}

#[test]
fn test_composite_guard_empty() {
    let composite = CompositeGuard::new();

    assert_eq!(composite.len(), 0);
    assert!(composite.is_empty());
    assert!(composite.guard_types().is_empty());
}

#[test]
fn test_composite_guard_types() {
    struct TypeAGuard;
    impl Guard for TypeAGuard {
        fn resource_type(&self) -> &'static str {
            "type_a"
        }
        fn metadata(&self) -> &GuardMetadata {
            static META: once_cell::sync::Lazy<GuardMetadata> =
                once_cell::sync::Lazy::new(|| GuardMetadata::new("type_a"));
            &META
        }
        fn is_active(&self) -> bool {
            true
        }
        fn release(&mut self) -> GuardResult<()> {
            Ok(())
        }
    }

    struct TypeBGuard;
    impl Guard for TypeBGuard {
        fn resource_type(&self) -> &'static str {
            "type_b"
        }
        fn metadata(&self) -> &GuardMetadata {
            static META: once_cell::sync::Lazy<GuardMetadata> =
                once_cell::sync::Lazy::new(|| GuardMetadata::new("type_b"));
            &META
        }
        fn is_active(&self) -> bool {
            true
        }
        fn release(&mut self) -> GuardResult<()> {
            Ok(())
        }
    }

    let composite = CompositeGuard::new()
        .add(TypeAGuard)
        .add(TypeBGuard);

    let types = composite.guard_types();
    assert_eq!(types.len(), 2);
    assert!(types.contains(&"type_a"));
    assert!(types.contains(&"type_b"));
}

#[test]
fn test_composite_guard_lifo_release_order() {
    let release_order = Arc::new(std::sync::Mutex::new(Vec::new()));

    struct OrderedGuard {
        id: usize,
        order: Arc<std::sync::Mutex<Vec<usize>>>,
        metadata: GuardMetadata,
    }

    impl Guard for OrderedGuard {
        fn resource_type(&self) -> &'static str {
            "ordered"
        }
        fn metadata(&self) -> &GuardMetadata {
            &self.metadata
        }
        fn is_active(&self) -> bool {
            true
        }
        fn release(&mut self) -> GuardResult<()> {
            self.order.lock().unwrap().push(self.id);
            Ok(())
        }
    }

    {
        let guard1 = OrderedGuard {
            id: 1,
            order: release_order.clone(),
            metadata: GuardMetadata::new("test"),
        };

        let guard2 = OrderedGuard {
            id: 2,
            order: release_order.clone(),
            metadata: GuardMetadata::new("test"),
        };

        let guard3 = OrderedGuard {
            id: 3,
            order: release_order.clone(),
            metadata: GuardMetadata::new("test"),
        };

        let _composite = CompositeGuard::new()
            .add(guard1)
            .add(guard2)
            .add(guard3);
    }

    // Should release in reverse order (LIFO): 3, 2, 1
    let order = release_order.lock().unwrap();
    assert_eq!(*order, vec![3, 2, 1]);
}
