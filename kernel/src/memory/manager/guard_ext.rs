/*!
 * Memory Guard Extensions
 *
 * Extension trait to create RAII guards for memory allocations
 */

use super::MemoryManager;
use crate::core::guard::{MemoryGuard, MemoryGuardRef};
use crate::core::types::{Pid, Size};
use crate::memory::types::MemoryResult;
use std::sync::Arc;

/// Extension trait for creating memory guards
pub trait MemoryGuardExt {
    /// Allocate memory with an RAII guard for automatic cleanup
    ///
    /// # Example
    ///
    /// ```rust
    /// use ai_os_kernel::memory::manager::{MemoryManager, MemoryGuardExt};
    ///
    /// let manager = MemoryManager::new();
    /// let guard = manager.allocate_guard(1024, 1)?;
    /// let addr = guard.address();
    /// // Memory automatically freed on drop
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn allocate_guard(&self, size: Size, pid: Pid) -> MemoryResult<MemoryGuard>;

    /// Allocate memory with a reference-counted guard for shared ownership
    ///
    /// Multiple owners can hold references; memory is freed when the last
    /// reference is dropped.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ai_os_kernel::memory::manager::{MemoryManager, MemoryGuardExt};
    ///
    /// let manager = MemoryManager::new();
    /// let guard1 = manager.allocate_guard_ref(1024, 1)?;
    /// let guard2 = guard1.clone(); // Shared ownership
    /// // Memory freed when both guards drop
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn allocate_guard_ref(&self, size: Size, pid: Pid) -> MemoryResult<MemoryGuardRef>;
}

impl MemoryGuardExt for MemoryManager {
    fn allocate_guard(&self, size: Size, pid: Pid) -> MemoryResult<MemoryGuard> {
        // Allocate memory first
        let address = self.allocate(size, pid)?;

        // Create guard that will deallocate on drop
        Ok(MemoryGuard::new(
            address,
            size,
            pid,
            Arc::new(self.clone()),
            self.collector(),
        ))
    }

    fn allocate_guard_ref(&self, size: Size, pid: Pid) -> MemoryResult<MemoryGuardRef> {
        // Allocate memory first
        let address = self.allocate(size, pid)?;

        // Create reference-counted guard
        Ok(MemoryGuardRef::new(
            address,
            size,
            pid,
            Arc::new(self.clone()),
            self.collector(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Guard, GuardRef};

    #[test]
    fn test_allocate_guard() {
        let manager = MemoryManager::with_capacity(1024 * 1024);
        let pid = 1;
        let size = 1024;

        let guard = manager.allocate_guard(size, pid).unwrap();

        assert!(guard.is_active());
        assert_eq!(guard.size(), size);
        assert_eq!(guard.pid(), pid);

        let address = guard.address();
        assert!(manager.is_valid(address));

        // Guard will auto-cleanup on drop
    }

    #[test]
    fn test_allocate_guard_ref_shared() {
        let manager = MemoryManager::with_capacity(1024 * 1024);
        let pid = 1;

        let guard1 = manager.allocate_guard_ref(512, pid).unwrap();
        let address = guard1.address();

        assert_eq!(guard1.ref_count(), 1);
        assert!(manager.is_valid(address));

        let guard2 = guard1.clone();
        assert_eq!(guard1.ref_count(), 2);
        assert_eq!(guard2.ref_count(), 2);

        drop(guard1);
        assert_eq!(guard2.ref_count(), 1);
        assert!(manager.is_valid(address));

        // Memory freed when last ref drops
    }

    #[test]
    fn test_guard_prevents_leak_on_error() {
        let manager = MemoryManager::with_capacity(1024 * 1024);
        let pid = 1;

        fn operation_that_fails(
            manager: &MemoryManager,
            pid: Pid,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let _guard = manager.allocate_guard(1024, pid)?;

            // Simulate error
            return Err("Operation failed".into());

            // Guard automatically cleans up despite error
        }

        let used_before = manager
            .used_memory
            .load(std::sync::atomic::Ordering::SeqCst);
        let _ = operation_that_fails(&manager, pid);
        let used_after = manager
            .used_memory
            .load(std::sync::atomic::Ordering::SeqCst);

        // Memory should be freed despite error
        assert_eq!(used_before, used_after);
    }
}
