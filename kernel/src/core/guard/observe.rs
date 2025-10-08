/*!
 * Observable Guard Wrapper
 *
 * Wraps any guard to add observability
 */

use super::traits::{Guard, Observable};
use super::GuardMetadata;
use crate::monitoring::{Category, Collector, Event, Payload, Severity};
use std::sync::Arc;

/// Wrapper that adds observability to any guard
///
/// # Example
///
/// ```ignore
/// let guard = some_guard;
/// let observable = ObservableGuard::wrap(guard, collector);
/// // Now emits events automatically
/// ```
pub struct ObservableGuard<G: Guard> {
    inner: G,
    collector: Arc<Collector>,
    category: Category,
}

impl<G: Guard> ObservableGuard<G> {
    /// Wrap a guard to make it observable
    pub fn wrap(guard: G, collector: Arc<Collector>) -> Self {
        let wrapped = Self {
            inner: guard,
            collector,
            category: Category::Resource,
        };

        wrapped.emit_created();
        wrapped
    }

    /// Wrap with a specific event category
    pub fn wrap_with_category(guard: G, collector: Arc<Collector>, category: Category) -> Self {
        let wrapped = Self {
            inner: guard,
            collector,
            category,
        };

        wrapped.emit_created();
        wrapped
    }

    /// Get reference to inner guard
    pub fn inner(&self) -> &G {
        &self.inner
    }

    /// Get mutable reference to inner guard
    pub fn inner_mut(&mut self) -> &mut G {
        &mut self.inner
    }

    /// Unwrap to get the inner guard
    pub fn into_inner(mut self) -> G {
        // Extract inner before drop runs
        let inner_ptr = &mut self.inner as *mut G;
        std::mem::forget(self);
        unsafe { std::ptr::read(inner_ptr) }
    }

    /// Execute an operation with observability
    pub fn with_operation<F, R>(&mut self, operation: &str, f: F) -> R
    where
        F: FnOnce(&mut G) -> R,
    {
        self.emit_used(operation);
        f(&mut self.inner)
    }
}

impl<G: Guard> Observable for ObservableGuard<G> {
    fn emit_created(&self) {
        let event = Event::new(
            Severity::Debug,
            self.category,
            Payload::MetricUpdate {
                name: format!("{}_created", self.inner.resource_type().into()),
                value: 1.0,
                labels: vec![
                    (
                        "resource_type".to_string(),
                        self.inner.resource_type().to_string(),
                    ),
                    (
                        "creation_time".to_string(),
                        format!("{:?}", self.inner.metadata().creation_time),
                    ),
                ],
            },
        );
        self.collector.emit(event);
    }

    fn emit_used(&self, operation: &str) {
        let event = Event::new(
            Severity::Debug,
            self.category,
            Payload::MetricUpdate {
                name: format!("{}_used", self.inner.resource_type().into()),
                value: 1.0,
                labels: vec![
                    (
                        "resource_type".to_string(),
                        self.inner.resource_type().to_string(),
                    ),
                    ("operation".to_string(), operation.to_string().into()),
                ],
            },
        );
        self.collector.emit(event);
    }

    fn emit_dropped(&self) {
        let lifetime = self.inner.metadata().lifetime_micros();
        let event = Event::new(
            Severity::Debug,
            self.category,
            Payload::MetricUpdate {
                name: format!("{}_dropped", self.inner.resource_type().into()),
                value: lifetime as f64,
                labels: vec![
                    (
                        "resource_type".to_string(),
                        self.inner.resource_type().to_string(),
                    ),
                    ("lifetime_micros".to_string(), lifetime.to_string().into()),
                ],
            },
        );
        self.collector.emit(event);
    }

    fn emit_error(&self, error: &super::GuardError) {
        let event = Event::new(
            Severity::Error,
            self.category,
            Payload::MetricUpdate {
                name: format!("{}_error", self.inner.resource_type().into()),
                value: 1.0,
                labels: vec![
                    (
                        "resource_type".to_string(),
                        self.inner.resource_type().to_string(),
                    ),
                    ("error".to_string(), error.to_string().into()),
                ],
            },
        );
        self.collector.emit(event);
    }
}

impl<G: Guard> Guard for ObservableGuard<G> {
    fn resource_type(&self) -> &'static str {
        self.inner.resource_type()
    }

    fn metadata(&self) -> &GuardMetadata {
        self.inner.metadata()
    }

    fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    fn release(&mut self) -> super::GuardResult<()> {
        let result = self.inner.release();
        if let Err(ref e) = result {
            self.emit_error(e);
        } else {
            self.emit_dropped();
        }
        result
    }
}

impl<G: Guard> Drop for ObservableGuard<G> {
    fn drop(&mut self) {
        if self.is_active() {
            self.emit_dropped();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::Collector;

    struct TestGuard {
        metadata: GuardMetadata,
        active: bool,
    }

    impl Guard for TestGuard {
        fn resource_type(&self) -> &'static str {
            "test"
        }

        fn metadata(&self) -> &GuardMetadata {
            &self.metadata
        }

        fn is_active(&self) -> bool {
            self.active
        }

        fn release(&mut self) -> crate::GuardResult<()> {
            self.active = false;
            Ok(())
        }
    }

    #[test]
    fn test_observable_guard_wrapper() {
        let collector = Arc::new(Collector::new());
        let guard = TestGuard {
            metadata: GuardMetadata::new("test"),
            active: true,
        };

        let mut observable = ObservableGuard::wrap(guard, collector);

        assert!(observable.is_active());
        observable.release().unwrap();
        assert!(!observable.is_active());
    }

    #[test]
    fn test_observable_guard_with_operation() {
        let collector = Arc::new(Collector::new());
        let guard = TestGuard {
            metadata: GuardMetadata::new("test"),
            active: true,
        };

        let mut observable = ObservableGuard::wrap(guard, collector);

        observable.with_operation("test_op", |g| {
            assert!(g.is_active());
        });
    }
}
