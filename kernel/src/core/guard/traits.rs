/*!
 * Guard Traits
 *
 * Core abstractions for RAII resource guards
 */

use super::{GuardError, GuardMetadata, GuardResult};
use std::any::Any;

/// Core guard trait
///
/// All guards must implement this to provide:
/// - Resource type identification
/// - Metadata access
/// - Manual release capability
///
/// # Type Safety
///
/// Guards use the type system to ensure resources are used correctly.
/// Invalid operations are compile errors, not runtime errors.
pub trait Guard: Send {
    /// Resource type name for logging/debugging
    fn resource_type(&self) -> &'static str;

    /// Get guard metadata
    fn metadata(&self) -> &GuardMetadata;

    /// Check if guard is still active
    fn is_active(&self) -> bool;

    /// Manually release the resource
    ///
    /// Returns `Err` if already released
    fn release(&mut self) -> GuardResult<()>;

    /// Leak the guard, preventing Drop from running
    ///
    /// Use when transferring ownership to another system
    fn leak(self) -> Box<dyn Any>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

/// Guards that can be dropped with custom cleanup
///
/// Separates Drop logic for better testability and observability
pub trait GuardDrop: Guard {
    /// Perform cleanup on drop
    ///
    /// # Panics
    ///
    /// Should NOT panic. Log errors instead.
    fn on_drop(&mut self);
}

/// Guards that can recover from poisoned state
///
/// Useful for lock guards and transaction guards
pub trait Recoverable: Guard {
    /// Check if guard is poisoned
    fn is_poisoned(&self) -> bool;

    /// Attempt to recover from poisoned state
    ///
    /// Returns `Ok(())` if recovery succeeded
    fn recover(&mut self) -> GuardResult<()>;

    /// Get poison reason if poisoned
    fn poison_reason(&self) -> Option<&str>;

    /// Mark as poisoned with a reason
    fn poison(&mut self, reason: String);
}

/// Guards with observable lifecycle
///
/// Automatically emits events for creation, usage, and cleanup
pub trait Observable: Guard {
    /// Emit creation event
    fn emit_created(&self);

    /// Emit usage event with operation name
    fn emit_used(&self, operation: &str);

    /// Emit cleanup event
    fn emit_dropped(&self);

    /// Emit error event
    fn emit_error(&self, error: &GuardError);
}

/// Guards that can be cloned with reference counting
///
/// Useful for shared ownership scenarios
pub trait GuardRef: Guard + Clone {
    /// Get current reference count
    fn ref_count(&self) -> usize;

    /// Check if this is the last reference
    fn is_last_ref(&self) -> bool {
        self.ref_count() == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        fn release(&mut self) -> GuardResult<()> {
            if !self.active {
                return Err(GuardError::AlreadyReleased);
            }
            self.active = false;
            Ok(())
        }
    }

    #[test]
    fn test_guard_release() {
        let mut guard = TestGuard {
            metadata: GuardMetadata::new("test"),
            active: true,
        };

        assert!(guard.is_active());
        assert!(guard.release().is_ok());
        assert!(!guard.is_active());
        assert!(guard.release().is_err());
    }
}
