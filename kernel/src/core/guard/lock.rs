/*!
 * Lock Guards with Type-State Pattern
 *
 * Type-safe lock guards that encode lock state in the type system
 */

use super::traits::{Guard, GuardDrop, Recoverable};
use super::{GuardError, GuardMetadata, GuardResult};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, MutexGuard as StdMutexGuard, PoisonError};

/// Lock state marker traits
pub trait LockState: Send + Sync {}

/// Type marker for locked state
pub struct Locked;
impl LockState for Locked {}

/// Type marker for unlocked state
pub struct Unlocked;
impl LockState for Unlocked {}

/// Type-safe lock guard with state tracking
///
/// # Type States
///
/// - `LockGuard<Unlocked>`: Lock not held
/// - `LockGuard<Locked>`: Lock held, can access data
///
/// # Example
///
/// ```rust
/// let unlocked = LockGuard::new(data);
/// let locked = unlocked.lock()?; // Type changes to Locked
/// let value = locked.access(); // Only available when Locked
/// let unlocked = locked.unlock(); // Type changes back to Unlocked
/// ```
pub struct LockGuard<T, S: LockState = Unlocked> {
    data: Arc<Mutex<T>>,
    guard: Option<StdMutexGuard<'static, T>>,
    metadata: GuardMetadata,
    poisoned: bool,
    poison_reason: Option<String>,
    _state: PhantomData<S>,
}

impl<T: Send> LockGuard<T, Unlocked> {
    /// Create a new unlocked guard
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
            guard: None,
            metadata: GuardMetadata::new("lock"),
            poisoned: false,
            poison_reason: None,
            _state: PhantomData,
        }
    }

    /// Acquire the lock, transitioning to Locked state
    ///
    /// # Type Safety
    ///
    /// Returns `LockGuard<T, Locked>` - different type!
    pub fn lock(self) -> GuardResult<LockGuard<T, Locked>> {
        let guard = self
            .data
            .lock()
            .map_err(|e| GuardError::Poisoned(format!("Lock poisoned: {:?}", e)))?;

        // SAFETY: We're extending the lifetime to 'static because we're storing
        // the Arc which keeps the data alive. This is safe because we control
        // the guard lifetime through the LockGuard struct.
        let guard_static: StdMutexGuard<'static, T> = unsafe { std::mem::transmute(guard) };

        Ok(LockGuard {
            data: self.data.clone(),
            guard: Some(guard_static),
            metadata: self.metadata,
            poisoned: false,
            poison_reason: None,
            _state: PhantomData,
        })
    }

    /// Try to acquire lock without blocking
    pub fn try_lock(self) -> Result<LockGuard<T, Locked>, Self> {
        match self.data.try_lock() {
            Ok(guard) => {
                let guard_static: StdMutexGuard<'static, T> = unsafe { std::mem::transmute(guard) };

                Ok(LockGuard {
                    data: self.data.clone(),
                    guard: Some(guard_static),
                    metadata: self.metadata,
                    poisoned: false,
                    poison_reason: None,
                    _state: PhantomData,
                })
            }
            Err(_) => Err(self),
        }
    }
}

impl<T: Send> LockGuard<T, Locked> {
    /// Access the protected data
    ///
    /// # Type Safety
    ///
    /// Only available when State = Locked
    #[inline]
    pub fn access(&self) -> &T {
        self.guard.as_ref().unwrap()
    }

    /// Mutably access the protected data
    #[inline]
    pub fn access_mut(&mut self) -> &mut T {
        self.guard.as_mut().unwrap()
    }

    /// Release the lock, transitioning to Unlocked state
    pub fn unlock(mut self) -> LockGuard<T, Unlocked> {
        drop(self.guard.take());

        LockGuard {
            data: self.data,
            guard: None,
            metadata: self.metadata,
            poisoned: self.poisoned,
            poison_reason: self.poison_reason,
            _state: PhantomData,
        }
    }

    /// Execute a function with the locked data
    ///
    /// Automatically unlocks after function completes
    pub fn with<F, R>(mut self, f: F) -> (R, LockGuard<T, Unlocked>)
    where
        F: FnOnce(&mut T) -> R,
    {
        let result = f(self.access_mut());
        let unlocked = self.unlock();
        (result, unlocked)
    }
}

impl<T, S: LockState> Guard for LockGuard<T, S> {
    fn resource_type(&self) -> &'static str {
        if self.guard.is_some() {
            "lock_locked"
        } else {
            "lock_unlocked"
        }
    }

    fn metadata(&self) -> &GuardMetadata {
        &self.metadata
    }

    fn is_active(&self) -> bool {
        !self.poisoned
    }

    fn release(&mut self) -> GuardResult<()> {
        if self.guard.is_some() {
            self.guard = None;
            Ok(())
        } else {
            Err(GuardError::AlreadyReleased)
        }
    }
}

impl<T> GuardDrop for LockGuard<T, Locked> {
    fn on_drop(&mut self) {
        // Release lock
        self.guard = None;
    }
}

impl<T, S: LockState> Recoverable for LockGuard<T, S> {
    fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    fn recover(&mut self) -> GuardResult<()> {
        if !self.poisoned {
            return Ok(());
        }

        self.poisoned = false;
        self.poison_reason = None;
        Ok(())
    }

    fn poison_reason(&self) -> Option<&str> {
        self.poison_reason.as_deref()
    }

    fn poison(&mut self, reason: String) {
        self.poisoned = true;
        self.poison_reason = Some(reason);
    }
}

impl<T> Drop for LockGuard<T, Locked> {
    fn drop(&mut self) {
        self.on_drop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_guard_type_states() {
        let unlocked = LockGuard::new(42);

        // Can't access data when unlocked (compile error if uncommented)
        // let _ = unlocked.access();

        let locked = unlocked.lock().unwrap();

        // Can access data when locked
        assert_eq!(*locked.access(), 42);

        // Unlock to change state back
        let _unlocked = locked.unlock();
    }

    #[test]
    fn test_lock_guard_with() {
        let unlocked = LockGuard::new(vec![1, 2, 3]);
        let locked = unlocked.lock().unwrap();

        let (len, _unlocked) = locked.with(|vec| {
            vec.push(4);
            vec.len()
        });

        assert_eq!(len, 4);
    }

    #[test]
    fn test_try_lock() {
        let guard1 = LockGuard::new(100);
        let locked = guard1.lock().unwrap();

        let guard2 = LockGuard {
            data: locked.data.clone(),
            guard: None,
            metadata: GuardMetadata::new("lock"),
            poisoned: false,
            poison_reason: None,
            _state: PhantomData,
        };

        // Try lock should fail
        assert!(guard2.try_lock().is_err());

        drop(locked);

        // Now should succeed
        let guard3 = LockGuard {
            data: locked.data.clone(),
            guard: None,
            metadata: GuardMetadata::new("lock"),
            poisoned: false,
            poison_reason: None,
            _state: PhantomData,
        };
        assert!(guard3.try_lock().is_ok());
    }

    #[test]
    fn test_lock_guard_recoverable() {
        let mut guard = LockGuard::new(0);

        assert!(!guard.is_poisoned());

        guard.poison("Test poison".to_string());
        assert!(guard.is_poisoned());
        assert_eq!(guard.poison_reason(), Some("Test poison"));

        guard.recover().unwrap();
        assert!(!guard.is_poisoned());
    }
}
