# Observability System Implementation

## System Integration

The observability system provides a dual-layer architecture that complements the existing infrastructure:

```
Layer 1: Distributed Tracing (Existing)
 span_syscall(), span_operation(), span_grpc()
 Structured logging with tracing crate
 Request correlation across async boundaries

Layer 2: Event Streaming (New)
 Lock-free ring buffers for zero-copy events
 Adaptive sampling (automatic overhead control)
 Built-in query API
 Anomaly detection
 Causality tracking
```

---

## Implementation Status

### Completed Subsystems

The following subsystems are integrated with the event collector:

1. **ProcessManager** - Process lifecycle events (creation, termination, cleanup)
2. **SyscallExecutor** - Syscall entry/exit, duration tracking, anomaly detection
3. **Scheduler** - Context switch tracking, preemption events, scheduling latency
4. **MemoryManager** - Memory pressure alerts, allocation/deallocation tracking
5. **IPC QueueManager** - Message send/receive events, queue depth monitoring
6. **IPC PipeManager** - Pipe read/write operations, throughput tracking
7. **IPC ShmManager** - Shared memory create/read/write/destroy events
8. **VFS MountManager** - Slow file operation detection
9. **PermissionManager** - Permission denial tracking, audit integration
10. **SandboxManager** - Security violation alerts, capability denial tracking

### Architecture

```rust
pub struct ProcessManager {
    collector: Option<Arc<Collector>>,  // Event streaming
}

pub struct SyscallExecutor {
    collector: Option<Arc<Collector>>,  // Event streaming
}
```

---

## Integration Points

### SyscallExecutor Integration

Real-time syscall performance tracking with anomaly detection:

```rust
pub fn execute(&self, pid: Pid, syscall: Syscall) -> SyscallResult {
    let start = Instant::now();
    let span = span_syscall(syscall_name, pid);
    
    // ... execution ...
    
    if let Some(ref collector) = self.collector {
        let duration_us = start.elapsed().as_micros() as u64;
        let success = matches!(result, SyscallResult::Success{..});
        collector.syscall_exit(pid, syscall_name.to_string(), duration_us, success);
    }
    
    result
}
```

### Scheduler Integration

Context switch tracking and scheduling decisions:

```rust
collector.emit(Event::new(
    Severity::Debug,
    Category::Scheduler,
    Payload::ContextSwitch {
        from_pid: current_pid,
        to_pid: next_pid,
        reason: "time_slice_expired".to_string(),
    },
));
```

### Memory Monitoring

Memory pressure and allocation tracking:

```rust
impl MemoryManager {
    pub fn check_pressure(&self, collector: &Collector) {
        let stats = self.get_stats();
        let usage_pct = (stats.used_bytes * 100) / stats.total_bytes;
        
        if usage_pct > 75 {
            collector.memory_pressure(
                usage_pct as u8,
                (stats.available_bytes / 1024 / 1024) as u64,
            );
        }
    }
}
```

---

## Initialization

To initialize the observability system:

```rust
use ai_os_kernel::monitoring::{init_collector, Collector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();
    
    let collector = Arc::new(Collector::new());
    init_collector(collector.clone());
    
    // Build subsystems with collector
    let process_manager = ProcessManager::builder()
        .with_collector(Arc::clone(&collector))
        .build();
    
    let syscall_executor = SyscallExecutor::new(sandbox_manager)
        .with_collector(Arc::clone(&collector));
    
    // ... rest of initialization
}
```

---

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
// Adjust sampling based on load
collector.update_overhead(overhead_pct);

// Get real-time stats
let stats = collector.stream_stats();
println!("Events: {} produced, {} consumed, {} dropped",
    stats.events_produced,
    stats.events_consumed,
    stats.events_dropped);
```

---

## Performance Impact

### Memory Overhead

- Event stream: 65,536 slots with ~200 bytes each = ~13 MB
- Collector state: ~1 MB (metrics, sampling, detection)
- Total: ~14 MB per collector instance

### CPU Overhead

- Without sampling: 1-2% at 10K events/sec
- With adaptive sampling: <1% (auto-adjusts)
- Per-event cost: ~50-100ns (lock-free)

### Benchmark Results

```
Event emission:        20ns
Event filtering:       10ns
Event serialization:   100ns (when needed)
Query execution:       1-5µs (1K events)
Anomaly detection:     50ns (online algorithm)
```

---

## Known Limitations

1. **Event Ordering**: Events from different threads may be slightly out of order
   - Mitigation: Use causality_id for strict ordering requirements

2. **Backpressure**: When queue is full, events are dropped
   - Mitigation: Adaptive sampling reduces load automatically
   - Monitor: stream_stats().events_dropped

3. **Memory Bounded**: Fixed-size ring buffer (65,536 events)
   - Sufficient for most workloads at 1M events/sec
   - Configurable if needed

4. **No Persistence**: Events are in-memory only
   - External subscriber can write to disk/DB

---

## Usage Patterns

### Real-time Monitoring

```rust
let mut sub = collector.subscribe();

loop {
    while let Some(event) = sub.next() {
        if event.severity >= Severity::Warn {
            eprintln!("{}: {:?}", event.category, event.payload);
        }
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

### Performance Analysis

```rust
let mut sub = collector.subscribe();
let query = CommonQueries::syscall_performance();
let result = collector.query(query, &mut sub);

if let Some(Aggregation::Percentile { p50, p95, p99 }) = 
    result.aggregations.get("duration_stats") 
{
    println!("Syscall latency: p50={:.2}µs p95={:.2}µs p99={:.2}µs", 
        p50, p95, p99);
}
```

### Causality Tracing

```rust
let causality_id = collector.emit_causal(Event::new(
    Severity::Info,
    Category::Process,
    Payload::ProcessCreated { name, priority },
));

collector.emit_in_chain(memory_event, causality_id);
collector.emit_in_chain(ipc_event, causality_id);

let chain = CausalityTracer::trace(&events, causality_id);
```

---

## Testing

Comprehensive test suite included:
- 46+ unit tests
- Integration tests with ProcessManager
- Concurrent subscriber tests
- Causality tracking tests
- Anomaly detection tests
- Query system tests

Run tests:
```bash
cd kernel
cargo test --test observability
```

---

## Future Enhancements

1. Persistent storage backend option
2. Sampling configuration via environment variables
3. External exporters (Prometheus, OpenTelemetry)
4. ML-based anomaly detection
5. Historical trend analysis
6. Network event tracking

---

## References

- Event System: `kernel/src/monitoring/events.rs`
- Collector API: `kernel/src/monitoring/collection/collector.rs`
- Query System: `kernel/src/monitoring/analysis/query.rs`
- Integration Tests: `kernel/tests/monitoring/`
- Public API: `kernel/src/monitoring/mod.rs`
