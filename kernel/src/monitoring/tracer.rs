/*!
 * Distributed Tracing
 * Structured tracing for syscalls and operations
 */

use log::{debug, warn};
use std::time::Instant;

/// Initialize tracing (using log for simplicity)
pub fn init_tracing() {
    // Already initialized in main.rs with env_logger
    debug!("Tracing initialized");
}

/// Span for syscall tracing
pub struct SyscallSpan {
    name: String,
    start: Instant,
}

impl SyscallSpan {
    pub fn new(syscall_name: &str, pid: u32) -> Self {
        debug!("syscall_start: {} pid={}", syscall_name, pid);
        Self {
            name: syscall_name.to_string(),
            start: Instant::now(),
        }
    }
}

impl Drop for SyscallSpan {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        if duration.as_millis() > 10 {
            warn!(
                "syscall_slow: {} duration_ms={}",
                self.name,
                duration.as_millis()
            );
        } else {
            debug!(
                "syscall_end: {} duration_us={}",
                self.name,
                duration.as_micros()
            );
        }
    }
}

/// Span for operation tracing
pub struct OperationSpan {
    name: String,
    start: Instant,
}

impl OperationSpan {
    pub fn new(operation: &str) -> Self {
        debug!("operation_start: {}", operation);
        Self {
            name: operation.to_string(),
            start: Instant::now(),
        }
    }

    pub fn record(&self, key: &str, value: &str) {
        debug!("{}: {}={}", self.name, key, value);
    }
}

impl Drop for OperationSpan {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        debug!(
            "operation_end: {} duration_us={}",
            self.name,
            duration.as_micros()
        );
    }
}

/// Helper to create syscall span
pub fn span_syscall(name: &str, pid: u32) -> SyscallSpan {
    SyscallSpan::new(name, pid)
}

/// Helper to create operation span
pub fn span_operation(name: &str) -> OperationSpan {
    OperationSpan::new(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_span() {
        let _span = span_syscall("test_syscall", 123);
        std::thread::sleep(std::time::Duration::from_micros(100));
        // Span will be dropped and logged
    }

    #[test]
    fn test_operation_span() {
        let span = span_operation("test_op");
        span.record("key", "value");
        std::thread::sleep(std::time::Duration::from_micros(100));
        // Span will be dropped and logged
    }
}
