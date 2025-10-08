/*!
 * Memory Guards
 *
 * RAII guards for scoped memory allocations with automatic cleanup
 */

use super::traits::{Guard, GuardDrop, GuardRef, Observable};
use super::{GuardError, GuardMetadata, GuardResult};
use crate::core::types::{Address, Pid, Size};
use crate::memory::manager::MemoryManager;
use crate::monitoring::{Category, Collector, Event, Payload, Severity};
use std::sync::Arc;

/// Scoped memory guard with automatic deallocation
///
/// # Example
///
/// ```ignore
/// let guard = memory_manager.allocate_guard(1024, pid)?;
/// let addr = guard.address();
/// // Use memory...
/// // Automatically freed on drop
/// ```
pub struct MemoryGuard {
    address: Address,
    size: Size,
    pid: Pid,
    manager: Arc<MemoryManager>,
    metadata: GuardMetadata,
    active: bool,
    collector: Option<Arc<Collector>>,
}

impl MemoryGuard {
    /// Create a new memory guard
    #[inline]
    pub fn new(
        address: Address,
        size: Size,
        pid: Pid,
        manager: Arc<MemoryManager>,
        collector: Option<Arc<Collector>>,
    ) -> Self {
        let metadata = GuardMetadata::new("memory").with_pid(pid).with_size(size);

        let guard = Self {
            address,
            size,
            pid,
            manager,
            metadata,
            active: true,
            collector,
        };

        guard.emit_created();
        guard
    }

    /// Get memory address
    #[inline]
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get allocation size
    #[inline]
    pub fn size(&self) -> Size {
        self.size
    }

    /// Get process ID
    #[inline]
    pub fn pid(&self) -> Pid {
        self.pid
    }

    /// Manually release without waiting for Drop
    ///
    /// Useful for early cleanup
    pub fn release_early(mut self) -> GuardResult<()> {
        self.release()?;
        // Prevent Drop from running
        std::mem::forget(self);
        Ok(())
    }
}

impl Guard for MemoryGuard {
    fn resource_type(&self) -> &'static str {
        "memory"
    }

    fn metadata(&self) -> &GuardMetadata {
        &self.metadata
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn release(&mut self) -> GuardResult<()> {
        if !self.active {
            return Err(GuardError::AlreadyReleased);
        }

        self.active = false;
        self.manager
            .deallocate(self.address)
            .map_err(|e| GuardError::OperationFailed(e.to_string()))?;

        self.emit_dropped();
        Ok(())
    }
}

impl GuardDrop for MemoryGuard {
    fn on_drop(&mut self) {
        if self.active {
            if let Err(e) = self.release() {
                log::error!(
                    "Memory guard drop failed for PID {}, address {}: {}",
                    self.pid,
                    self.address,
                    e
                );
                self.emit_error(&e);
            }
        }
    }
}

impl Observable for MemoryGuard {
    fn emit_created(&self) {
        if let Some(ref collector) = self.collector {
            let event = Event::new(
                Severity::Debug,
                Category::Memory,
                Payload::MemoryAllocated {
                    size: self.size,
                    region_id: self.address as u64,
                },
            ).with_pid(self.pid);
            collector.emit(event);
        }
    }

    fn emit_used(&self, operation: &str) {
        if let Some(ref collector) = self.collector {
            let event = Event::new(
                Severity::Debug,
                Category::Memory,
                Payload::MetricUpdate {
                    name: "memory_guard_used".to_string(),
                    value: 1.0,
                    labels: vec![
                        ("pid".to_string(), self.pid.to_string()),
                        ("address".to_string(), self.address.to_string()),
                        ("operation".to_string(), operation.to_string()),
                    ],
                },
            ).with_pid(self.pid);
            collector.emit(event);
        }
    }

    fn emit_dropped(&self) {
        if let Some(ref collector) = self.collector {
            let _lifetime = self.metadata.lifetime_micros();
            let event = Event::new(
                Severity::Debug,
                Category::Memory,
                Payload::MemoryFreed {
                    size: self.size,
                    region_id: self.address as u64,
                },
            ).with_pid(self.pid);
            collector.emit(event);
        }
    }

