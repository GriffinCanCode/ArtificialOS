/*!
 * File Descriptor Guards
 *
 * RAII guards for file descriptors with automatic cleanup
 */

use super::traits::{Guard, GuardDrop, Observable};
use super::{GuardError, GuardMetadata, GuardResult};
use crate::core::types::Pid;
use crate::monitoring::{Category, Collector, Event, Payload, Severity};
use std::sync::Arc;

/// File descriptor guard with automatic close
///
/// # Example
///
/// ```rust
/// let fd_guard = fd_manager.open_guard(pid, "/path/to/file", O_RDONLY)?;
/// let fd = fd_guard.fd();
/// // Use file descriptor
/// // Automatically closed on drop
/// ```
pub struct FdGuard {
    fd: u32,
    pid: Pid,
    path: Option<String>,
    close_fn: Box<dyn Fn(Pid, u32) -> Result<(), String> + Send + Sync>,
    metadata: GuardMetadata,
    active: bool,
    collector: Option<Arc<Collector>>,
}

impl FdGuard {
    /// Create a new file descriptor guard
    pub fn new<F>(
        fd: u32,
        pid: Pid,
        path: Option<String>,
        close_fn: F,
        collector: Option<Arc<Collector>>,
    ) -> Self
    where
        F: Fn(Pid, u32) -> Result<(), String> + Send + Sync + 'static,
    {
        let metadata = GuardMetadata::new("fd").with_pid(pid);

        let guard = Self {
            fd,
            pid,
            path,
            close_fn: Box::new(close_fn),
            metadata,
            active: true,
            collector,
        };

        guard.emit_created();
        guard
    }

    /// Get the file descriptor
    #[inline]
    pub fn fd(&self) -> u32 {
        self.fd
    }

    /// Get the process ID
    #[inline]
    pub fn pid(&self) -> Pid {
        self.pid
    }

    /// Get the file path if known
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Manually close the file descriptor early
    pub fn close_early(mut self) -> GuardResult<()> {
        self.release()?;
        std::mem::forget(self);
        Ok(())
    }
}

impl Guard for FdGuard {
    fn resource_type(&self) -> &'static str {
        "fd"
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
        (self.close_fn)(self.pid, self.fd).map_err(|e| GuardError::OperationFailed(e))?;

        self.emit_dropped();
        Ok(())
    }
}

impl GuardDrop for FdGuard {
    fn on_drop(&mut self) {
        if self.active {
            if let Err(e) = self.release() {
                log::error!("FD guard drop failed for PID {}, FD {}: {}", self.pid, self.fd, e);
                self.emit_error(&e);
            }
        }
    }
}

impl Observable for FdGuard {
    fn emit_created(&self) {
        if let Some(ref collector) = self.collector {
            let mut labels = vec![
                ("pid".to_string(), self.pid.to_string()),
                ("fd".to_string(), self.fd.to_string()),
            ];

            if let Some(ref path) = self.path {
                labels.push(("path".to_string(), path.clone()));
            }

            let event = Event::new(
                Severity::Debug,
                Category::Resource,
                Payload::MetricUpdate {
                    name: "fd_opened".to_string(),
                    value: 1.0,
                    labels,
                },
            ).with_pid(self.pid);
            collector.emit(event);
        }
    }

    fn emit_used(&self, operation: &str) {
        if let Some(ref collector) = self.collector {
            let event = Event::new(
                Severity::Debug,
                Category::Resource,
                Payload::MetricUpdate {
                    name: "fd_operation".to_string(),
                    value: 1.0,
                    labels: vec![
                        ("pid".to_string(), self.pid.to_string()),
                        ("fd".to_string(), self.fd.to_string()),
                        ("operation".to_string(), operation.to_string()),
                    ],
                },
            ).with_pid(self.pid);
            collector.emit(event);
        }
    }

    fn emit_dropped(&self) {
        if let Some(ref collector) = self.collector {
            let lifetime = self.metadata.lifetime_micros();
            let event = Event::new(
                Severity::Debug,
                Category::Resource,
                Payload::MetricUpdate {
                    name: "fd_closed".to_string(),
                    value: lifetime as f64,
                    labels: vec![
                        ("pid".to_string(), self.pid.to_string()),
                        ("fd".to_string(), self.fd.to_string()),
                        ("lifetime_micros".to_string(), lifetime.to_string()),
                    ],
                },
            ).with_pid(self.pid);
            collector.emit(event);
        }
    }

    fn emit_error(&self, error: &GuardError) {
        if let Some(ref collector) = self.collector {
            let event = Event::new(
                Severity::Error,
                Category::Resource,
                Payload::MetricUpdate {
                    name: "fd_error".to_string(),
                    value: 1.0,
                    labels: vec![
                        ("pid".to_string(), self.pid.to_string()),
                        ("fd".to_string(), self.fd.to_string()),
                        ("error".to_string(), error.to_string()),
                    ],
                },
            ).with_pid(self.pid);
            collector.emit(event);
        }
    }
}

impl Drop for FdGuard {
    fn drop(&mut self) {
        self.on_drop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_fd_guard_cleanup() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = closed.clone();

        let close_fn = move |_pid: Pid, _fd: u32| {
            closed_clone.store(true, Ordering::SeqCst);
            Ok(())
        };

        {
            let guard = FdGuard::new(3, 1, Some("/test/file".to_string()), close_fn, None);
            assert_eq!(guard.fd(), 3);
            assert_eq!(guard.path(), Some("/test/file"));
            assert!(!closed.load(Ordering::SeqCst));
        }

        // Should be closed after drop
        assert!(closed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_fd_guard_early_close() {
        let closed = Arc::new(AtomicBool::new(false));
        let closed_clone = closed.clone();

        let close_fn = move |_: Pid, _: u32| {
            closed_clone.store(true, Ordering::SeqCst);
            Ok(())
        };

        let guard = FdGuard::new(5, 1, None, close_fn, None);
        guard.close_early().unwrap();

        assert!(closed.load(Ordering::SeqCst));
    }
}
