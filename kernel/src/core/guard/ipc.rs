/*!
 * IPC Resource Guards
 *
 * RAII guards for IPC resources (pipes, queues, shared memory)
 */

use super::traits::{Guard, GuardDrop, GuardRef, Observable};
use super::{GuardError, GuardMetadata, GuardResult};
use crate::core::types::Pid;
use crate::monitoring::{Category, Collector, Event, Payload, Severity};
use std::sync::Arc;

/// IPC resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcResourceType {
    Pipe,
    Queue,
    SharedMemory,
    ZeroCopyRing,
}

impl IpcResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pipe => "pipe",
            Self::Queue => "queue",
            Self::SharedMemory => "shm",
            Self::ZeroCopyRing => "zerocopy_ring",
        }
    }
}

/// IPC resource guard with automatic cleanup
///
/// # Example
///
/// ```ignore
/// let pipe_guard = IpcGuard::new(
///     pipe_id,
///     IpcResourceType::Pipe,
///     pid,
///     cleanup_fn,
///     collector,
/// );
/// // Use pipe...
/// // Automatically cleaned up on drop
/// ```
pub struct IpcGuard {
    resource_id: u64,
    resource_type: IpcResourceType,
    pid: Pid,
    cleanup: Box<dyn Fn(u64) -> Result<(), String> + Send + Sync>,
    metadata: GuardMetadata,
    active: bool,
    collector: Option<Arc<Collector>>,
}

impl IpcGuard {
    /// Create a new IPC resource guard
    pub fn new<F>(
        resource_id: u64,
        resource_type: IpcResourceType,
        pid: Pid,
        cleanup: F,
        collector: Option<Arc<Collector>>,
    ) -> Self
    where
        F: Fn(u64) -> Result<(), String> + Send + Sync + 'static,
    {
        let metadata = GuardMetadata::new(resource_type.as_str()).with_pid(pid);

        let guard = Self {
            resource_id,
            resource_type,
            pid,
            cleanup: Box::new(cleanup),
            metadata,
            active: true,
            collector,
        };

        guard.emit_created();
        guard
    }

    /// Get resource ID
    #[inline]
    pub fn resource_id(&self) -> u64 {
        self.resource_id
    }

    /// Get resource type
    #[inline]
    pub fn resource_type_kind(&self) -> IpcResourceType {
        self.resource_type
    }

    /// Get process ID
    #[inline]
    pub fn pid(&self) -> Pid {
        self.pid
    }

    /// Manually release resource early
    pub fn release_early(mut self) -> GuardResult<()> {
        self.release()?;
        std::mem::forget(self);
        Ok(())
    }
}

impl Guard for IpcGuard {
    fn resource_type(&self) -> &'static str {
        self.resource_type.as_str()
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
        (self.cleanup)(self.resource_id).map_err(|e| GuardError::OperationFailed(e))?;

        self.emit_dropped();
        Ok(())
    }
}

impl GuardDrop for IpcGuard {
    fn on_drop(&mut self) {
        if self.active {
            if let Err(e) = self.release() {
                log::error!(
                    "IPC guard drop failed for PID {}, {} {}: {}",
                    self.pid,
                    self.resource_type.as_str(),
                    self.resource_id,
                    e
                );
                self.emit_error(&e);
            }
        }
    }
}

impl Observable for IpcGuard {
    fn emit_created(&self) {
        if let Some(ref collector) = self.collector {
            let event = Event::new(
                Severity::Debug,
                Category::Ipc,
                Payload::MetricUpdate {
                    name: "ipc_guard_created".to_string(),
                    value: 1.0,
                    labels: vec![
                        ("pid".to_string(), self.pid.to_string().into()),
                        (
                            "resource_type".to_string(),
                            self.resource_type.as_str().to_string(),
                        ),
                        ("resource_id".to_string(), self.resource_id.to_string().into()),
                    ],
                },
            )
            .with_pid(self.pid);
            collector.emit(event);
        }
    }

    fn emit_used(&self, operation: &str) {
        if let Some(ref collector) = self.collector {
            let event = Event::new(
                Severity::Debug,
                Category::Ipc,
                Payload::MetricUpdate {
                    name: "ipc_guard_used".to_string(),
                    value: 1.0,
                    labels: vec![
                        ("pid".to_string(), self.pid.to_string().into()),
                        (
                            "resource_type".to_string(),
                            self.resource_type.as_str().to_string(),
                        ),
                        ("resource_id".to_string(), self.resource_id.to_string().into()),
                        ("operation".to_string(), operation.to_string().into()),
                    ],
                },
            )
            .with_pid(self.pid);
            collector.emit(event);
        }
    }

    fn emit_dropped(&self) {
        if let Some(ref collector) = self.collector {
            let lifetime = self.metadata.lifetime_micros();
            let event = Event::new(
                Severity::Debug,
                Category::Ipc,
                Payload::MetricUpdate {
                    name: "ipc_guard_dropped".to_string(),
                    value: lifetime as f64,
                    labels: vec![
                        ("pid".to_string(), self.pid.to_string().into()),
                        (
                            "resource_type".to_string(),
                            self.resource_type.as_str().to_string(),
                        ),
                        ("resource_id".to_string(), self.resource_id.to_string().into()),
                        ("lifetime_micros".to_string(), lifetime.to_string().into()),
                    ],
                },
            )
            .with_pid(self.pid);
            collector.emit(event);
        }
    }