    fn emit_error(&self, error: &GuardError) {
        if let Some(ref collector) = self.collector {
            let event = Event::new(
                Severity::Error,
                Category::Memory,
                Payload::MetricUpdate {
                    name: "memory_guard_error".to_string(),
                    value: 1.0,
                    labels: vec![
                        ("pid".to_string(), self.pid.to_string()),
                        ("address".to_string(), self.address.to_string()),
                        ("error".to_string(), error.to_string()),
                    ],
                },
            ).with_pid(self.pid);
            collector.emit(event);
        }
    }
}

impl Drop for MemoryGuard {
    #[inline]
    fn drop(&mut self) {
        self.on_drop();
    }
}

/// Reference-counted memory guard for shared ownership
///
/// Multiple references can exist; memory freed when last ref drops
pub struct MemoryGuardRef {
    inner: Arc<std::sync::Mutex<MemoryGuardState>>,
    metadata: GuardMetadata,
}

struct MemoryGuardState {
    address: Address,
    size: Size,
    pid: Pid,
    manager: Arc<MemoryManager>,
    active: bool,
    collector: Option<Arc<Collector>>,
}

impl MemoryGuardRef {
    /// Create a new reference-counted memory guard
    pub fn new(
        address: Address,
        size: Size,
        pid: Pid,
        manager: Arc<MemoryManager>,
        collector: Option<Arc<Collector>>,
    ) -> Self {
        let metadata = GuardMetadata::new("memory_ref")
            .with_pid(pid)
            .with_size(size);

        let state = MemoryGuardState {
            address,
            size,
            pid,
            manager,
            active: true,
            collector,
        };

        Self {
            inner: Arc::new(std::sync::Mutex::new(state)),
            metadata,
        }
    }

    /// Get memory address
    pub fn address(&self) -> Address {
        self.inner.lock().unwrap().address
    }

    /// Get allocation size
    pub fn size(&self) -> Size {
        self.inner.lock().unwrap().size
    }

    /// Get process ID
    pub fn pid(&self) -> Pid {
        self.inner.lock().unwrap().pid
    }
}

impl Guard for MemoryGuardRef {
    fn resource_type(&self) -> &'static str {
        "memory_ref"
    }

    fn metadata(&self) -> &GuardMetadata {
        &self.metadata
    }

    fn is_active(&self) -> bool {
        self.inner.lock().unwrap().active
    }

    fn release(&mut self) -> GuardResult<()> {
        let mut state = self.inner.lock().unwrap();
        if !state.active {
            return Err(GuardError::AlreadyReleased);
        }

        state.active = false;
        state
            .manager
            .deallocate(state.address)
            .map_err(|e| GuardError::OperationFailed(e.to_string()))?;

        Ok(())
    }
}

impl GuardRef for MemoryGuardRef {
    fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl Clone for MemoryGuardRef {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            metadata: self.metadata.clone(),
        }
    }
}

impl Drop for MemoryGuardRef {
    fn drop(&mut self) {
        if self.is_last_ref() {
            let _ = self.release();
        }
    }
}

/// Extension trait for MemoryManager to create guards
pub trait MemoryGuardExt {
    /// Allocate memory with a guard
    fn allocate_guard(
        &self,
        size: Size,
        pid: Pid,
    ) -> Result<MemoryGuard, crate::memory::types::MemoryError>;

    /// Allocate memory with a reference-counted guard
    fn allocate_guard_ref(
        &self,
        size: Size,
        pid: Pid,
    ) -> Result<MemoryGuardRef, crate::memory::types::MemoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_guard_lifecycle() {
        let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
        let pid = 1;
        let size = 1024;

        // Allocate memory
        let address = manager.allocate(size, pid).unwrap();

        // Create guard
        let guard = MemoryGuard::new(address, size, pid, manager.clone(), None);

        assert!(guard.is_active());
        assert_eq!(guard.address(), address);
        assert_eq!(guard.size(), size);

        // Guard will auto-release on drop
    }

    #[test]
    fn test_memory_guard_ref() {
        let manager = Arc::new(MemoryManager::with_capacity(1024 * 1024));
        let pid = 1;
        let size = 512;

        let address = manager.allocate(size, pid).unwrap();
        let guard1 = MemoryGuardRef::new(address, size, pid, manager.clone(), None);

        assert_eq!(guard1.ref_count(), 1);

        let guard2 = guard1.clone();
        assert_eq!(guard1.ref_count(), 2);
        assert_eq!(guard2.ref_count(), 2);

        drop(guard1);
        assert_eq!(guard2.ref_count(), 1);
        assert!(guard2.is_last_ref());
    }
}
