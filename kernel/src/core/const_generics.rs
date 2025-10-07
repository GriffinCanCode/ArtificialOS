/*!
 * Const Generics Module
 * Compile-time optimized data structures using const generics for 2025 kernel
 */

#![allow(unused)]

use std::mem::MaybeUninit;

/// Fixed-size buffer with compile-time size checking
///
/// # Performance
/// - Zero runtime overhead - size checked at compile time
/// - Stack-allocated for fast access
/// - No heap allocations
///
/// # Type Parameters
/// - `T`: Element type
/// - `N`: Buffer capacity (compile-time constant)
#[derive(Debug)]
pub struct FixedBuffer<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedBuffer<T, N> {
    /// Create a new empty fixed buffer
    ///
    /// # Performance
    /// - Const function - can be evaluated at compile time
    /// - No runtime initialization cost
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    /// Get the capacity (compile-time constant)
    #[inline]
    #[must_use]
    pub const fn capacity() -> usize {
        N
    }

    /// Get current length
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Check if buffer is full
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// Get remaining capacity
    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> usize {
        N - self.len
    }

    /// Push an element to the buffer
    ///
    /// # Returns
    /// - `Ok(())` if element was pushed
    /// - `Err(value)` if buffer is full
    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len < N {
            self.data[self.len].write(value);
            self.len += 1;
            Ok(())
        } else {
            Err(value)
        }
    }

    /// Pop an element from the buffer
    ///
    /// # Returns
    /// - `Some(value)` if buffer had an element
    /// - `None` if buffer was empty
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(unsafe { self.data[self.len].assume_init_read() })
        } else {
            None
        }
    }

    /// Get a reference to an element
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            Some(unsafe { self.data[index].assume_init_ref() })
        } else {
            None
        }
    }

    /// Get a mutable reference to an element
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            Some(unsafe { self.data[index].assume_init_mut() })
        } else {
            None
        }
    }

    /// Clear the buffer
    #[inline]
    pub fn clear(&mut self) {
        for i in 0..self.len {
            unsafe {
                self.data[i].assume_init_drop();
            }
        }
        self.len = 0;
    }

    /// Get as slice
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const T,
                self.len,
            )
        }
    }

    /// Get as mutable slice
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr() as *mut T,
                self.len,
            )
        }
    }
}

impl<T, const N: usize> Drop for FixedBuffer<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N: usize> Default for FixedBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Fixed-size ring buffer with compile-time size checking
///
/// # Performance
/// - Constant-time push/pop operations
/// - No allocations - fully stack-allocated
/// - Power-of-2 sizes enable optimized indexing
#[derive(Debug)]
pub struct FixedRingBuffer<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    head: usize,
    tail: usize,
    full: bool,
}

impl<T, const N: usize> FixedRingBuffer<T, N> {
    /// Create a new empty ring buffer
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        // Compile-time check: N must be power of 2 for optimal performance
        // This enables bit-masking instead of modulo operations
        assert!(N > 0 && (N & (N - 1)) == 0, "N must be a power of 2");

        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            head: 0,
            tail: 0,
            full: false,
        }
    }

    /// Get the capacity (compile-time constant)
    #[inline]
    #[must_use]
    pub const fn capacity() -> usize {
        N
    }

    /// Get current length
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        if self.full {
            N
        } else if self.head >= self.tail {
            self.head - self.tail
        } else {
            N - (self.tail - self.head)
        }
    }

    /// Check if buffer is empty
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.full && self.head == self.tail
    }

    /// Check if buffer is full
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.full
    }

    /// Push an element to the buffer
    ///
    /// # Returns
    /// - `Ok(())` if element was pushed
    /// - `Err(value)` if buffer is full
    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.full {
            return Err(value);
        }

        self.data[self.head].write(value);
        self.head = (self.head + 1) & (N - 1); // Bit-mask for power-of-2 wrapping
        self.full = self.head == self.tail;
        Ok(())
    }

    /// Pop an element from the buffer
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let value = unsafe { self.data[self.tail].assume_init_read() };
        self.tail = (self.tail + 1) & (N - 1); // Bit-mask for power-of-2 wrapping
        self.full = false;
        Some(value)
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        while !self.is_empty() {
            self.pop();
        }
    }
}

