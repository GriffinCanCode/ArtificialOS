/*!
 * Epoch-Based File Descriptor Table
 * Lock-free, wait-free FD lookups using epoch-based reclamation
 */

use crossbeam_epoch::{self as epoch, Atomic, Owned, Shared};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// File descriptor entry (generic over handle type)
pub struct FdEntry<T> {
    pub handle: Arc<T>,
    pub flags: u32,
}

/// Lock-free FD table with epoch-based reclamation
///
/// # Performance
///
/// - **Reads**: Wait-free, ~2-5ns
/// - **Writes**: Lock-free, ~20-50ns
/// - **Memory reclamation**: Deferred via epochs
///
/// # Safety
///
/// Uses crossbeam-epoch to ensure memory safety. Dropped entries
/// are only deallocated after all readers have finished.
pub struct EpochFdTable<T> {
    entries: Vec<Atomic<FdEntry<T>>>,
    size: AtomicUsize,
}

impl<T> EpochFdTable<T> {
    /// Create new FD table with fixed capacity
    ///
    /// Capacity should be set to max FDs (typically 1024 or 4096)
    pub fn with_capacity(capacity: usize) -> Self {
        let mut entries = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            entries.push(Atomic::null());
        }

        Self {
            entries,
            size: AtomicUsize::new(0),
        }
    }

    /// Get entry for FD (wait-free)
    ///
    /// # Performance
    /// This is the hot path - optimized for maximum speed
    #[inline(always)]
    pub fn get(&self, fd: usize) -> Option<Arc<T>> {
        if fd >= self.entries.len() {
            return None;
        }

        // Enter epoch (very cheap - just bumps thread-local counter)
        let guard = epoch::pin();

        // Load pointer (atomic load, wait-free)
        let entry_ptr = self.entries[fd].load(Ordering::Acquire, &guard);

        if entry_ptr.is_null() {
            return None;
        }

        // Safe to dereference - epoch ensures it won't be freed
        unsafe {
            let entry = entry_ptr.deref();
            Some(Arc::clone(&entry.handle))
        }
    }

    /// Insert entry for FD (lock-free)
    pub fn insert(&self, fd: usize, handle: Arc<T>, flags: u32) -> Result<(), &'static str> {
        if fd >= self.entries.len() {
            return Err("FD out of bounds");
        }

        let guard = epoch::pin();

        let new_entry = Owned::new(FdEntry { handle, flags });
        let old = self.entries[fd].swap(new_entry, Ordering::AcqRel, &guard);

        if old.is_null() {
            self.size.fetch_add(1, Ordering::Relaxed);
        } else {
            // Defer deallocation of old entry
            unsafe {
                guard.defer_destroy(old);
            }
        }

        Ok(())
    }

    /// Remove entry for FD (lock-free)
    pub fn remove(&self, fd: usize) -> Option<Arc<T>> {
        if fd >= self.entries.len() {
            return None;
        }

        let guard = epoch::pin();

        let old = self.entries[fd].swap(Owned::null(), Ordering::AcqRel, &guard);

        if old.is_null() {
            return None;
        }

        self.size.fetch_sub(1, Ordering::Relaxed);

        // Extract handle before deferring destruction
        let handle = unsafe { Arc::clone(&old.deref().handle) };

        unsafe {
            guard.defer_destroy(old);
        }

        Some(handle)
    }

    /// Get entry with flags (wait-free)
    #[inline]
    pub fn get_with_flags(&self, fd: usize) -> Option<(Arc<T>, u32)> {
        if fd >= self.entries.len() {
            return None;
        }

        let guard = epoch::pin();
        let entry_ptr = self.entries[fd].load(Ordering::Acquire, &guard);

        if entry_ptr.is_null() {
            return None;
        }

        unsafe {
            let entry = entry_ptr.deref();
            Some((Arc::clone(&entry.handle), entry.flags))
        }
    }

    /// Update flags for FD (lock-free)
    pub fn update_flags(&self, fd: usize, flags: u32) -> Result<(), &'static str> {
        if fd >= self.entries.len() {
            return Err("FD out of bounds");
        }

        let guard = epoch::pin();

        // Load current entry
        let current_ptr = self.entries[fd].load(Ordering::Acquire, &guard);

        if current_ptr.is_null() {
            return Err("FD not found");
        }

        // Create new entry with updated flags
        let handle = unsafe { Arc::clone(&current_ptr.deref().handle) };
        let new_entry = Owned::new(FdEntry { handle, flags });

        // Try to swap (may fail if concurrent modification)
        match self.entries[fd].compare_exchange(
            current_ptr,
            new_entry,
            Ordering::AcqRel,
            Ordering::Acquire,
            &guard,
        ) {
            Ok(old) => {
                unsafe {
                    guard.defer_destroy(old);
                }
                Ok(())
            }
            Err(e) => {
                // CAS failed, someone else modified it
                drop(e);
                Err("Concurrent modification")
            }
        }
    }

    /// Get number of active FDs
    #[inline]
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    /// Check if table is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.entries.len()
    }

    /// Clear all entries (slow operation)
    pub fn clear(&self) {
        let guard = epoch::pin();

        for entry in &self.entries {
            let old = entry.swap(Owned::null(), Ordering::AcqRel, &guard);
            if !old.is_null() {
                unsafe {
                    guard.defer_destroy(old);
                }
            }
        }

        self.size.store(0, Ordering::Relaxed);
    }
}

