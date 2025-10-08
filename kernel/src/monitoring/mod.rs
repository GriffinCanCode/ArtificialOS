/*!
 * Observability System
 * Unified monitoring with dual-layer architecture
 *
 * Architecture:
 *
 * Layer 1: Distributed Tracing (span_*, init_tracing)
 * - Request correlation across async boundaries
 * - Performance profiling with structured context
 * - JSON/human-readable log output
 *
 * Layer 2: Event Streaming (Collector)
 * - Lock-free event streams (ring buffers)
 * - Adaptive sampling (automatic overhead control)
 * - Built-in query API (no external tools needed)
 * - Anomaly detection (automatic outlier detection)
 * - Causality tracking (link related events)
 *
 * Integration: Tracing spans can emit events to Collector for aggregation
 *
 * Usage:
 * ```no_run
 * use ai_os_kernel::monitoring::{Collector, span_syscall};
 *
 * // Event streaming
 * let collector = Collector::new();
 * collector.process_created(123, "myapp".to_string(), 5);
 *
 * // Distributed tracing
 * let span = span_syscall("read", 123);
 * span.record("bytes", 1024);
 * ```
 */

// Domain modules
mod analysis;
mod collection;
mod events;
mod metrics;
mod streaming;
mod tracing;

// Primary Event Streaming API
pub use collection::Collector;
pub use events::{Category, Event, EventFilter, Payload, Severity, SyscallResult};
pub use streaming::{EventStream, StreamStats, Subscriber};

// Analysis API
pub use analysis::{
    Anomaly, AggregationType, CausalityTracer, CommonQueries, Detector, Query, QueryResult,
    SampleDecision, Sampler,
};

// Metrics API
pub use metrics::{MetricsCollector, MetricsSnapshot, TimeoutObserver, TimeoutStats};

// Distributed Tracing API (complementary to Collector)
pub use tracing::{
    current_span, generate_trace_id, init_tracing, span_grpc, span_operation, span_syscall,
    GrpcSpan, OperationSpan, SyscallSpan,
};

// Bridge for integrating tracing with event streaming
pub use collection::{
    global_collector, emit_from_span, emit_from_span_with_pid, init_collector,
};
