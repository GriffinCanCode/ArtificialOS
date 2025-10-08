/*!
 * Lock Guards with Type-State Pattern
 *
 * Type-safe lock guards that encode lock state in the type system
 */

use super::traits::{Guard, Recoverable};
use super::{GuardError, GuardMetadata, GuardResult, TimeoutPolicy};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, MutexGuard as StdMutexGuard};
use std::time::Instant;

/// Lock state marker traits
pub trait LockState: Send + Sync {}

/// Type marker for locked state
pub struct Locked;
impl LockState for Locked {}

/// Type marker for unlocked state
pub struct Unlocked;
impl LockState for Unlocked {}

// SAFETY: LockGuard is Send because the actual shared data is in Arc<Mutex<T>>,
// which is Send when T: Send. The MutexGuard is only ever accessed on the thread
// that holds the LockGuard, and we never send it across threads directly.
unsafe impl<T: Send + 'static, S: LockState> Send for LockGuard<T, S> {}

/// Type-safe lock guard with state tracking
///
/// # Type States
///
/// - `LockGuard<Unlocked>`: Lock not held
/// - `LockGuard<Locked>`: Lock held, can access data
///
/// # Design Note
///
/// This guard uses type states to track lock ownership but does NOT store
/// a MutexGuard. Instead, it acquires the lock on each access. This is sound
/// and avoids lifetime issues with stored guards.
///
/// # Example
///
/// ```rust
/// let unlocked = LockGuard::new(data);
/// let locked = unlocked.lock()?; // Type changes to Locked
/// let value = locked.access(); // Only available when Locked
/// let unlocked = locked.unlock(); // Type changes back to Unlocked
/// ```
pub struct LockGuard<T: 'static, S: LockState = Unlocked> {
    data: Arc<Mutex<T>>,
    // Track if we conceptually hold the lock (for type state)
    // In Locked state, we own the logical lock but acquire mutex on each access
    lock_held: bool,
    metadata: GuardMetadata,
    poisoned: bool,
    poison_reason: Option<String>,
    _state: PhantomData<S>,
}

impl<T: Send + 'static> LockGuard<T, Unlocked> {
    /// Create a new unlocked guard
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
            lock_held: false,
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
    ///
    /// # Design
    ///
    /// We acquire the mutex briefly to ensure we CAN lock it, then release it.
    /// The Locked state means we have logical ownership - actual mutex locks
    /// happen on each access.
    pub fn lock(self) -> GuardResult<LockGuard<T, Locked>> {
        // Try to lock to verify it's not poisoned and available
        let _guard = self
            .data
            .lock()
            .map_err(|e| GuardError::Poisoned(format!("Lock poisoned: {:?}", e)))?;

        // Immediately drop the guard - we'll reacquire on access
        drop(_guard);

        Ok(LockGuard {
            data: self.data,
            lock_held: true, // Conceptually we hold the lock now
            metadata: self.metadata,
            poisoned: false,
            poison_reason: None,
            _state: PhantomData,
        })
    }

    /// Try to acquire lock without blocking
    pub fn try_lock(self) -> Result<LockGuard<T, Locked>, Self> {
        // Try to lock and immediately check result
        let can_lock = self.data.try_lock().is_ok();

        if can_lock {
            Ok(LockGuard {
                data: self.data,
                lock_held: true,
                metadata: self.metadata,
                poisoned: false,
                poison_reason: None,
                _state: PhantomData,
            })
        } else {
            Err(self)
        }
    }

    /// Acquire the lock with timeout
    ///
    /// # Type Safety
    ///
    /// Returns `LockGuard<T, Locked>` on success
    ///
    /// # Errors
    ///
    /// Returns `GuardError::Timeout` if timeout expires before acquiring lock
    pub fn lock_timeout(self, timeout: TimeoutPolicy) -> GuardResult<LockGuard<T, Locked>> {
        let start = Instant::now();

        // Fast path: try immediate acquisition
        match self.try_lock() {
            Ok(locked) => return Ok(locked),
            Err(unlocked) => {
                // Check if we have no timeout
                if timeout.duration().is_none() {
                    return unlocked.lock();
                }

                // Spin-wait with backoff
                return Self::lock_with_timeout_spin(unlocked, timeout, start);
            }
        }
    }

    /// Internal: lock with timeout using spin-wait strategy
    fn lock_with_timeout_spin(
        mut unlocked: LockGuard<T, Unlocked>,
        timeout: TimeoutPolicy,
        start: Instant,
    ) -> GuardResult<LockGuard<T, Locked>> {
        let mut backoff = 1;
        const MAX_BACKOFF: u64 = 1000; // Max 1ms backoff

        loop {
            // Check timeout
            if timeout.is_expired(start) {
                return Err(GuardError::Timeout {
                    resource_type: "lock",
                    category: timeout.category(),
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    timeout_ms: timeout.duration().map(|d| d.as_millis() as u64),
                });
            }

            // Try to acquire
            match unlocked.try_lock() {
                Ok(locked) => return Ok(locked),
                Err(guard) => {
                    unlocked = guard;

                    // Exponential backoff with hint::spin_loop
                    for _ in 0..backoff {
                        std::hint::spin_loop();
                    }
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                }
            }
        }
    }
}