// Safety: FdEntry and operations are thread-safe
unsafe impl<T: Send + Sync> Send for EpochFdTable<T> {}
unsafe impl<T: Send + Sync> Sync for EpochFdTable<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[derive(Debug, Clone)]
    struct FileHandle {
        path: String,
        offset: u64,
    }

    #[test]
    fn test_basic_operations() {
        let table = EpochFdTable::with_capacity(16);

        let handle = Arc::new(FileHandle {
            path: "/test".to_string(),
            offset: 0,
        });

        assert!(table.insert(3, handle.clone(), 0).is_ok());
        assert_eq!(table.len(), 1);

        let retrieved = table.get(3).unwrap();
        assert_eq!(retrieved.path, "/test");

        let removed = table.remove(3).unwrap();
        assert_eq!(removed.path, "/test");
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn test_concurrent_reads() {
        let table = Arc::new(EpochFdTable::with_capacity(64));

        // Insert some handles
        for i in 0..8 {
            let handle = Arc::new(FileHandle {
                path: format!("/file{}", i),
                offset: i,
            });
            table.insert(i, handle, 0).unwrap();
        }

        let mut handles = vec![];

        // Spawn many reader threads
        for _ in 0..16 {
            let table = table.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..10_000 {
                    for fd in 0..8 {
                        if let Some(handle) = table.get(fd) {
                            assert_eq!(handle.path, format!("/file{}", fd));
                        }
                    }
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_flags() {
        let table = EpochFdTable::with_capacity(16);

        let handle = Arc::new(FileHandle {
            path: "/test".to_string(),
            offset: 0,
        });

        table.insert(5, handle, 0x01).unwrap();

        let (retrieved, flags) = table.get_with_flags(5).unwrap();
        assert_eq!(retrieved.path, "/test");
        assert_eq!(flags, 0x01);

        table.update_flags(5, 0x02).unwrap();

        let (_, new_flags) = table.get_with_flags(5).unwrap();
        assert_eq!(new_flags, 0x02);
    }

    #[test]
    fn test_replace() {
        let table = EpochFdTable::with_capacity(16);

        let handle1 = Arc::new(FileHandle {
            path: "/first".to_string(),
            offset: 0,
        });

        let handle2 = Arc::new(FileHandle {
            path: "/second".to_string(),
            offset: 100,
        });

        table.insert(3, handle1, 0).unwrap();
        assert_eq!(table.len(), 1);

        table.insert(3, handle2, 0).unwrap();
        assert_eq!(table.len(), 1); // Still 1, replaced

        let retrieved = table.get(3).unwrap();
        assert_eq!(retrieved.path, "/second");
    }
}

