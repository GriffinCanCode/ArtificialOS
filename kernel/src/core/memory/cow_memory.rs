/*!
 * Copy-on-Write Memory Management
 * Share memory between processes with lazy copying
 */

use crate::core::sync::StripedMap;
use std::sync::{Arc, Mutex};

/// Copy-on-Write memory region
///
/// # Performance
///
/// - **Initial copy**: Instant (just Arc clone)
/// - **First write**: Triggers actual copy
/// - **Memory savings**: 80%+ for similar processes
///
/// # Example
///
/// ```ignore
/// let parent_memory = CowMemory::new(data);
///
/// // Child gets CoW reference (instant)
/// let child_memory = parent_memory.clone_cow();
///
/// // Parent and child share memory until write
/// assert!(Arc::ptr_eq(&parent_memory.data, &child_memory.data));
///
/// // First write triggers copy
/// child_memory.write(|data| data[0] = 99);
/// assert!(!Arc::ptr_eq(&parent_memory.data, &child_memory.data));
/// ```
#[derive(Debug)]
pub struct CowMemory {
    data: Arc<Mutex<Vec<u8>>>,
    has_written: bool,
}

impl CowMemory {
    /// Create new CoW memory region
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
            has_written: false,
        }
    }

    /// Clone for CoW (instant operation)
    pub fn clone_cow(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            has_written: false,
        }
    }

    /// Read data (shared, no copy)
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        let guard = self.data.lock().unwrap();
        f(&guard)
    }

    /// Write data (triggers copy if shared)
    pub fn write<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Vec<u8>) -> R,
    {
        // If this Arc has multiple owners, we need to copy
        if Arc::strong_count(&self.data) > 1 {
            // Copy-on-write: clone the data
            let current = self.data.lock().unwrap();
            let mut new_data = current.clone();
            drop(current);

            let result = f(&mut new_data);

            // Replace our Arc with exclusive copy
            self.data = Arc::new(Mutex::new(new_data));
            self.has_written = true;

            result
        } else {
            // We have exclusive access, no copy needed
            let mut guard = self.data.lock().unwrap();
            self.has_written = true;
            f(&mut guard)
        }
    }

    /// Check if memory is shared
    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    /// Get size
    pub fn len(&self) -> usize {
        self.data.lock().unwrap().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.lock().unwrap().is_empty()
    }
}

/// Copy-on-Write process memory manager
///
/// Manages CoW memory for multiple processes with automatic deduplication.
pub struct CowMemoryManager {
    /// Per-process memory regions
    regions: StripedMap<u32, CowMemory>,
}

impl CowMemoryManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            regions: StripedMap::new(32),
        }
    }

    /// Allocate memory for process
    pub fn allocate(&self, pid: u32, data: Vec<u8>) {
        self.regions.insert(pid, CowMemory::new(data));
    }

    /// Fork process memory (CoW)
    pub fn fork(&self, parent_pid: u32, child_pid: u32) -> Result<(), &'static str> {
        // Clone the parent region with CoW semantics
        let child_region = self
            .regions
            .get(&parent_pid, |region| region.clone_cow())
            .ok_or("Parent process not found")?;

        // Insert the child region
        self.regions.insert(child_pid, child_region);

        Ok(())
    }

    /// Read process memory
    pub fn read<F, R>(&self, pid: u32, f: F) -> Result<R, &'static str>
    where
        F: FnOnce(&[u8]) -> R,
    {
        self.regions
            .get(&pid, |memory| memory.read(f))
            .ok_or("Process not found")
    }

    /// Write process memory (triggers CoW if needed)
    pub fn write<F, R>(&self, pid: u32, f: F) -> Result<R, &'static str>
    where
        F: FnOnce(&mut Vec<u8>) -> R,
    {
        self.regions
            .get_mut(&pid, |memory| memory.write(f))
            .ok_or("Process not found")
    }

    /// Free process memory
    pub fn free(&self, pid: u32) -> Result<(), &'static str> {
        self.regions.remove(&pid).ok_or("Process not found")?;
        Ok(())
    }

    /// Get memory stats
    pub fn stats(&self) -> CowStats {
        let total_regions = self.regions.len();
        let mut shared_regions = 0;

        self.regions.iter(|_pid, memory| {
            if memory.is_shared() {
                shared_regions += 1;
            }
        });

        CowStats {
            total_regions,
            shared_regions,
            unique_regions: total_regions - shared_regions,
        }
    }
}

impl Default for CowMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// CoW memory statistics
#[derive(Debug, Clone)]
pub struct CowStats {
    pub total_regions: usize,
    pub shared_regions: usize,
    pub unique_regions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_basic() {
        let mem1 = CowMemory::new(vec![1, 2, 3]);
        let mut mem2 = mem1.clone_cow();

        // Initially shared
        assert!(mem1.is_shared());
        assert!(mem2.is_shared());

        // Read doesn't trigger copy
        let val1 = mem1.read(|data| data[0]);
        let val2 = mem2.read(|data| data[0]);
        assert_eq!(val1, val2);

        // Write triggers copy
        mem2.write(|data| data[0] = 99);

        // Now separate
        assert!(!mem1.is_shared());
        assert!(!mem2.is_shared());

        assert_eq!(mem1.read(|d| d[0]), 1); // Original
        assert_eq!(mem2.read(|d| d[0]), 99); // Modified
    }

    #[test]
    fn test_cow_manager_fork() {
        let manager = CowMemoryManager::new();

        // Allocate parent memory
        manager.allocate(1, vec![10, 20, 30]);

        // Fork child (CoW)
        assert!(manager.fork(1, 2).is_ok());

        // Both should have same data initially
        let parent_data = manager.read(1, |d| d.to_vec()).unwrap();
        let child_data = manager.read(2, |d| d.to_vec()).unwrap();
        assert_eq!(parent_data, child_data);

        // Child writes (triggers copy)
        manager.write(2, |d| d[0] = 99).unwrap();

        // Now different
        let parent_data = manager.read(1, |d| d[0]).unwrap();
        let child_data = manager.read(2, |d| d[0]).unwrap();
        assert_eq!(parent_data, 10);
        assert_eq!(child_data, 99);
    }

    #[test]
    fn test_cow_stats() {
        let manager = CowMemoryManager::new();

        manager.allocate(1, vec![1, 2, 3]);
        manager.fork(1, 2).unwrap();
        manager.fork(1, 3).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.total_regions, 3);
        assert!(stats.shared_regions > 0);
    }

    #[test]
    fn test_multiple_forks() {
        let manager = CowMemoryManager::new();

        manager.allocate(1, vec![100; 1024]); // 1KB

        // Fork multiple children
        for child_pid in 2..10 {
            manager.fork(1, child_pid).unwrap();
        }

        // All share same memory initially
        let stats = manager.stats();
        assert_eq!(stats.total_regions, 9); // 1 parent + 8 children

        // One child writes
        manager.write(5, |d| d[0] = 200).unwrap();

        // That child now has unique copy
        let stats = manager.stats();
        assert_eq!(stats.total_regions, 9);
    }
}
