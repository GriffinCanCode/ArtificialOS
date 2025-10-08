/*!
 * Futex-Based Wait Strategy
 *
 * Uses parking_lot_core for futex-like operations on all platforms.
 * On Linux, this maps directly to futex syscalls for minimal overhead.
 *
 * # Design
 *
 * Follows Linux futex design: a fixed sharded hash table of parking slots.
 * - Zero allocations after initialization
 * - Guaranteed stable memory addresses
 * - Lock-free fast path
 * - Multiple keys can share a slot (spurious wakeups are acceptable)
 */

use super::traits::{WaitStrategy, WakeResult};
use parking_lot_core::{park, unpark_all, unpark_one, ParkResult, ParkToken, UnparkToken};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Number of parking slots (power of 2 for fast modulo via bitwise AND)
const PARKING_SLOTS: usize = 512;
const SLOT_MASK: usize = PARKING_SLOTS - 1;

/// A single parking slot with a waiter counter
#[repr(C, align(64))] // Cache-line aligned to prevent false sharing
struct ParkingSlot {
    waiters: AtomicUsize,
}

impl ParkingSlot {
    const fn new() -> Self {
        Self {
            waiters: AtomicUsize::new(0),
        }
    }
}

/// Futex-based wait strategy using sharded parking slots
///
/// # Performance
///
/// - Zero allocations after initialization
/// - Direct futex syscalls on Linux
/// - Lock-free fast path
/// - O(1) lookup via hash
#[repr(C, align(64))]
pub struct FutexWait<K>
where
    K: Eq + Hash + Copy + Send + Sync + 'static,
{
    /// Fixed array of parking slots (never resizes, stable addresses)
    slots: Box<[ParkingSlot; PARKING_SLOTS]>,
    _phantom: std::marker::PhantomData<K>,
}

impl<K> FutexWait<K>
where
    K: Eq + Hash + Copy + Send + Sync + 'static,
{
    /// Create a new futex-based wait strategy
    pub fn new() -> Self {
        // Initialize all slots (const init, very fast)
        Self {
            slots: Box::new([const { ParkingSlot::new() }; PARKING_SLOTS]),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Hash key to parking slot index
    #[inline]
    fn slot_index(&self, key: K) -> usize {
        let mut hasher = ahash::AHasher::default();
        key.hash(&mut hasher);
        (hasher.finish() as usize) & SLOT_MASK
    }
}

impl<K> Default for FutexWait<K>
where
    K: Eq + Hash + Copy + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K> WaitStrategy<K> for FutexWait<K>
where
    K: Eq + Hash + Copy + Send + Sync + 'static,
{
    fn wait(&self, key: K, timeout: Option<Duration>) -> bool {
        let idx = self.slot_index(key);
        let slot = &self.slots[idx];

        // Increment waiter count
        slot.waiters.fetch_add(1, Ordering::Relaxed);

        // Get stable parking address (same as in wake methods)
        let addr = &slot.waiters as *const AtomicUsize as usize;
        let start = Instant::now();
        let deadline = timeout.map(|d| Instant::now() + d);

        // Park the thread using parking_lot_core
        let result = unsafe {
            park(
                addr,
                || {
                    // Validate callback: always park
                    true
                },
                || {
                    // Before sleep callback
                },
                |_timed_out, _result| {
                    // After unpark callback
                },
                ParkToken(0),
                deadline,
            )
        };

        // Decrement waiter count
        slot.waiters.fetch_sub(1, Ordering::Relaxed);

        // Return true if woken, false if timeout
        match result {
            ParkResult::Unparked(_) => true,
            ParkResult::TimedOut => false, // Timed out, not woken
            ParkResult::Invalid => false,  // Invalid state, treat as not woken
        }
    }

    fn wake_one(&self, key: K) -> WakeResult {
        let idx = self.slot_index(key);
        let slot = &self.slots[idx];

        // Check if anyone is waiting
        let count = slot.waiters.load(Ordering::Relaxed);
        if count == 0 {
            return WakeResult::NoWaiters;
        }

        // Unpark one thread at this address
        let addr = &slot.waiters as *const AtomicUsize as usize;
        let result = unsafe { unpark_one(addr, |_| UnparkToken(0)) };

        WakeResult::Woken(result.unparked_threads)
    }

    fn wake_all(&self, key: K) -> WakeResult {
        let idx = self.slot_index(key);
        let slot = &self.slots[idx];

        // Check if anyone is waiting
        let count = slot.waiters.load(Ordering::Relaxed);
        if count == 0 {
            return WakeResult::NoWaiters;
        }

        // Unpark all threads at this address
        let addr = &slot.waiters as *const AtomicUsize as usize;
        let unparked = unsafe { unpark_all(addr, UnparkToken(0)) };

        WakeResult::Woken(unparked)
    }

    fn waiter_count(&self, key: K) -> usize {
        let idx = self.slot_index(key);
        self.slots[idx].waiters.load(Ordering::Relaxed)
    }

    fn name(&self) -> &'static str {
        "futex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_futex_wake_one() {
        let futex = Arc::new(FutexWait::<u64>::new());
        let futex_clone = futex.clone();

        let handle = thread::spawn(move || futex_clone.wait(42, Some(Duration::from_secs(1))));

        // Give thread time to park
        thread::sleep(Duration::from_millis(50));

        let result = futex.wake_one(42);
        assert_eq!(result, WakeResult::Woken(1));

        assert!(handle.join().unwrap());
    }

    #[test]
    fn test_futex_timeout() {
        let futex = FutexWait::<u64>::new();
        let start = Instant::now();
        let result = futex.wait(99, Some(Duration::from_millis(50)));
        let elapsed = start.elapsed();

        assert!(!result); // Should timeout
        assert!(elapsed >= Duration::from_millis(50));
    }
}

