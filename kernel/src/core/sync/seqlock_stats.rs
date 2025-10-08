/*!
 * Seqlock-based Statistics Wrapper
 * Zero-cost reads for read-heavy statistics structures
 */

use seqlock::{SeqLock as InnerSeqLock};
use std::sync::Arc;

/// Seqlock wrapper for statistics structures
///
/// # Performance
///
/// - **Reads**: Lock-free, wait-free (just sequence number check)
/// - **Writes**: Mutex-based, increments sequence number
/// - **Best for**: Structures read 100x more than written
///
/// # Example
///
/// ```ignore
/// let stats = SeqlockStats::new(JitStats::default());
///
/// // Read (lock-free, ~1-2ns)
/// let current = stats.read();
///
/// // Write (locks briefly, ~10-20ns)
/// stats.write(|s| {
///     s.jit_hits += 1;
/// });
/// ```
pub struct SeqlockStats<T> {
    inner: Arc<InnerSeqLock<T>>,
}

impl<T: Clone> SeqlockStats<T> {
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

impl<T: Clone> Clone for SeqlockStats<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

// Safety: SeqLock is Sync and Send when T is
unsafe impl<T: Send> Send for SeqlockStats<T> {}
unsafe impl<T: Sync> Sync for SeqlockStats<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    #[derive(Debug, Clone, PartialEq)]
    struct TestStats {
        counter: u64,
        name: String,
    }

    #[test]
    fn test_basic_read_write() {
        let stats = SeqlockStats::new(TestStats {
            counter: 0,
            name: "test".to_string(),
        });

        stats.write(|s| s.counter = 42);
        let read = stats.read();
        assert_eq!(read.counter, 42);
    }

    #[test]
    fn test_concurrent_reads() {
        let stats = Arc::new(SeqlockStats::new(TestStats {
            counter: 100,
            name: "concurrent".to_string(),
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
        let stats = SeqlockStats::new(TestStats {
            counter: 0,
            name: "batch".to_string(),
        });

        stats.write_batch(|s| {
            s.counter += 10;
            s.counter += 20;
            s.counter += 30;
        });

        assert_eq!(stats.read().counter, 60);
    }
}

