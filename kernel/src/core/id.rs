/*!
 * ID Generation System
 * Centralized ID management with type-safe wrappers and recycling support
 */

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

// ============================================================================
// Type-Safe ID Wrappers
// ============================================================================

/// Process ID (32-bit for performance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pid(pub u32);

/// File descriptor (32-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Fd(pub u32);

/// Socket descriptor (32-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SockFd(pub u32);

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Fd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for SockFd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// ID Generator Trait
// ============================================================================

/// Generic ID generator interface
pub trait IdGenerator<T> {
    /// Generate next ID
    fn next(&self) -> T;

    /// Recycle an ID for reuse
    fn recycle(&self, id: T);

    /// Get current counter value (for debugging)
    fn current(&self) -> T;
}

// ============================================================================
// Atomic Counter Generator
// ============================================================================

/// High-performance atomic counter for hot paths
///
/// # Performance
/// - Cache-line aligned to prevent false sharing
/// - Lock-free atomic operations
/// - Suitable for process IDs, file descriptors, etc.
#[repr(C, align(64))]
pub struct AtomicGenerator<T> {
    counter: Arc<AtomicU64>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> AtomicGenerator<T> {
    /// Create new generator starting at given value
    #[inline]
    pub fn new(start: u64) -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(start)),
            _marker: std::marker::PhantomData,
        }
    }

    /// Create new generator starting at 1
    #[inline]
    pub fn default_start() -> Self {
        Self::new(1)
    }
}

impl<T> Clone for AtomicGenerator<T> {
    fn clone(&self) -> Self {
        Self {
            counter: Arc::clone(&self.counter),
            _marker: std::marker::PhantomData,
        }
    }
}

// Specialized implementations for different ID types
impl IdGenerator<u32> for AtomicGenerator<u32> {
    #[inline]
    fn next(&self) -> u32 {
        self.counter.fetch_add(1, Ordering::SeqCst) as u32
    }

    #[inline]
    fn recycle(&self, _id: u32) {
        // Atomic counter doesn't support recycling
        // Use RecyclingGenerator if recycling is needed
    }

    #[inline]
    fn current(&self) -> u32 {
        self.counter.load(Ordering::Relaxed) as u32
    }
}

impl IdGenerator<u64> for AtomicGenerator<u64> {
    #[inline]
    fn next(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    #[inline]
    fn recycle(&self, _id: u64) {
        // Atomic counter doesn't support recycling
    }

    #[inline]
    fn current(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }
}

// ============================================================================
// Recycling Generator
// ============================================================================

/// ID generator with recycling support
///
/// Uses a lock-free queue for recycled IDs and falls back to atomic counter
/// when no recycled IDs are available.
///
/// # Performance
/// - Lock-free ID recycling via SegQueue
/// - Prevents ID exhaustion in long-running systems
/// - Suitable for pipes, shared memory, queues
pub struct RecyclingGenerator<T> {
    counter: Arc<AtomicU32>,
    free_list: Arc<crossbeam_queue::SegQueue<T>>,
}

impl<T> RecyclingGenerator<T> {
    /// Create new recycling generator
    #[inline]
    pub fn new(start: u32) -> Self {
        Self {
            counter: Arc::new(AtomicU32::new(start)),
            free_list: Arc::new(crossbeam_queue::SegQueue::new()),
        }
    }

    /// Create new generator starting at 1
    #[inline]
    pub fn default_start() -> Self {
        Self::new(1)
    }
}

impl<T> Clone for RecyclingGenerator<T> {
    fn clone(&self) -> Self {
        Self {
            counter: Arc::clone(&self.counter),
            free_list: Arc::clone(&self.free_list),
        }
    }
}

impl IdGenerator<u32> for RecyclingGenerator<u32> {
    #[inline]
    fn next(&self) -> u32 {
        // Try to recycle first, otherwise allocate new
        self.free_list
            .pop()
            .unwrap_or_else(|| self.counter.fetch_add(1, Ordering::SeqCst))
    }

    #[inline]
    fn recycle(&self, id: u32) {
        self.free_list.push(id);
    }

    #[inline]
    fn current(&self) -> u32 {
        self.counter.load(Ordering::Relaxed)
    }
}

// ============================================================================
// Convenience Type Aliases
// ============================================================================

/// Process ID generator (no recycling - processes are long-lived)
pub type PidGenerator = AtomicGenerator<u32>;

/// File descriptor generator (recycling enabled)
pub type FdGenerator = RecyclingGenerator<u32>;

/// Pipe ID generator (recycling enabled)
pub type PipeIdGenerator = RecyclingGenerator<u32>;

/// Shared memory ID generator (recycling enabled)
pub type ShmIdGenerator = RecyclingGenerator<u32>;

/// Queue ID generator (recycling enabled)
pub type QueueIdGenerator = RecyclingGenerator<u32>;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_generator() {
        let gen = AtomicGenerator::<u32>::new(100);

        assert_eq!(gen.next(), 100);
        assert_eq!(gen.next(), 101);
        assert_eq!(gen.next(), 102);
        assert_eq!(gen.current(), 103);
    }

    #[test]
    fn test_recycling_generator() {
        let gen = RecyclingGenerator::<u32>::new(1);

        let id1 = gen.next(); // 1
        let id2 = gen.next(); // 2
        let id3 = gen.next(); // 3

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);

        // Recycle id2
        gen.recycle(id2);

        // Next should return recycled ID
        assert_eq!(gen.next(), 2);
        assert_eq!(gen.next(), 4); // Back to counter
    }

    #[test]
    fn test_concurrent_generation() {
        use std::sync::Arc;
        use std::thread;

        let gen = Arc::new(AtomicGenerator::<u32>::new(1));
        let mut handles = vec![];

        for _ in 0..10 {
            let g = Arc::clone(&gen);
            handles.push(thread::spawn(move || {
                let mut ids = vec![];
                for _ in 0..100 {
                    ids.push(g.next());
                }
                ids
            }));
        }

        let mut all_ids = vec![];
        for handle in handles {
            all_ids.extend(handle.join().unwrap());
        }

        // Check uniqueness
        all_ids.sort_unstable();
        let unique_count = all_ids.windows(2).filter(|w| w[0] != w[1]).count() + 1;
        assert_eq!(unique_count, 1000);
    }

    #[test]
    fn test_pid_display() {
        let pid = Pid(42);
        assert_eq!(format!("{}", pid), "42");
    }
}

