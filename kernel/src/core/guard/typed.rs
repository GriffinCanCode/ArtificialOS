/*!
 * Typed State Guards
 *
 * Guards that encode state transitions in the type system
 */

use super::traits::Guard;
use super::{GuardError, GuardMetadata, GuardResult};
use std::marker::PhantomData;

/// Marker trait for type states
pub trait TypedState: Send + Sync {
    /// State name for debugging
    fn state_name() -> &'static str;

    /// Check if transition to another state is valid
    fn can_transition_to<S: TypedState>() -> bool {
        true // Default: allow all transitions
    }
}

/// Type-state guard that enforces valid state transitions at compile time
///
/// # Type Parameters
///
/// - `T`: The resource being guarded
/// - `S`: Current state (implements TypedState)
///
/// # Example
///
/// ```rust
/// struct Initialized;
/// impl TypedState for Initialized {
///     fn state_name() -> &'static str { "initialized" }
/// }
///
/// struct Running;
/// impl TypedState for Running {
///     fn state_name() -> &'static str { "running" }
/// }
///
/// let guard: TypedGuard<Process, Initialized> = ...;
/// let guard: TypedGuard<Process, Running> = guard.transition();
/// ```
pub struct TypedGuard<T, S: TypedState> {
    resource: T,
    metadata: GuardMetadata,
    _state: PhantomData<S>,
}

impl<T, S: TypedState> TypedGuard<T, S> {
    /// Create a new typed guard in the given state
    pub fn new(resource: T, resource_type: &'static str) -> Self {
        Self {
            resource,
            metadata: GuardMetadata::new(resource_type),
            _state: PhantomData,
        }
    }

    /// Transition to a new state
    ///
    /// # Type Safety
    ///
    /// The return type is different! This consumes self and returns a guard
    /// with a different state type parameter.
    pub fn transition<NewState: TypedState>(self) -> TypedGuard<T, NewState>
    where
        S: TypedState,
    {
        TypedGuard {
            resource: self.resource,
            metadata: self.metadata,
            _state: PhantomData,
        }
    }

    /// Try to transition to a new state with validation
    pub fn try_transition<NewState: TypedState>(self) -> Result<TypedGuard<T, NewState>, Self>
    where
        S: TypedState,
    {
        if S::can_transition_to::<NewState>() {
            Ok(self.transition())
        } else {
            Err(self)
        }
    }

    /// Access the resource immutably
    pub fn resource(&self) -> &T {
        &self.resource
    }

    /// Access the resource mutably
    pub fn resource_mut(&mut self) -> &mut T {
        &mut self.resource
    }

    /// Extract the resource, consuming the guard
    pub fn into_resource(self) -> T {
        self.resource
    }

    /// Get current state name
    pub fn state_name(&self) -> &'static str {
        S::state_name()
    }

    /// Execute a function with the resource and transition
    ///
    /// Useful for atomic state transitions with operations
    pub fn with_transition<NewState, F>(mut self, f: F) -> GuardResult<TypedGuard<T, NewState>>
    where
        NewState: TypedState,
        F: FnOnce(&mut T) -> GuardResult<()>,
    {
        f(&mut self.resource)?;
        Ok(self.transition())
    }
}

impl<T, S: TypedState> Guard for TypedGuard<T, S> {
    fn resource_type(&self) -> &'static str {
        self.metadata.resource_type
    }

    fn metadata(&self) -> &GuardMetadata {
        &self.metadata
    }

    fn is_active(&self) -> bool {
        true // Typed guards are always active
    }

    fn release(&mut self) -> GuardResult<()> {
        Ok(()) // Typed guards don't manage cleanup
    }
}

// Common state types

/// Uninitialized state
pub struct Uninitialized;
impl TypedState for Uninitialized {
    fn state_name() -> &'static str {
        "uninitialized"
    }

    fn can_transition_to<S: TypedState>() -> bool {
        // Can only transition to Initialized
        std::any::TypeId::of::<S>() == std::any::TypeId::of::<Initialized>()
    }
}

/// Initialized state
pub struct Initialized;
impl TypedState for Initialized {
    fn state_name() -> &'static str {
        "initialized"
    }
}

/// Running state
pub struct Running;
impl TypedState for Running {
    fn state_name() -> &'static str {
        "running"
    }
}

/// Stopped state
pub struct Stopped;
impl TypedState for Stopped {
    fn state_name() -> &'static str {
        "stopped"
    }
}

/// Terminated state
pub struct Terminated;
impl TypedState for Terminated {
    fn state_name() -> &'static str {
        "terminated"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestResource {
        value: usize,
    }

    #[test]
    fn test_typed_guard_transitions() {
        let resource = TestResource { value: 42 };
        let guard: TypedGuard<TestResource, Uninitialized> = TypedGuard::new(resource, "test");

        assert_eq!(guard.state_name(), "uninitialized");
        assert_eq!(guard.resource().value, 42);

        let guard: TypedGuard<TestResource, Initialized> = guard.transition();
        assert_eq!(guard.state_name(), "initialized");

        let guard: TypedGuard<TestResource, Running> = guard.transition();
        assert_eq!(guard.state_name(), "running");

        let guard: TypedGuard<TestResource, Stopped> = guard.transition();
        assert_eq!(guard.state_name(), "stopped");
    }

    #[test]
    fn test_typed_guard_with_transition() {
        let resource = TestResource { value: 0 };
        let guard: TypedGuard<TestResource, Initialized> = TypedGuard::new(resource, "test");

        let guard: TypedGuard<TestResource, Running> = guard
            .with_transition(|r| {
                r.value = 100;
                Ok(())
            })
            .unwrap();

        assert_eq!(guard.resource().value, 100);
        assert_eq!(guard.state_name(), "running");
    }

    #[test]
    fn test_typed_guard_into_resource() {
        let resource = TestResource { value: 999 };
        let guard: TypedGuard<TestResource, Initialized> = TypedGuard::new(resource, "test");

        let extracted = guard.into_resource();
        assert_eq!(extracted.value, 999);
    }

    #[test]
    fn test_typed_guard_validation() {
        let resource = TestResource { value: 0 };
        let guard: TypedGuard<TestResource, Uninitialized> = TypedGuard::new(resource, "test");

        // Valid transition
        let result = guard.try_transition::<Initialized>();
        assert!(result.is_ok());

        // Invalid transition (enforced by can_transition_to)
        let resource2 = TestResource { value: 0 };
        let guard2: TypedGuard<TestResource, Uninitialized> = TypedGuard::new(resource2, "test");
        let result2 = guard2.try_transition::<Running>();
        assert!(result2.is_err());
    }
}
