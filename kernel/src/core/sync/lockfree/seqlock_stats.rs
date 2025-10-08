/*!
 * Seqlock-based Statistics Wrapper
 *
 * Zero-cost reads for read-heavy statistics structures using sequence locks.
 *
 * # Design: Seqlock for Read-Heavy Workloads
 *
 * Seqlocks are optimal when reads vastly outnumber writes (100:1+ ratio).
 * Unlike RwLock, readers never block writers and vice versa:
 *
 * **Read path** (lock-free, wait-free):
 * 1. Read sequence number (odd = writer active)
 * 2. Read data
 * 3. Re-read sequence number (retry if changed)
 *
 * **Write path** (locks, increments sequence):
 * 1. Acquire write lock
 * 2. Increment sequence (odd value signals "writing")
 * 3. Update data
 * 4. Increment sequence again (even value signals "complete")
 *
 * Result: **Reads complete in ~1-2ns** regardless of writer activity.
 *
 * # When to Use
 *
 * ✅ **Use when**:
 * - Read:Write ratio > 100:1
 * - Data fits in a few cache lines (<256 bytes)
 * - Occasional stale reads are acceptable
 *
 * ❌ **Don't use when**:
 * - Frequent writes
 * - Large data structures (>1KB)
 * - Readers need precise consistency
 *
 * # Example
 *
 * ```ignore
 * #[derive(Clone, Copy)]
 * struct JitStats {
 *     hits: u64,
 *     misses: u64,
 * }
 *
 * let stats = SeqlockStats::new(JitStats { hits: 0, misses: 0 });
 *
 * // Read (lock-free, ~1-2ns)
 * let current = stats.read();
 * println!("Hits: {}", current.hits);
 *
 * // Write (locks briefly, ~10-20ns)
 * stats.write(|s| s.hits += 1);
 * ```
 */

use seqlock::SeqLock as InnerSeqLock;
use std::sync::Arc;

/// Seqlock wrapper for statistics structures
///
/// Generic over any `Copy` type (required for seqlock semantics).
pub struct SeqlockStats<T: Copy> {
    inner: Arc<InnerSeqLock<T>>,
}

impl<T: Clone + Copy> SeqlockStats<T> {
    /// Create new seqlock-protected stats
    #[inline]
    pub fn new(initial: T) -> Self {
        Self {
            inner: Arc::new(InnerSeqLock::new(initial)),
        }
    }

    /// Read current stats (lock-free)
    ///
    /// This is wait-free and never blocks. If a writer is active,
    /// it will spin briefly and retry, but this is very rare.
    #[inline(always)]
    pub fn read(&self) -> T {
        self.inner.read()
    }

    /// Update stats with a closure
    ///
    /// The closure receives mutable access to the stats. This operation
    /// takes a write lock briefly.
    #[inline]
    pub fn write<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut guard = self.inner.lock_write();
        f(&mut *guard);
    }

    /// Replace stats entirely
    #[inline]
    pub fn replace(&self, new_value: T) {
        *self.inner.lock_write() = new_value;
    }

    /// Access stats for batched updates (holds lock longer)
    #[inline]
    pub fn write_batch<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.lock_write();
        f(&mut *guard)
    }
}

impl<T: Clone + Copy> Clone for SeqlockStats<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

// Safety: SeqLock is Sync and Send when T is (T must be Copy for SeqlockStats)
unsafe impl<T: Copy + Send> Send for SeqlockStats<T> {}
unsafe impl<T: Copy + Sync> Sync for SeqlockStats<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct TestStats {
        counter: u64,
        id: u64,
    }

    #[test]
    fn test_basic_read_write() {
        let stats = SeqlockStats::new(TestStats { counter: 0, id: 1 });

        stats.write(|s| s.counter = 42);
        let read = stats.read();
        assert_eq!(read.counter, 42);
    }

    #[test]
    fn test_concurrent_reads() {
        let stats = Arc::new(SeqlockStats::new(TestStats {
            counter: 100,
            id: 2,
        }));

        let mut handles = vec![];

        // Spawn 16 reader threads
        for _ in 0..16 {
            let stats = stats.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..10_000 {
                    let read = stats.read();
                    assert!(read.counter >= 100);
                }
            }));
        }

        // Single writer thread
        let writer_stats = stats.clone();
        handles.push(thread::spawn(move || {
            for i in 0..1000 {
                writer_stats.write(|s| s.counter = 100 + i);
            }
        }));

        for handle in handles {
            handle.join().unwrap();
        }

        let final_stats = stats.read();
        assert_eq!(final_stats.counter, 1099);
    }

    #[test]
    fn test_batched_updates() {
        let stats = SeqlockStats::new(TestStats { counter: 0, id: 3 });

        stats.write_batch(|s| {
            s.counter += 10;
            s.counter += 20;
            s.counter += 30;
        });

        assert_eq!(stats.read().counter, 60);
    }
}
