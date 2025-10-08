/*!
 * Read-Copy-Update (RCU) Pattern
 * Zero-contention reads for read-heavy data structures
 */

use arc_swap::ArcSwap;
use std::sync::Arc;

/// RCU-protected data structure with zero-contention reads
///
/// # Performance
///
/// - **Reads**: Zero-contention, just atomic pointer load (~1-2ns)
/// - **Writes**: Clone-modify-swap (~100ns-1μs depending on size)
/// - **Best for**: Read:Write ratio > 100:1
///
/// # Example
///
/// ```ignore
/// use std::collections::HashMap;
///
/// let processes = RcuCell::new(HashMap::new());
///
/// // Read (zero-contention, lock-free)
/// if let Some(process) = processes.load().get(&pid) {
///     // Use process...
/// }
///
/// // Write (clone-modify-swap)
/// processes.update(|map| {
///     let mut new_map = (**map).clone();
///     new_map.insert(pid, process);
///     new_map
/// });
/// ```
///
/// # When to Use
///
/// ✅ **Use when**:
/// - Reads vastly outnumber writes (>100:1 ratio)
/// - Data structure is relatively small (<10MB)
/// - Occasional write latency is acceptable
/// - Examples: Process map, VFS mounts, sandbox rules
///
/// ❌ **Don't use when**:
/// - Frequent writes
/// - Very large data structures (>100MB)
/// - Write latency is critical
/// - Need precise write ordering
pub struct RcuCell<T> {
    inner: Arc<ArcSwap<T>>,
}

impl<T> RcuCell<T> {
    /// Create new RCU cell
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(ArcSwap::from_pointee(value)),
        }
    }

    /// Load current value (zero-contention)
    ///
    /// Uses ArcSwap's load for fast atomic access.
    #[inline(always)]
    pub fn load(&self) -> Arc<T> {
        Arc::clone(&*self.inner.load())
    }

    /// Load current value without cache (still lock-free)
    ///
    /// Use this if you need the absolute latest value and can afford
    /// a slightly slower atomic load.
    #[inline]
    pub fn load_full(&self) -> Arc<T> {
        self.inner.load_full()
    }

    /// Update value using a function
    ///
    /// The function receives the current value and should return a new value.
    /// This performs clone-modify-swap internally.
    /// Note: The function may be called multiple times if there's contention.
    #[inline]
    pub fn update<F>(&self, mut f: F)
    where
        T: Clone,
        F: FnMut(&T) -> T,
    {
        self.inner.rcu(|old| f(&*old));
    }

    /// Replace value entirely
    #[inline]
    pub fn store(&self, new_value: T) {
        self.inner.store(Arc::new(new_value));
    }

    /// Swap value and return old one
    #[inline]
    pub fn swap(&self, new_value: T) -> Arc<T> {
        self.inner.swap(Arc::new(new_value))
    }

    /// Compare and swap
    ///
    /// Returns Ok with new value if swap succeeded, Err with current value if failed.
    #[inline]
    pub fn compare_and_swap(&self, current: &Arc<T>, new_value: T) -> Result<Arc<T>, Arc<T>>
    where
        T: PartialEq,
    {
        let new_arc = Arc::new(new_value);
        let result = self.inner.compare_and_swap(current, Arc::clone(&new_arc));
        if Arc::ptr_eq(&*result, current) {
            Ok(new_arc)
        } else {
            Err(Arc::clone(&*result))
        }
    }
}

impl<T: Clone> Clone for RcuCell<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

// Safety: ArcSwap is Sync and Send when T is
unsafe impl<T: Send + Sync> Send for RcuCell<T> {}
unsafe impl<T: Send + Sync> Sync for RcuCell<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::thread;

    #[test]
    fn test_basic_read_write() {
        let cell = RcuCell::new(42);

        assert_eq!(*cell.load(), 42);

        cell.store(100);
        assert_eq!(*cell.load(), 100);
    }

    #[test]
    fn test_update() {
        let cell = RcuCell::new(10);

        cell.update(|val| val + 5);
        assert_eq!(*cell.load(), 15);

        cell.update(|val| val * 2);
        assert_eq!(*cell.load(), 30);
    }

    #[test]
    fn test_concurrent_reads() {
        let cell = Arc::new(RcuCell::new(HashMap::from([
            ("key1", 100),
            ("key2", 200),
        ])));

        let mut handles = vec![];

        // Spawn many reader threads (should have zero contention)
        for _ in 0..16 {
            let cell = cell.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..10_000 {
                    let map = cell.load();
                    assert_eq!(map.get("key1").copied(), Some(100));
                    assert_eq!(map.get("key2").copied(), Some(200));
                }
            }));
        }

        // Single writer thread
        let writer_cell = cell.clone();
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                writer_cell.update(|map| {
                    let mut new_map = map.clone();
                    new_map.insert("counter", i);
                    new_map
                });
                thread::sleep(std::time::Duration::from_micros(100));
            }
        }));

        for handle in handles {
            handle.join().unwrap();
        }

        let final_map = cell.load();
        assert_eq!(final_map.get("counter").copied(), Some(99));
    }

    #[test]
    fn test_compare_and_swap() {
        let cell = RcuCell::new(42);

        let current = cell.load();
        assert!(cell.compare_and_swap(&current, 100).is_ok());
        assert_eq!(*cell.load(), 100);

        // Should fail with stale value
        assert!(cell.compare_and_swap(&current, 200).is_err());
        assert_eq!(*cell.load(), 100); // Unchanged
    }

    #[test]
    fn test_swap() {
        let cell = RcuCell::new("initial");

        let old = cell.swap("new");
        assert_eq!(*old, "initial");
        assert_eq!(*cell.load(), "new");
    }

    #[test]
    fn test_cache_behavior() {
        let cell = RcuCell::new(1);

        // First load populates cache
        let val1 = cell.load();
        assert_eq!(*val1, 1);

        // Second load uses cache (should be same Arc)
        let val2 = cell.load();
        assert_eq!(*val2, 1);
        assert!(Arc::ptr_eq(&val1, &val2));

        // Update invalidates cache
        cell.store(2);
        let val3 = cell.load();
        assert_eq!(*val3, 2);
        assert!(!Arc::ptr_eq(&val1, &val3));
    }
}

