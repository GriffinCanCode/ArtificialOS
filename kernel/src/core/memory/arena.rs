/*!
 * Arena Allocation for Request Lifecycle
 * Fast bump allocation with automatic cleanup
 */

use bumpalo::Bump;
use std::cell::RefCell;

// Thread-local arena for request-scoped allocations
//
// # Performance
//
// - **Allocation**: O(1), just bumps pointer
// - **Deallocation**: O(1), drops entire arena at once
// - **10-100x faster** than individual allocations
//
// # Example
//
// ```
// with_arena(|arena| {
//     // All allocations use arena
//     let vec = arena.alloc_vec();
//     vec.push(1);
//     vec.push(2);
//
//     let string = arena.alloc_str("temporary");
//
//     // Process request...
//
//     // Arena automatically freed when closure returns
// });
// ```
thread_local! {
    static ARENA: RefCell<Option<Bump>> = RefCell::new(None);
}

/// Execute closure with arena allocator
///
/// All allocations during closure lifetime use arena and are freed together.
#[inline]
pub fn with_arena<F, R>(f: F) -> R
where
    F: FnOnce(&Bump) -> R,
{
    ARENA.with(|cell| {
        let mut opt = cell.borrow_mut();

        // Reuse existing arena or create new one
        let arena = opt.get_or_insert_with(|| Bump::with_capacity(64 * 1024)); // 64KB

        arena.reset(); // Clear previous allocations
        let result = f(arena);

        result
    })
}

/// Arena-allocated vector
pub struct ArenaVec<'a, T> {
    arena: &'a Bump,
    items: bumpalo::collections::Vec<'a, T>,
}

impl<'a, T> ArenaVec<'a, T> {
    /// Create new arena vector
    pub fn new_in(arena: &'a Bump) -> Self {
        Self {
            arena,
            items: bumpalo::collections::Vec::new_in(arena),
        }
    }

    /// Push item
    #[inline]
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Get slice
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    /// Get length
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Arena-allocated string
pub struct ArenaString<'a> {
    arena: &'a Bump,
    string: bumpalo::collections::String<'a>,
}

impl<'a> ArenaString<'a> {
    /// Create new arena string
    pub fn new_in(arena: &'a Bump) -> Self {
        Self {
            arena,
            string: bumpalo::collections::String::new_in(arena),
        }
    }

    /// Push string slice
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.string.push_str(s);
    }

    /// Get string slice
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.string
    }

    /// Get length
    #[inline]
    pub fn len(&self) -> usize {
        self.string.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }
}

/// Helper trait for types that can be arena-allocated
pub trait ArenaAllocatable: Sized {
    /// Allocate in arena
    fn alloc_in(self, arena: &Bump) -> &mut Self {
        arena.alloc(self)
    }
}

impl<T> ArenaAllocatable for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic() {
        with_arena(|arena| {
            let x = arena.alloc(42);
            assert_eq!(*x, 42);

            let s = arena.alloc_str("test");
            assert_eq!(s, "test");
        });
    }

    #[test]
    fn test_arena_vec() {
        with_arena(|arena| {
            let mut vec = ArenaVec::new_in(arena);
            vec.push(1);
            vec.push(2);
            vec.push(3);

            assert_eq!(vec.as_slice(), &[1, 2, 3]);
            assert_eq!(vec.len(), 3);
        });
    }

    #[test]
    fn test_arena_string() {
        with_arena(|arena| {
            let mut s = ArenaString::new_in(arena);
            s.push_str("Hello");
            s.push_str(" ");
            s.push_str("World");

            assert_eq!(s.as_str(), "Hello World");
            assert_eq!(s.len(), 11);
        });
    }

    #[test]
    fn test_arena_reuse() {
        // First use
        with_arena(|arena| {
            let x = arena.alloc(100);
            assert_eq!(*x, 100);
        });

        // Second use - arena should be reset
        with_arena(|arena| {
            let y = arena.alloc(200);
            assert_eq!(*y, 200);
        });
    }

    #[test]
    fn test_many_allocations() {
        with_arena(|arena| {
            let mut vecs = Vec::new();

            for i in 0..1000 {
                let mut v = ArenaVec::new_in(arena);
                v.push(i);
                vecs.push(v);
            }

            assert_eq!(vecs.len(), 1000);
        });
    }
}
