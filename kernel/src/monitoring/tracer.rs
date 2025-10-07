/*!
 * Distributed Tracing
 * Structured tracing for syscalls and operations using the tracing crate
 *
 * Features:
 * - Automatic trace ID generation for request correlation
 * - JSON-formatted logs for structured parsing
 * - Span hierarchies for complex operations
 * - Context propagation across async boundaries
 * - Performance metrics embedded in traces
 */

use tracing::{debug, error, info, warn, span, Level, Span};
use tracing_subscriber::{
    fmt::format::FmtSpan,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use std::time::Instant;
use uuid::Uuid;

/// Initialize structured tracing with enhanced features
///
/// Environment variables:
/// - RUST_LOG: Set log level (default: info)
/// - KERNEL_TRACE_JSON: Enable JSON output (default: false)
pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Check if JSON output is requested
    let use_json = std::env::var("KERNEL_TRACE_JSON")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    let registry = tracing_subscriber::registry().with(env_filter);

    if use_json {
        // JSON output for production/parsing
        registry
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_line_number(true)
                    .with_file(true)
                    .with_current_span(true)
                    .with_span_list(true)
                    .with_span_events(FmtSpan::FULL)
            )
            .init();
        info!("Structured tracing initialized with JSON output and full span events");
    } else {
        // Human-readable output for development
        registry
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_line_number(true)
                    .with_file(true)
                    .with_span_events(FmtSpan::CLOSE)
                    .compact()
            )
            .init();
        info!("Structured tracing initialized with context propagation");
    }
}

/// Generate a unique trace ID for request correlation
pub fn generate_trace_id() -> String {
    Uuid::new_v4().to_string()
}

/// Span for syscall tracing with rich structured fields
pub struct SyscallSpan {
    _span: tracing::Span,
    start: Instant,
    syscall_name: String,
    trace_id: String,
}

impl SyscallSpan {
    pub fn new(syscall_name: &str, pid: u32) -> Self {
        let trace_id = generate_trace_id();

        // Create a tracing span with extensive structured fields
        let span = span!(
            Level::DEBUG,
            "syscall",
            trace_id = %trace_id,
            syscall = syscall_name,
            pid = pid,
            duration_us = tracing::field::Empty,
            duration_ms = tracing::field::Empty,
            result = tracing::field::Empty,
            error = tracing::field::Empty,
            args_count = tracing::field::Empty,
            return_value = tracing::field::Empty,
        );

        let _entered = span.enter();
        debug!(
            syscall = syscall_name,
            pid = pid,
            "syscall started"
        );
        drop(_entered);

        Self {
            _span: span,
            start: Instant::now(),
            syscall_name: syscall_name.to_string(),
            trace_id,
        }
    }

    /// Get the trace ID for this syscall
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Record additional structured fields during syscall execution
    pub fn record<V: std::fmt::Debug>(&self, key: &str, value: V) {
        self._span.record(key, &format!("{:?}", value));
    }

    /// Record the syscall result
    pub fn record_result(&self, success: bool) {
        self._span.record("result", if success { "success" } else { "error" });
    }

    /// Record an error
    pub fn record_error(&self, error: &str) {
        self._span.record("error", error);
        self._span.record("result", "error");
    }

    /// Record the return value
    pub fn record_return<V: std::fmt::Debug>(&self, value: V) {
        self._span.record("return_value", &format!("{:?}", value));
    }

    /// Record the number of arguments
    pub fn record_args_count(&self, count: usize) {
        self._span.record("args_count", count);
    }

    /// Record a field with any Debug-compatible type
    pub fn record_debug<V: std::fmt::Debug>(&self, key: &str, value: V) {
        self._span.record(key, &format!("{:?}", value));
    }

    /// Enter the span context (useful for async operations)
    pub fn enter(&self) -> tracing::span::Entered<'_> {
        self._span.enter()
    }
}