    fn emit_error(&self, error: &GuardError) {
        if let Some(ref collector) = self.collector {
            let event = Event::new(
                Severity::Error,
                Category::Ipc,
                Payload::MetricUpdate {
                    name: "ipc_guard_error".to_string(),
                    value: 1.0,
                    labels: vec![
                        ("pid".to_string(), self.pid.to_string().into()),
                        (
                            "resource_type".to_string(),
                            self.resource_type.as_str().to_string(),
                        ),
                        ("resource_id".to_string(), self.resource_id.to_string().into()),
                        ("error".to_string(), error.to_string().into()),
                    ],
                },
            )
            .with_pid(self.pid);
            collector.emit(event);
        }
    }
}

impl Drop for IpcGuard {
    fn drop(&mut self) {
        self.on_drop();
    }
}

/// Reference-counted IPC guard for shared ownership
pub struct IpcGuardRef {
    inner: Arc<std::sync::Mutex<IpcGuardState>>,
    metadata: GuardMetadata,
}

struct IpcGuardState {
    resource_id: u64,
    resource_type: IpcResourceType,
    pid: Pid,
    cleanup: Box<dyn Fn(u64) -> Result<(), String> + Send + Sync>,
    active: bool,
    collector: Option<Arc<Collector>>,
}

impl IpcGuardRef {
    /// Create a new reference-counted IPC guard
    pub fn new<F>(
        resource_id: u64,
        resource_type: IpcResourceType,
        pid: Pid,
        cleanup: F,
        collector: Option<Arc<Collector>>,
    ) -> Self
    where
        F: Fn(u64) -> Result<(), String> + Send + Sync + 'static,
    {
        let metadata = GuardMetadata::new(resource_type.as_str()).with_pid(pid);

        let state = IpcGuardState {
            resource_id,
            resource_type,
            pid,
            cleanup: Box::new(cleanup),
            active: true,
            collector,
        };

        Self {
            inner: Arc::new(std::sync::Mutex::new(state).into()),
            metadata,
        }
    }

    /// Get resource ID
    pub fn resource_id(&self) -> u64 {
        self.inner
            .lock()
            .expect("ipc guard lock poisoned - ipc state corrupted")
            .resource_id
    }

    /// Get resource type
    pub fn resource_type_kind(&self) -> IpcResourceType {
        self.inner
            .lock()
            .expect("ipc guard lock poisoned - ipc state corrupted")
            .resource_type
    }

    /// Get process ID
    pub fn pid(&self) -> Pid {
        self.inner
            .lock()
            .expect("ipc guard lock poisoned - ipc state corrupted")
            .pid
    }
}

impl Guard for IpcGuardRef {
    fn resource_type(&self) -> &'static str {
        self.metadata.resource_type
    }

    fn metadata(&self) -> &GuardMetadata {
        &self.metadata
    }

    fn is_active(&self) -> bool {
        self.inner
            .lock()
            .expect("ipc guard lock poisoned - ipc state corrupted")
            .active
    }

    fn release(&mut self) -> GuardResult<()> {
        let mut state = self
            .inner
            .lock()
            .expect("ipc guard lock poisoned - ipc state corrupted");
        if !state.active {
            return Err(GuardError::AlreadyReleased);
        }

        state.active = false;
        (state.cleanup)(state.resource_id).map_err(|e| GuardError::OperationFailed(e))?;

        Ok(())
    }
}

impl GuardRef for IpcGuardRef {
    fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl Clone for IpcGuardRef {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            metadata: self.metadata.clone(),
        }
    }
}

impl Drop for IpcGuardRef {
    fn drop(&mut self) {
        if self.is_last_ref() {
            let _ = self.release();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_ipc_guard_lifecycle() {
        let cleaned = Arc::new(AtomicBool::new(false));
        let cleaned_clone = cleaned.clone();

        let cleanup = move |_id: u64| {
            cleaned_clone.store(true, Ordering::SeqCst);
            Ok(())
        };

        {
            let guard = IpcGuard::new(42, IpcResourceType::Pipe, 1, cleanup, None);

            assert!(guard.is_active());
            assert_eq!(guard.resource_id(), 42);
            assert!(!cleaned.load(Ordering::SeqCst));
        }

        // Should be cleaned after drop
        assert!(cleaned.load(Ordering::SeqCst));
    }

    #[test]
    fn test_ipc_guard_ref() {
        let cleaned = Arc::new(AtomicBool::new(false));
        let cleaned_clone = cleaned.clone();

        let cleanup = move |_id: u64| {
            cleaned_clone.store(true, Ordering::SeqCst);
            Ok(())
        };

        let guard1 = IpcGuardRef::new(100, IpcResourceType::Queue, 1, cleanup, None);

        assert_eq!(guard1.ref_count(), 1);

        let guard2 = guard1.clone();
        assert_eq!(guard1.ref_count(), 2);

        drop(guard1);
        assert_eq!(guard2.ref_count(), 1);
        assert!(!cleaned.load(Ordering::SeqCst));

        drop(guard2);
        assert!(cleaned.load(Ordering::SeqCst));
    }
}
