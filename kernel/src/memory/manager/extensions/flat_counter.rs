/*!
 * Flat Combining Counter Integration for MemoryManager
 * Optimized atomic counter wrapper for high-contention scenarios
 */

use crate::core::FlatCombiningCounter;
use std::sync::atomic::Ordering;

/// Wrapper for memory manager counters with flat combining optimization
///
/// This wrapper provides the same API as AtomicU64 but uses flat combining
/// internally for better performance under high contention (8+ cores).
pub struct MemoryCounter {
    inner: FlatCombiningCounter,
}

impl MemoryCounter {
    /// Create new counter with initial value
    #[inline]
    pub fn new(initial: u64) -> Self {
        Self {
            inner: FlatCombiningCounter::new(initial),
        }
    }

    /// Add to counter (optimized for high contention)
    #[inline]
    pub fn fetch_add(&self, delta: u64, order: Ordering) -> u64 {
        self.inner.fetch_add(delta, order)
    }

    /// Subtract from counter (optimized for high contention)
    #[inline]
    pub fn fetch_sub(&self, delta: u64, order: Ordering) -> u64 {
        self.inner.fetch_sub(delta, order)
    }

    /// Load current value (lock-free, no combining)
    #[inline(always)]
    pub fn load(&self, order: Ordering) -> u64 {
        self.inner.load(order)
    }

    /// Store new value (rare operation)
    #[inline]
    pub fn store(&self, val: u64, order: Ordering) {
        self.inner.store(val, order);
    }
}

impl Default for MemoryCounter {
    fn default() -> Self {
        Self::new(0)
    }
}

// Safety: FlatCombiningCounter is Sync and Send
unsafe impl Sync for MemoryCounter {}
unsafe impl Send for MemoryCounter {}