impl<T, const N: usize> Drop for FixedRingBuffer<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N: usize> Default for FixedRingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache-line aligned fixed buffer for optimal cache performance
///
/// # Performance
/// - Aligned to 64-byte cache lines
/// - Prevents false sharing in multi-threaded scenarios
/// - Optimal for frequently accessed data structures
#[repr(C, align(64))]
#[derive(Debug)]
pub struct CacheAlignedBuffer<T, const N: usize> {
    inner: FixedBuffer<T, N>,
}

impl<T, const N: usize> CacheAlignedBuffer<T, N> {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: FixedBuffer::new(),
        }
    }

    // Delegate methods to inner buffer
    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), T> {
        self.inner.push(value)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T, const N: usize> Default for CacheAlignedBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Type-safe index for arrays with compile-time bounds checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypedIndex<const MAX: usize> {
    value: usize,
}

impl<const MAX: usize> TypedIndex<MAX> {
    /// Create a new index with runtime bounds checking
    #[inline]
    pub const fn new(value: usize) -> Option<Self> {
        if value < MAX {
            Some(Self { value })
        } else {
            None
        }
    }

    /// Create an index without bounds checking (unsafe)
    ///
    /// # Safety
    /// Caller must ensure `value < MAX`
    #[inline]
    pub const unsafe fn new_unchecked(value: usize) -> Self {
        Self { value }
    }

    /// Get the raw value
    #[inline]
    #[must_use]
    pub const fn get(self) -> usize {
        self.value
    }

    /// Get the maximum value (compile-time constant)
    #[inline]
    #[must_use]
    pub const fn max() -> usize {
        MAX
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_buffer() {
        let mut buf: FixedBuffer<u32, 4> = FixedBuffer::new();
        assert_eq!(FixedBuffer::<u32, 4>::capacity(), 4);
        assert!(buf.is_empty());

        assert!(buf.push(1).is_ok());
        assert!(buf.push(2).is_ok());
        assert_eq!(buf.len(), 2);

        assert_eq!(buf.pop(), Some(2));
        assert_eq!(buf.pop(), Some(1));
        assert!(buf.is_empty());
    }

    #[test]
    fn test_fixed_buffer_full() {
        let mut buf: FixedBuffer<u32, 2> = FixedBuffer::new();
        assert!(buf.push(1).is_ok());
        assert!(buf.push(2).is_ok());
        assert!(buf.is_full());
        assert!(buf.push(3).is_err());
    }

    #[test]
    fn test_ring_buffer() {
        let mut buf: FixedRingBuffer<u32, 4> = FixedRingBuffer::new();
        assert!(buf.is_empty());

        assert!(buf.push(1).is_ok());
        assert!(buf.push(2).is_ok());
        assert!(buf.push(3).is_ok());
        assert_eq!(buf.len(), 3);

        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.pop(), Some(2));
        assert!(buf.push(4).is_ok());
        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(4));
        assert!(buf.is_empty());
    }

    #[test]
    fn test_typed_index() {
        type Index10 = TypedIndex<10>;

        assert!(Index10::new(5).is_some());
        assert!(Index10::new(9).is_some());
        assert!(Index10::new(10).is_none());
        assert!(Index10::new(100).is_none());

        let idx = Index10::new(5).unwrap();
        assert_eq!(idx.get(), 5);
        assert_eq!(Index10::max(), 10);
    }

    #[test]
    fn test_cache_aligned() {
        let buf: CacheAlignedBuffer<u32, 8> = CacheAlignedBuffer::new();
        let ptr = &buf as *const _ as usize;
        assert_eq!(ptr % 64, 0, "Buffer should be 64-byte aligned");
    }
}
