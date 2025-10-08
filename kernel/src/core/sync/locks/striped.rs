/*!
 * Lock Striping Pattern
 * Reduces contention by partitioning locks across multiple stripes
 */

use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Lock-striped hash map for reduced contention
///
/// # Performance
///
/// - **Contention reduction**: N-way striping reduces lock contention by ~N
/// - **Typical stripe count**: 16-64 (balance between memory and contention)
/// - **Best for**: Per-process state with many processes
///
/// # Example
///
/// ```ignore
/// let fd_tables = StripedMap::new(32); // 32 stripes
///
/// // Different PIDs likely hash to different stripes
/// fd_tables.insert(pid1, FdTable::new());
/// fd_tables.insert(pid2, FdTable::new());
/// ```
pub struct StripedMap<K, V> {
    stripes: Vec<RwLock<HashMap<K, V>>>,
    stripe_mask: usize,
}

impl<K: Hash + Eq, V> StripedMap<K, V> {
    /// Create new striped map with specified stripe count
    ///
    /// `stripe_count` should be a power of 2 for optimal performance
    pub fn new(stripe_count: usize) -> Self {
        assert!(
            stripe_count > 0 && stripe_count.is_power_of_two(),
            "Stripe count must be a power of 2"
        );

        let mut stripes = Vec::with_capacity(stripe_count);
        for _ in 0..stripe_count {
            stripes.push(RwLock::new(HashMap::new()));
        }

        Self {
            stripes,
            stripe_mask: stripe_count - 1,
        }
    }

    /// Get stripe index for key (uses hash)
    #[inline]
    fn stripe_index(&self, key: &K) -> usize {
        let mut hasher = ahash::AHasher::default();
        key.hash(&mut hasher);
        (hasher.finish() as usize) & self.stripe_mask
    }

    /// Insert key-value pair
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let idx = self.stripe_index(&key);
        self.stripes[idx].write().insert(key, value)
    }

    /// Get value by key (read lock only)
    pub fn get<F, R>(&self, key: &K, f: F) -> Option<R>
    where
        F: FnOnce(&V) -> R,
    {
        let idx = self.stripe_index(key);
        let stripe = self.stripes[idx].read();
        stripe.get(key).map(f)
    }

    /// Get mutable access to value
    pub fn get_mut<F, R>(&self, key: &K, f: F) -> Option<R>
    where
        F: FnOnce(&mut V) -> R,
    {
        let idx = self.stripe_index(key);
        let mut stripe = self.stripes[idx].write();
        stripe.get_mut(key).map(f)
    }

    /// Remove key
    pub fn remove(&self, key: &K) -> Option<V> {
        let idx = self.stripe_index(key);
        self.stripes[idx].write().remove(key)
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &K) -> bool {
        let idx = self.stripe_index(key);
        self.stripes[idx].read().contains_key(key)
    }

    /// Get total number of entries across all stripes
    pub fn len(&self) -> usize {
        self.stripes.iter().map(|stripe| stripe.read().len()).sum()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.stripes.iter().all(|stripe| stripe.read().is_empty())
    }

    /// Clear all entries
    pub fn clear(&self) {
        for stripe in &self.stripes {
            stripe.write().clear();
        }
    }

    /// Iterate over all entries (acquires all read locks)
    pub fn iter<F>(&self, mut f: F)
    where
        F: FnMut(&K, &V),
    {
        for stripe in &self.stripes {
            let guard = stripe.read();
            for (k, v) in guard.iter() {
                f(k, v);
            }
        }
    }
}

impl<K: Hash + Eq, V> Default for StripedMap<K, V> {
    fn default() -> Self {
        Self::new(16) // 16 stripes by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_operations() {
        let map = StripedMap::new(8);

        map.insert("key1", 100);
        map.insert("key2", 200);

        assert_eq!(map.get(&"key1", |v| *v), Some(100));
        assert_eq!(map.get(&"key2", |v| *v), Some(200));
        assert_eq!(map.get(&"key3", |v| *v), None);

        assert_eq!(map.len(), 2);
        assert!(!map.is_empty());
    }

    #[test]
    fn test_update() {
        let map = StripedMap::new(8);

        map.insert("counter", 0);

        map.get_mut(&"counter", |v| *v += 10);
        assert_eq!(map.get(&"counter", |v| *v), Some(10));

        map.get_mut(&"counter", |v| *v *= 2);
        assert_eq!(map.get(&"counter", |v| *v), Some(20));
    }

    #[test]
    fn test_remove() {
        let map = StripedMap::new(8);

        map.insert(1, "one");
        map.insert(2, "two");

        assert_eq!(map.remove(&1), Some("one"));
        assert_eq!(map.len(), 1);
        assert_eq!(map.remove(&1), None);
    }

    #[test]
    fn test_concurrent_access() {
        let map = Arc::new(StripedMap::new(16));
        let mut handles = vec![];

        // Spawn threads to insert different keys
        for i in 0..16 {
            let map = map.clone();
            handles.push(thread::spawn(move || {
                for j in 0..1000 {
                    map.insert(i * 1000 + j, j);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(map.len(), 16_000);
    }

    #[test]
    fn test_stripe_distribution() {
        let map = StripedMap::new(8);

        // Insert many keys
        for i in 0..1000 {
            map.insert(i, i);
        }

        // Check that keys are distributed across stripes
        let mut counts = vec![0; 8];
        for (i, stripe) in map.stripes.iter().enumerate() {
            counts[i] = stripe.read().len();
        }

        // Each stripe should have roughly 1000/8 ≈ 125 keys
        // Allow some variance
        for count in counts {
            assert!(count > 50 && count < 250, "Bad distribution: {}", count);
        }
    }

    #[test]
    fn test_iter() {
        let map = StripedMap::new(4);

        map.insert(1, "one");
        map.insert(2, "two");
        map.insert(3, "three");

        let mut collected = Vec::new();
        map.iter(|k, v| {
            collected.push((*k, *v));
        });

        collected.sort();
        assert_eq!(collected, vec![(1, "one"), (2, "two"), (3, "three")]);
    }
}
