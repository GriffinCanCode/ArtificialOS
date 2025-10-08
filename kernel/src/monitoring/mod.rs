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

// Event streaming modules
mod anomaly;
mod bridge;
mod collector;
mod events;
mod query;
mod sampler;
mod stream;
mod timeout;

// Distributed tracing modules
mod metrics;
mod tracer;

// Primary Event Streaming API
pub use collector::Collector;
pub use events::{Category, Event, EventFilter, Payload, Severity, SyscallResult};
pub use query::{AggregationType, CausalityTracer, CommonQueries, Query, QueryResult};
pub use stream::{EventStream, StreamStats, Subscriber};

// Anomaly detection
pub use anomaly::{Anomaly, Detector};

// Sampling
pub use sampler::{SampleDecision, Sampler};

// Timeout observability
pub use timeout::{TimeoutObserver, TimeoutStats};

// Distributed Tracing API (complementary to Collector)
pub use tracer::{
    current_span, generate_trace_id, init_tracing, span_grpc, span_operation, span_syscall,
    GrpcSpan, OperationSpan, SyscallSpan,
};

// Metrics API (used by gRPC endpoints)
pub use metrics::{MetricsCollector, MetricsSnapshot};

// Bridge for integrating tracing with event streaming
pub use bridge::{
    collector as global_collector, emit_from_span, emit_from_span_with_pid, init_collector,
};
