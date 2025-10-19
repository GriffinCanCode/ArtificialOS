# Observability System Implementation

## System Integration

The observability system provides a unified architecture for monitoring and tracing:

```
Event Streaming Layer (Primary)
 Lock-free ring buffers
 Adaptive sampling
 Anomaly detection
 Query API

Distributed Tracing Layer (Complementary)
 Structured logging with tracing crate
 Request correlation across async boundaries
 Spans for performance profiling
```

## Core Components

### Collector

Central orchestrator for observability data:
- Event stream management
- Metrics collection
- Adaptive sampling
- Anomaly detection
- Causality tracking

### Event Stream

Lock-free MPMC ring buffer for event transport:
- Non-blocking event publishing
- Multiple subscribers
- Automatic ring buffer management

### Analysis

Real-time event processing:
- Statistical anomaly detection
- Query execution
- Adaptive sampling for overhead control

## Initialization

To initialize the observability system:

```rust
use ai_os_kernel::monitoring::{Collector, init_collector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let collector = Collector::new();
    init_collector(collector.clone());
    
    // Build subsystems with collector
    let process_manager = ProcessManager::builder()
        .with_collector(Arc::clone(&collector))
        .build();
    
    let scheduler = Scheduler::new(SchedulingPolicy::Fair)
        .with_collector(Arc::clone(&collector));
    
    Ok(())
}
```

## Integration Points

### Scheduler Integration

Scheduling decisions and context switches are observable:

```rust
// Scheduler emits events for scheduling decisions
// Tracks scheduling policy and quantum settings
```

### Memory Monitoring

Memory allocation and pressure tracking:

```rust
// Memory operations trigger events
collector.emit(Event::new(
    Severity::Info,
    Category::Memory,
    Payload::MemoryAllocated { size, pid },
));
```

### SyscallExecutor Integration

Syscall execution with performance tracking:

```rust
// Syscalls emit entry/exit events with duration
// Automatic anomaly detection for slow operations
```

## Configuration

### Environment Variables

```bash
# Distributed tracing
RUST_LOG=debug                   # Log level
KERNEL_TRACE_JSON=true           # JSON output

# Event streaming
KERNEL_SAMPLING_RATE=100         # Start at 100% (default)
KERNEL_STREAM_SIZE=65536         # Ring buffer size (default)
KERNEL_ANOMALY_THRESHOLD=3.0     # Z-score threshold (default)
```

### Runtime Tuning

```rust
// Get real-time stats
let stats = collector.stream_stats();
println!("Events: {} produced, {} consumed, {} dropped",
    stats.events_produced,
    stats.events_consumed,
    stats.events_dropped);
```

## Event Categories

The event system covers these categories:

1. **Process**: Process lifecycle events (creation, termination)
2. **Syscall**: Syscall entry/exit with duration tracking
3. **Memory**: Memory allocation/deallocation
4. **IPC**: Message send/receive operations
5. **Scheduler**: Scheduling decisions
6. **Performance**: Performance anomalies
7. **Security**: Permission denials, capability checks

## Usage Patterns

### Real-time Monitoring

```rust
let mut subscriber = collector.subscribe();

loop {
    while let Some(event) = subscriber.next() {
        if event.severity >= Severity::Warn {
            eprintln!("{}: {:?}", event.category, event.payload);
        }
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

### Causality Tracking

```rust
let causality_id = collector.emit_causal(Event::new(
    Severity::Info,
    Category::Process,
    Payload::ProcessCreated { name, priority },
));

// Emit related events in the same chain
collector.emit_in_chain(Event::new(
    Severity::Info,
    Category::Memory,
    Payload::MemoryAllocated { size: 1024, pid },
), causality_id);
```

## Performance Impact

### Memory Overhead

- Event stream: 65,536 slots with ~200 bytes each = ~13 MB
- Collector state: ~1 MB (metrics, sampling, detection)
- Total: ~14 MB per collector instance

### CPU Overhead

- Event emission: ~50-100ns per event (lock-free)
- With adaptive sampling: <1% at typical workloads
- Overhead auto-adjusts based on system load

## Known Limitations

1. **Event Ordering**: Events from different threads may be slightly out of order
   - Mitigation: Use causality_id for strict ordering requirements

2. **Backpressure**: When queue is full, events are dropped
   - Mitigation: Adaptive sampling reduces load automatically
   - Monitor: stream_stats().events_dropped

3. **Memory Bounded**: Fixed-size ring buffer (65,536 events)
   - Sufficient for most workloads at typical event rates
   - Configurable if needed

4. **No Persistence**: Events are in-memory only
   - External subscriber can write to disk if needed

## Query API

Basic query support for event analysis:

```rust
let mut subscriber = collector.subscribe();
let query = CommonQueries::syscall_performance();
let result = collector.query(query, &mut subscriber);
```

## Testing

Comprehensive test suite included:
- Unit tests for event emission
- Integration tests with subsystems
- Concurrent subscriber tests
- Causality tracking tests
- Anomaly detection tests

Run tests:
```bash
cd kernel
cargo test --test observability
```

## References

- Event System: `kernel/src/monitoring/events.rs`
- Collector API: `kernel/src/monitoring/collection/collector.rs`
- Query System: `kernel/src/monitoring/analysis/query.rs`
- Integration Tests: `kernel/tests/monitoring/`
- Public API: `kernel/src/monitoring/mod.rs`
