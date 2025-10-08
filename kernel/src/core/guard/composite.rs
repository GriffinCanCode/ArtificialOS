/*!
 * Composite Guards
 *
 * Combine multiple guards into a single guard with unified lifecycle
 */

use super::traits::{Guard, GuardDrop};
use super::{GuardError, GuardMetadata, GuardResult};
use std::any::Any;

/// Composite guard that manages multiple guards as one
///
/// # Example
///
/// ```rust
/// let memory_guard = allocate_memory()?;
/// let ipc_guard = create_pipe()?;
///
/// let composite = CompositeGuard::new()
///     .add(memory_guard)
///     .add(ipc_guard);
///
/// // Both guards released together on drop
/// ```
pub struct CompositeGuard {
    guards: Vec<Box<dyn Guard>>,
    metadata: GuardMetadata,
    active: bool,
}

impl CompositeGuard {
    /// Create a new empty composite guard
    pub fn new() -> Self {
        Self {
            guards: Vec::new(),
            metadata: GuardMetadata::new("composite"),
            active: true,
        }
    }

    /// Add a guard to the composite
    pub fn add<G: Guard + 'static>(mut self, guard: G) -> Self {
        self.guards.push(Box::new(guard));
        self
    }

    /// Add a boxed guard to the composite
    pub fn add_boxed(mut self, guard: Box<dyn Guard>) -> Self {
        self.guards.push(guard);
        self
    }

    /// Get number of guards in the composite
    pub fn len(&self) -> usize {
        self.guards.len()
    }

    /// Check if composite is empty
    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }

    /// Get all guard resource types
    pub fn guard_types(&self) -> Vec<&'static str> {
        self.guards.iter().map(|g| g.resource_type()).collect()
    }

    /// Check if all guards are active
    pub fn all_active(&self) -> bool {
        self.guards.iter().all(|g| g.is_active())
    }

    /// Release all guards in reverse order (LIFO)
    ///
    /// Continues even if some guards fail, collecting all errors
    pub fn release_all(&mut self) -> Vec<GuardError> {
        let mut errors = Vec::new();

        // Release in reverse order for dependency safety
        for guard in self.guards.iter_mut().rev() {
            if let Err(e) = guard.release() {
                errors.push(e);
            }
        }

        self.active = false;
        errors
    }
}

impl Default for CompositeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Guard for CompositeGuard {
    fn resource_type(&self) -> &'static str {
        "composite"
    }

    fn metadata(&self) -> &GuardMetadata {
        &self.metadata
    }

    fn is_active(&self) -> bool {
        self.active && self.all_active()
    }

    fn release(&mut self) -> GuardResult<()> {
        if !self.active {
            return Err(GuardError::AlreadyReleased);
        }

        let errors = self.release_all();

        if errors.is_empty() {
            Ok(())
        } else {
            // Return first error, log others
            for (i, err) in errors.iter().enumerate().skip(1) {
                log::error!("Composite guard error {}: {}", i, err);
            }
            Err(errors.into_iter().next().unwrap())
        }
    }

    fn leak(self) -> Box<dyn Any>
    where
        Self: Sized + 'static,
    {
        // Leak all inner guards
        for guard in self.guards {
            let _ = guard.leak();
        }
        Box::new(self)
    }
}

impl GuardDrop for CompositeGuard {
    fn on_drop(&mut self) {
        if self.active {
            let errors = self.release_all();
            if !errors.is_empty() {
                log::error!(
                    "Composite guard drop had {} errors: {:?}",
                    errors.len(),
                    errors
                );
            }
        }
    }
}

impl Drop for CompositeGuard {
    fn drop(&mut self) {
        self.on_drop();
    }
}

/// Builder for composite guards with named guards
pub struct CompositeGuardBuilder {
    guards: Vec<(String, Box<dyn Guard>)>,
}

impl CompositeGuardBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }

    /// Add a named guard
    pub fn with<G: Guard + 'static>(mut self, name: impl Into<String>, guard: G) -> Self {
        self.guards.push((name.into(), Box::new(guard)));
        self
    }

    /// Build the composite guard
    pub fn build(self) -> CompositeGuard {
        let mut composite = CompositeGuard::new();
        for (name, guard) in self.guards {
            log::debug!("Adding guard '{}' to composite", name);
            composite = composite.add_boxed(guard);
        }
        composite
    }
}

impl Default for CompositeGuardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct TestGuard {
        id: usize,
        metadata: GuardMetadata,
        active: bool,
        release_count: Arc<AtomicUsize>,
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
    fn test_composite_guard() {
        let count = Arc::new(AtomicUsize::new(0));

        let guard1 = TestGuard {
            id: 1,
            metadata: GuardMetadata::new("test"),
            active: true,
            release_count: count.clone(),
        };

        let guard2 = TestGuard {
            id: 2,
            metadata: GuardMetadata::new("test"),
            active: true,
            release_count: count.clone(),
        };

        {
            let composite = CompositeGuard::new().add(guard1).add(guard2);

            assert_eq!(composite.len(), 2);
            assert!(composite.all_active());
        }

        // Both guards should be released
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_composite_guard_builder() {
        let count = Arc::new(AtomicUsize::new(0));

        let guard1 = TestGuard {
            id: 1,
            metadata: GuardMetadata::new("test"),
            active: true,
            release_count: count.clone(),
        };

        let guard2 = TestGuard {
            id: 2,
            metadata: GuardMetadata::new("test"),
            active: true,
            release_count: count.clone(),
        };

        {
            let _composite = CompositeGuardBuilder::new()
                .with("memory", guard1)
                .with("ipc", guard2)
                .build();
        }

        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_composite_manual_release() {
        let count = Arc::new(AtomicUsize::new(0));

        let guard = TestGuard {
            id: 1,
            metadata: GuardMetadata::new("test"),
            active: true,
            release_count: count.clone(),
        };

        let mut composite = CompositeGuard::new().add(guard);

        assert!(composite.release().is_ok());
        assert_eq!(count.load(Ordering::SeqCst), 1);

        // Second release should fail
        assert!(composite.release().is_err());
    }
}