impl<T: Send + 'static> LockGuard<T, Locked> {
    /// Access the protected data
    ///
    /// # Type Safety
    ///
    /// Only available when State = Locked
    ///
    /// # Note
    ///
    /// This acquires the mutex each time. For better performance with multiple
    /// accesses, use `with()` or `with_mut()` instead.
    #[inline]
    pub fn access(&self) -> StdMutexGuard<'_, T> {
        self.data.lock().expect("Lock poisoned during access")
    }

    /// Mutably access the protected data
    ///
    /// Returns a guard that allows mutable access
    #[inline]
    pub fn access_mut(&self) -> StdMutexGuard<'_, T> {
        self.data.lock().expect("Lock poisoned during mutable access")
    }

    /// Release the lock, transitioning to Unlocked state
    pub fn unlock(self) -> LockGuard<T, Unlocked> {
        LockGuard {
            data: self.data,
            lock_held: false,
            metadata: self.metadata,
            poisoned: self.poisoned,
            poison_reason: self.poison_reason,
            _state: PhantomData,
        }
    }

    /// Execute a function with the locked data
    ///
    /// Automatically unlocks after function completes.
    /// This is more efficient than multiple `access()` calls as it holds
    /// the mutex for the duration of the closure.
    pub fn with<F, R>(self, f: F) -> (R, LockGuard<T, Unlocked>)
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.data.lock().expect("Lock poisoned in with()");
        let result = f(&mut *guard);
        drop(guard);
        let unlocked = self.unlock();
        (result, unlocked)
    }
}

impl<T: Send + 'static, S: LockState> Guard for LockGuard<T, S> {
    fn resource_type(&self) -> &'static str {
        if self.lock_held {
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
        if self.lock_held {
            self.lock_held = false;
            Ok(())
        } else {
            Err(GuardError::AlreadyReleased)
        }
    }
}

impl<T: Send + 'static, S: LockState> Recoverable for LockGuard<T, S> {
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

        // Clone the Arc before we lock
        let data_arc = guard1.data.clone();

        // Hold an actual mutex lock
        let _real_lock = guard1.data.lock().unwrap();

        let guard2 = LockGuard {
            data: data_arc.clone(),
            lock_held: false,
            metadata: GuardMetadata::new("lock"),
            poisoned: false,
            poison_reason: None,
            _state: PhantomData,
        };

        // Try lock should fail because we're holding _real_lock
        assert!(guard2.try_lock().is_err());

        drop(_real_lock);

        // Now should succeed
        let guard3 = LockGuard {
            data: data_arc.clone(),
            lock_held: false,
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

    #[test]
    fn test_lock_guard_timeout_success() {
        use std::time::Duration;

        let guard = LockGuard::new(42);
        let timeout = TimeoutPolicy::Lock(Duration::from_secs(1));

        let locked = guard.lock_timeout(timeout).unwrap();
        assert_eq!(*locked.access(), 42);
    }

    #[test]
    fn test_lock_guard_timeout_expires() {
        use std::time::Duration;
        use std::thread;

        let guard1 = LockGuard::new(100);
        let data = guard1.data.clone();

        // Hold lock in one thread by keeping an actual MutexGuard alive
        let _real_lock = guard1.data.lock().unwrap();

        // Try to acquire with short timeout in another context
        thread::spawn(move || {
            let guard2 = LockGuard {
                data: data.clone(),
                lock_held: false,
                metadata: GuardMetadata::new("lock"),
                poisoned: false,
                poison_reason: None,
                _state: PhantomData,
            };

            let timeout = TimeoutPolicy::Lock(Duration::from_millis(10));
            let result = guard2.lock_timeout(timeout);

            // Should timeout
            assert!(result.is_err());
            match result.err().unwrap() {
                GuardError::Timeout { .. } => {},
                _ => panic!("Expected timeout error"),
            }
        }).join().unwrap();

        drop(_real_lock);
    }

    #[test]
    fn test_lock_guard_no_timeout() {
        let guard = LockGuard::new(42);
        let locked = guard.lock_timeout(TimeoutPolicy::None).unwrap();
        assert_eq!(*locked.access(), 42);
    }
}
