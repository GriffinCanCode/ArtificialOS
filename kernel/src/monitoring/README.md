# Observability System

Unified monitoring with dual-layer architecture for comprehensive system observability.

## Architecture

### Layer 1: Distributed Tracing
- Request correlation across async boundaries
- Performance profiling with structured context
- JSON/human-readable log output
- See: `tracing/`

### Layer 2: Event Streaming
- Lock-free event streams (ring buffers)
- Adaptive sampling (automatic overhead control)
- Built-in query API (no external tools needed)
- Anomaly detection (automatic outlier detection)
- Causality tracking (link related events)
- See: `events/`, `streaming/`, `collection/`

## Module Organization

```
monitoring/
├── mod.rs              # Public API and re-exports
├── README.md           # This file
│
├── events/             # Core event definitions
│   └── mod.rs          # Event types, severity, categories, payloads
│
├── streaming/          # Event transport layer
│   └── mod.rs          # Lock-free MPMC ring buffer, subscribers
│
├── collection/         # Central orchestration
│   ├── mod.rs          # Re-exports
│   ├── collector.rs    # Unified collector (main API)
│   └── bridge.rs       # Tracing ↔ event streaming integration
│
├── analysis/           # Event processing & analysis
│   ├── mod.rs          # Re-exports
│   ├── anomaly.rs      # Statistical anomaly detection
│   ├── query.rs        # Real-time event querying
│   └── sampler.rs      # Adaptive sampling for overhead control
│
├── metrics/            # Metrics collection
│   ├── mod.rs          # Re-exports
│   ├── collector.rs    # Legacy metrics (counters, gauges, histograms)
│   └── timeout.rs      # Timeout observability
│
└── tracing/            # Distributed tracing
    └── mod.rs          # Structured tracing with spans

```

## Usage

### Event Streaming

```rust
use ai_os_kernel::monitoring::{Collector, Category, Event, Payload, Severity};

// Create collector
let collector = Collector::new();

// Emit events
collector.process_created(123, "myapp".to_string(), 5);
collector.memory_pressure(85, 100);

// Subscribe and query
let mut sub = collector.subscribe();
let query = Query::new()
    .category(Category::Memory)
    .severity(Severity::Warn);
let results = collector.query(query, &mut sub);
```

### Distributed Tracing

```rust
use ai_os_kernel::monitoring::span_syscall;

// Create span (automatically tracked)
let span = span_syscall("read", 123);
span.record("bytes", 1024);
span.record_result(true);
// Span is automatically logged on drop with duration
```

### Causality Tracking

```rust
// Start causality chain
let causality_id = collector.emit_causal(Event::new(
    Severity::Info,
    Category::Process,
    Payload::ProcessCreated { name: "test".to_string(), priority: 5 }
));

// Emit related events in chain
collector.emit_in_chain(Event::new(
    Severity::Info,
    Category::Memory,
    Payload::MemoryAllocated { size: 1024, region_id: 1 }
), causality_id);

// Query causality chain
let chain = CausalityTracer::trace(&events, causality_id);
let timeline = CausalityTracer::timeline(&events, causality_id);
```

## Integration

The observability system integrates with:
- **Process Manager**: Process lifecycle events
- **Memory Manager**: Allocation/deallocation tracking
- **Scheduler**: Context switches, latency tracking
- **IPC**: Message passing metrics
- **Syscalls**: Execution timing, slow operations
- **Security**: Permission checks, violations
- **gRPC**: Request/response tracking

## Performance

- **Adaptive Sampling**: Maintains <2% CPU overhead automatically
- **Lock-Free Streams**: Zero-copy where possible
- **Cache-Line Alignment**: Hot structures aligned for performance
- **Bounded Memory**: Fixed-size ring buffers prevent unbounded growth

## Testing

Each module has comprehensive unit tests. Run with:

```bash
cargo test --lib monitoring
```

For integration tests:

```bash
cargo test --test observability
```

