/*!
 * Syscall Guards
 *
 * Automatic tracing and timing for syscalls
 */

use super::{Guard, GuardMetadata, GuardResult};
use crate::core::types::Pid;
use crate::monitoring::{Category, Collector, Event, Payload, Severity};
use std::sync::Arc;
use std::time::Instant;

/// Syscall guard for automatic timing and tracing
///
/// Creates observability events automatically:
/// - syscall_enter on creation
/// - syscall_exit on drop
/// - Includes timing information
///
/// # Example
///
/// ```ignore
/// fn sys_read(fd: u32, buf: &mut [u8], pid: Pid, collector: Arc<Collector>) -> Result<usize> {
///     let _guard = SyscallGuard::new("read", pid, collector);
///     // Automatically traced and timed
///     do_read_impl(fd, buf, pid)
/// }
/// ```
pub struct SyscallGuard {
    syscall_name: &'static str,
    pid: Pid,
    start_time: Instant,
    collector: Arc<Collector>,
    metadata: GuardMetadata,
    result_recorded: bool,
}

impl SyscallGuard {
    /// Create a new syscall guard
    ///
    /// Emits syscall_enter event immediately
    pub fn new(syscall_name: &'static str, pid: Pid, collector: Arc<Collector>) -> Self {
        let start_time = Instant::now();

        // Emit entry event
        let event = Event::new(
            Severity::Debug,
            Category::Syscall,
            Payload::SyscallEnter {
                name: syscall_name.into(),
                args_hash: 0,
            },
        )
        .with_pid(pid);
        collector.emit(event);

        Self {
            syscall_name,
            pid,
            start_time,
            collector,
            metadata: GuardMetadata::new("syscall").with_pid(pid),
            result_recorded: false,
        }
    }

    /// Record syscall result before drop
    ///
    /// Call this to include result information in the exit event
    pub fn record_result<T>(&mut self, result: &Result<T, impl std::fmt::Display>) {
        self.result_recorded = true;

        let success = result.is_ok();
        let error_msg = result.as_ref().err().map(|e| e.to_string());

        let duration_micros = self.start_time.elapsed().as_micros() as u64;

        let mut payload = vec![
            ("syscall", self.syscall_name.to_string().into()),
            ("pid", self.pid.to_string().into()),
            ("duration_micros", duration_micros.to_string().into()),
            ("success", success.to_string().into()),
        ];

        if let Some(err) = error_msg {
            payload.push(("error", err));
        }

        let result_enum = if success {
            crate::monitoring::SyscallResult::Success
        } else {
            crate::monitoring::SyscallResult::Error
        };

        let severity = if success {
            Severity::Debug
        } else {
            Severity::Warn
        };

        let event = Event::new(
            severity,
            Category::Syscall,
            Payload::SyscallExit {
                name: self.syscall_name.into(),
                duration_us: duration_micros,
                result: result_enum,
            },
        )
        .with_pid(self.pid);

        self.collector.emit(event);
    }

    /// Get syscall name
    pub fn syscall_name(&self) -> &'static str {
        self.syscall_name
    }

    /// Get elapsed time since syscall started
    pub fn elapsed_micros(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }
}

impl Guard for SyscallGuard {
    fn resource_type(&self) -> &'static str {
        "syscall"
    }

    fn metadata(&self) -> &GuardMetadata {
        &self.metadata
    }

    fn is_active(&self) -> bool {
        true
    }

    fn release(&mut self) -> GuardResult<()> {
        Ok(())
    }
}

impl Drop for SyscallGuard {
    fn drop(&mut self) {
        // If result wasn't recorded, emit generic exit event
        if !self.result_recorded {
            let duration_micros = self.start_time.elapsed().as_micros() as u64;

            let event = Event::new(
                Severity::Debug,
                Category::Syscall,
                Payload::SyscallExit {
                    name: self.syscall_name.into(),
                    duration_us: duration_micros,
                    result: crate::monitoring::SyscallResult::Success,
                },
            )
            .with_pid(self.pid);

            self.collector.emit(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_guard_basic() {
        let collector = Arc::new(Collector::new());

        {
            let _guard = SyscallGuard::new("test_syscall", 1, collector.clone());
            // Simulates syscall execution
            std::thread::sleep(std::time::Duration::from_micros(100));
        }

        // Guard dropped, exit event emitted
    }

    #[test]
    fn test_syscall_guard_with_result() {
        let collector = Arc::new(Collector::new());

        {
            let mut guard = SyscallGuard::new("test_syscall", 1, collector.clone());

            let result: Result<i32, &str> = Ok(42);
            guard.record_result(&result);
        }
    }

    #[test]
    fn test_syscall_guard_with_error() {
        let collector = Arc::new(Collector::new());

        {
            let mut guard = SyscallGuard::new("test_syscall", 1, collector.clone());

            let result: Result<i32, &str> = Err("Permission denied");
            guard.record_result(&result);
        }
    }

    #[test]
    fn test_syscall_guard_timing() {
        let collector = Arc::new(Collector::new());
        let guard = SyscallGuard::new("test_syscall", 1, collector);

        std::thread::sleep(std::time::Duration::from_micros(100));

        assert!(guard.elapsed_micros() >= 100);
    }
}
