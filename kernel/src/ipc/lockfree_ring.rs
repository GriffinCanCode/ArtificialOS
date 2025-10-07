/*!
 * Lock-Free Ring Buffer
 * SPSC (Single Producer Single Consumer) lock-free ring buffer for IPC hot paths
 */

use crossbeam_queue::ArrayQueue;
use std::sync::Arc;

/// Lock-free SPSC ring buffer optimized for IPC
///
/// # Performance
/// - Zero-contention single-producer single-consumer pattern
/// - Lock-free push/pop operations using atomics
/// - Cache-line aligned for optimal performance
/// - No memory allocations after initialization
///
/// # Thread Safety
/// Safe for one producer and one consumer thread only
#[derive(Clone)]
pub struct LockFreeRing<T> {
    queue: Arc<ArrayQueue<T>>,
}

impl<T> LockFreeRing<T> {
    /// Create a new lock-free ring buffer with the specified capacity
    ///
    /// # Performance
    /// Capacity must be > 0. The actual capacity may be rounded up to a power of two
    /// for optimal performance.
    #[inline]
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
        }
    }

    /// Push an item to the ring buffer (lock-free)
    ///
    /// Returns `Ok(())` if successful, `Err(item)` if the buffer is full.
    ///
    /// # Performance
    /// Hot path - lock-free atomic operation, no blocking
    #[inline(always)]
    pub fn push(&self, item: T) -> Result<(), T> {
        self.queue.push(item)
    }

    /// Pop an item from the ring buffer (lock-free)
    ///
    /// Returns `Some(item)` if successful, `None` if the buffer is empty.
    ///
    /// # Performance
    /// Hot path - lock-free atomic operation, no blocking
    #[inline(always)]
    pub fn pop(&self) -> Option<T> {
        self.queue.pop()
    }

    /// Check if the buffer is empty (lock-free)
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Check if the buffer is full (lock-free)
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    /// Get the current number of items in the buffer (approximate)
    ///
    /// # Note
    /// This is an approximate count due to concurrent access. The actual count
    /// may change immediately after this call returns.
    #[inline]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Get the capacity of the buffer
    #[inline]
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }

    /// Get available space in the buffer (approximate)
    #[inline]
    pub fn available(&self) -> usize {
        self.capacity().saturating_sub(self.len())
    }
}

/// Lock-free byte ring buffer for pipe-like IPC
///
/// # Performance
/// - Optimized for bulk byte transfers
/// - Lock-free push/pop operations
/// - Zero-copy slice operations where possible
pub struct LockFreeByteRing {
    ring: LockFreeRing<u8>,
}

impl LockFreeByteRing {
    /// Create a new lock-free byte ring buffer
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            ring: LockFreeRing::new(capacity),
        }
    }

    /// Write bytes to the buffer (lock-free)
    ///
    /// Returns the number of bytes written. May be less than requested if
    /// the buffer becomes full.
    ///
    /// # Performance
    /// Hot path - lock-free operation, no allocation
    pub fn write(&self, data: &[u8]) -> usize {
        let mut written = 0;
        for &byte in data {
            if self.ring.push(byte).is_err() {
                break;
            }
            written += 1;
        }
        written
    }

    /// Read bytes from the buffer (lock-free)
    ///
    /// Returns a Vec containing up to `size` bytes. May return fewer bytes
    /// if the buffer doesn't have enough data.
    ///
    /// # Performance
    /// Hot path - lock-free operation, allocates Vec for return
    pub fn read(&self, size: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            if let Some(byte) = self.ring.pop() {
                data.push(byte);
            } else {
                break;
            }
        }
        data
    }

    /// Get the number of buffered bytes (approximate)
    #[inline]
    pub fn buffered(&self) -> usize {
        self.ring.len()
    }

    /// Get available space for writing (approximate)
    #[inline]
    pub fn available_space(&self) -> usize {
        self.ring.available()
    }

    /// Check if buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    /// Get the capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.ring.capacity()
    }
}

impl Clone for LockFreeByteRing {
    fn clone(&self) -> Self {
        Self {
            ring: self.ring.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_push_pop() {
        let ring = LockFreeRing::<u32>::new(10);

        assert!(ring.push(1).is_ok());
        assert!(ring.push(2).is_ok());

        assert_eq!(ring.pop(), Some(1));
        assert_eq!(ring.pop(), Some(2));
        assert_eq!(ring.pop(), None);
    }

    #[test]
    fn test_capacity() {
        let ring = LockFreeRing::<u32>::new(5);
        assert_eq!(ring.capacity(), 5);

        for i in 0..5 {
            assert!(ring.push(i).is_ok());
        }

        assert!(ring.push(100).is_err());
    }

    #[test]
    fn test_byte_ring_write_read() {
        let ring = LockFreeByteRing::new(100);

        let data = b"Hello, World!";
        let written = ring.write(data);
        assert_eq!(written, data.len());

        let read_data = ring.read(data.len());
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_byte_ring_partial_write() {
        let ring = LockFreeByteRing::new(5);

        let data = b"Hello, World!";
        let written = ring.write(data);
        assert_eq!(written, 5); // Only 5 bytes fit

        let read_data = ring.read(5);
        assert_eq!(read_data, b"Hello");
    }
}