impl Drop for SyscallSpan {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let _entered = self._span.enter();

        if duration.as_millis() > 10 {
            // Slow syscall warning with structured fields
            self._span.record("duration_ms", duration.as_millis());
            warn!(
                trace_id = %self.trace_id,
                syscall = %self.syscall_name,
                duration_ms = duration.as_millis(),
                slow = true,
                "slow syscall detected"
            );
        } else {
            self._span.record("duration_us", duration.as_micros());
            debug!(
                trace_id = %self.trace_id,
                syscall = %self.syscall_name,
                duration_us = duration.as_micros(),
                "syscall completed"
            );
        }
    }
}

/// Span for operation tracing with structured fields and context propagation
pub struct OperationSpan {
    _span: tracing::Span,
    start: Instant,
    trace_id: String,
}

impl OperationSpan {
    pub fn new(operation: &str) -> Self {
        let trace_id = generate_trace_id();

        // Create a tracing span for the operation with rich fields
        let span = span!(
            Level::DEBUG,
            "operation",
            trace_id = %trace_id,
            operation = operation,
            duration_us = tracing::field::Empty,
            duration_ms = tracing::field::Empty,
            result = tracing::field::Empty,
            items_processed = tracing::field::Empty,
            error = tracing::field::Empty,
        );

        let _entered = span.enter();
        debug!(
            operation = operation,
            trace_id = %trace_id,
            "operation started"
        );
        drop(_entered);

        Self {
            _span: span,
            start: Instant::now(),
            trace_id,
        }
    }

    /// Get the trace ID for this operation
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Record structured fields during operation execution
    pub fn record(&self, key: &str, value: &str) {
        self._span.record(key, value);
    }

    /// Record a field with any Debug-compatible type
    pub fn record_debug<V: std::fmt::Debug>(&self, key: &str, value: V) {
        self._span.record(key, &format!("{:?}", value));
    }

    /// Record the operation result
    pub fn record_result(&self, success: bool) {
        self._span.record("result", if success { "success" } else { "error" });
    }

    /// Record an error
    pub fn record_error(&self, error: &str) {
        self._span.record("error", error);
        self._span.record("result", "error");
    }

    /// Record items processed count
    pub fn record_items_processed(&self, count: usize) {
        self._span.record("items_processed", count);
    }

    /// Enter the span context (useful for async operations)
    pub fn enter(&self) -> tracing::span::Entered<'_> {
        self._span.enter()
    }
}

impl Drop for OperationSpan {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let _entered = self._span.enter();

        if duration.as_millis() > 100 {
            self._span.record("duration_ms", duration.as_millis());
            warn!(
                trace_id = %self.trace_id,
                duration_ms = duration.as_millis(),
                slow = true,
                "slow operation detected"
            );
        } else {
            self._span.record("duration_us", duration.as_micros());
            debug!(
                trace_id = %self.trace_id,
                duration_us = duration.as_micros(),
                "operation completed"
            );
        }
    }
}

/// Span for gRPC request tracing with full request context
pub struct GrpcSpan {
    _span: tracing::Span,
    start: Instant,
    trace_id: String,
}

impl GrpcSpan {
    pub fn new(method: &str) -> Self {
        let trace_id = generate_trace_id();

        let span = span!(
            Level::INFO,
            "grpc_request",
            trace_id = %trace_id,
            method = method,
            duration_us = tracing::field::Empty,
            duration_ms = tracing::field::Empty,
            status = tracing::field::Empty,
            error = tracing::field::Empty,
            request_size = tracing::field::Empty,
            response_size = tracing::field::Empty,
        );

        let _entered = span.enter();
        info!(
            method = method,
            trace_id = %trace_id,
            "gRPC request started"
        );
        drop(_entered);

        Self {
            _span: span,
            start: Instant::now(),
            trace_id,
        }
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn record_status(&self, status: &str) {
        self._span.record("status", status);
    }

    pub fn record_error(&self, error: &str) {
        self._span.record("error", error);
        self._span.record("status", "error");
    }

    pub fn record_request_size(&self, size: usize) {
        self._span.record("request_size", size);
    }

    pub fn record_response_size(&self, size: usize) {
        self._span.record("response_size", size);
    }

    pub fn enter(&self) -> tracing::span::Entered<'_> {
        self._span.enter()
    }
}

impl Drop for GrpcSpan {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let _entered = self._span.enter();

        if duration.as_millis() > 50 {
            self._span.record("duration_ms", duration.as_millis());
            warn!(
                trace_id = %self.trace_id,
                duration_ms = duration.as_millis(),
                slow = true,
                "slow gRPC request"
            );
        } else {
            self._span.record("duration_us", duration.as_micros());
            info!(
                trace_id = %self.trace_id,
                duration_us = duration.as_micros(),
                "gRPC request completed"
            );
        }
    }
}

/// Helper to create syscall span with automatic context propagation
#[inline]
pub fn span_syscall(name: &str, pid: u32) -> SyscallSpan {
    SyscallSpan::new(name, pid)
}

/// Helper to create operation span with automatic context propagation
#[inline]
pub fn span_operation(name: &str) -> OperationSpan {
    OperationSpan::new(name)
}

/// Helper to create gRPC span with automatic context propagation
#[inline]
pub fn span_grpc(method: &str) -> GrpcSpan {
    GrpcSpan::new(method)
}

/// Get the current span for manual tracing
pub fn current_span() -> Span {
    Span::current()
}

/// Create an info-level event with structured fields
#[macro_export]
macro_rules! trace_info {
    ($($key:tt = $value:expr),+ $(,)?) => {
        tracing::info!($($key = $value),+);
    };
}

/// Create a debug-level event with structured fields
#[macro_export]
macro_rules! trace_debug {
    ($($key:tt = $value:expr),+ $(,)?) => {
        tracing::debug!($($key = $value),+);
    };
}

/// Create a warn-level event with structured fields
#[macro_export]
macro_rules! trace_warn {
    ($($key:tt = $value:expr),+ $(,)?) => {
        tracing::warn!($($key = $value),+);
    };
}

/// Create an error-level event with structured fields
#[macro_export]
macro_rules! trace_error {
    ($($key:tt = $value:expr),+ $(,)?) => {
        tracing::error!($($key = $value),+);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    fn init_test_tracing() {
        let _ = tracing_subscriber::registry()
            .with(EnvFilter::new("debug"))
            .with(tracing_subscriber::fmt::layer().compact())
            .try_init();
    }

    #[test]
    fn test_syscall_span() {
        init_test_tracing();

        let span = span_syscall("test_syscall", 123);
        span.record("arg1", "test_value");
        std::thread::sleep(std::time::Duration::from_micros(100));
        // Span will be dropped and logged with structured fields
    }

    #[test]
    fn test_operation_span() {
        init_test_tracing();

        let span = span_operation("test_op");
        span.record("key", "value");
        span.record_debug("count", 42);
        std::thread::sleep(std::time::Duration::from_micros(100));
        // Span will be dropped and logged with structured fields
    }

    #[test]
    fn test_span_context_propagation() {
        init_test_tracing();

        let parent_span = span_operation("parent_operation");
        let _guard = parent_span.enter();

        // This span will be a child of parent_operation due to context propagation
        let child_span = span_syscall("child_syscall", 456);
        child_span.record("nested", true);

        drop(child_span);
        drop(_guard);
        // Both spans will show hierarchy in the logs
    }

    #[test]
    fn test_slow_syscall_detection() {
        init_test_tracing();

        let span = span_syscall("slow_syscall", 789);
        // Sleep for more than 10ms to trigger slow syscall warning
        std::thread::sleep(std::time::Duration::from_millis(15));
        drop(span);
        // Should log a warning for slow syscall
    }
}
