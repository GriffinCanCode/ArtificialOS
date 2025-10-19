# Synchronization Primitives

## Overview

The kernel provides high-performance synchronization primitives optimized for different use cases. The system automatically selects the best strategy based on platform and workload characteristics.

## Architecture

### Core Components

```
kernel/src/core/sync/
 mod.rs       - Public API and exports
 traits.rs    - WaitStrategy trait abstraction
 wait.rs      - WaitQueue (main user-facing type)
 futex.rs     - Futex-based implementation (Linux)
 condvar.rs   - Condvar-based fallback (cross-platform)
 spinwait.rs  - Adaptive spinwait (low-latency)
 config.rs    - Strategy selection logic
```

### Strategy Selection

The system supports four strategies:

1. **Futex** (Linux) - Direct futex syscalls for minimal overhead
2. **Condvar** (Cross-platform) - Reliable parking_lot::Condvar
3. **SpinWait** (Low-latency) - Adaptive spinning before parking
4. **Auto** - Platform-aware automatic selection

## Performance Characteristics

### Futex Strategy

Best for long waits (> 100µs) on Linux

**Characteristics:**
- Direct kernel futex syscalls
- 1-2µs wake latency
- Zero CPU usage while waiting
- Platform: Linux only

**Use cases:**
- IPC wait operations
- Long-running syscall completions
- Cross-process synchronization

### Condvar Strategy

Best for cross-platform reliability

**Characteristics:**
- Uses parking_lot::Condvar (futex on Linux internally)
- 2-5µs wake latency
- Zero CPU usage while waiting
- Platform: All

**Use cases:**
- Default fallback
- Cross-platform code
- General-purpose waiting

### SpinWait Strategy

Best for very short waits (< 50µs)

**Characteristics:**
- Adaptive spinning before parking
- 0.1-1µs wake latency for short waits
- High CPU usage during spin phase
- Falls back to condvar for longer waits

**Use cases:**
- Lock-free data structure synchronization
- Ultra-low-latency syscalls
- Ring buffer completions

### Auto Strategy

**Behavior:**
- Linux: Selects Futex
- Other platforms: Selects Condvar

## API Usage

### Basic Usage

```rust
use ai_os_kernel::core::sync::WaitQueue;
use std::time::Duration;

// Create with automatic strategy selection
let queue = WaitQueue::<u64>::with_defaults();

// Wait for a specific key (sequence number)
queue.wait(42, Some(Duration::from_secs(1)))?;

// Wake waiters from another thread
queue.wake_one(42);
```

### Advanced Configuration

```rust
use ai_os_kernel::core::sync::{WaitQueue, SyncConfig, StrategyType};

// Low-latency configuration
let config = SyncConfig::low_latency();
let queue = WaitQueue::<u64>::new(config);

// Custom configuration
let config = SyncConfig {
    strategy: StrategyType::SpinWait,
    spin_duration: Duration::from_micros(100),
    max_spins: 1000,
};
let queue = WaitQueue::<u64>::new(config);
```

### Predicate-Based Waiting

```rust
use parking_lot::Mutex;
use std::sync::Arc;

let queue = WaitQueue::<u64>::with_defaults();
let value = Arc::new(Mutex::new(0));
let value_clone = value.clone();

// Wait until predicate is satisfied
queue.wait_while(100, Some(Duration::from_secs(1)), || {
    *value_clone.lock() < 10
})?;
```

## Integration Examples

### Ring Buffer Completion

```rust
pub struct CompletionRing {
    wait_queue: WaitQueue<u64>,
    // ...
}

impl CompletionRing {
    pub fn complete(&self, seq: u64, result: usize) {
        // Add to completion queue...
        
        // Wake waiters
        self.wait_queue.wake_one(seq);
    }

    pub fn wait_completion(&self, seq: u64) -> Result<Entry> {
        loop {
            // Check if already complete
            if let Some(entry) = self.try_get(seq) {
                return Ok(entry);
            }

            // Efficient wait (no polling)
            self.wait_queue.wait(seq, Some(Duration::from_secs(1)))?;
        }
    }
}
```

### IPC Synchronization

```rust
pub struct IpcChannel {
    wait_queue: WaitQueue<MessageId>,
    // ...
}

impl IpcChannel {
    pub fn send(&self, msg_id: MessageId, data: &[u8]) -> Result<()> {
        // Send message...
        
        // Wake receiver
        self.wait_queue.wake_one(msg_id);
        Ok(())
    }

    pub fn recv(&self, msg_id: MessageId, timeout: Duration) -> Result<Vec<u8>> {
        self.wait_queue.wait(msg_id, Some(timeout))?;
        // Retrieve message...
    }
}
```

## Performance Guidelines

### When to Use Each Strategy

| Wait Duration | Strategy | Reason |
|--------------|----------|--------|
| < 10µs | SpinWait | Spinning faster than syscall |
| 10-100µs | SpinWait | Adaptive spin avoids syscall |
| 100µs-1ms | Futex/Condvar | Syscall overhead acceptable |
| > 1ms | Futex/Condvar | Zero CPU usage critical |

### Optimization Tips

1. Use appropriate configuration:
   - `WaitQueue::low_latency()` for < 100µs waits
   - `WaitQueue::long_wait()` for > 1ms waits
   - `WaitQueue::with_defaults()` for general use

2. Key selection:
   - Use u64 for sequence numbers (efficient hashing)
   - Use tuple types for composite keys: `(Pid, SeqNum)`
   - Keep keys small (Copy types)

3. Wake strategies:
   - Use `wake_one()` for single waiter per key
   - Use `wake_all()` for broadcast patterns
   - Check `waiter_count()` for diagnostics

4. Predicate-based waiting:
   - Keep predicates fast (< 1µs)
   - Avoid locks in predicates if possible
   - Use for complex wait conditions

## Benchmarks

Run benchmarks to compare strategies:

```bash
cd kernel
cargo bench --bench sync_benchmark
```

Expected results (Linux, AMD Ryzen 9):
- Futex wake latency: 1-2µs
- Condvar wake latency: 2-5µs  
- SpinWait wake latency: 0.5-1µs (for < 10µs waits)
- Multi-waiter throughput: 500K ops/sec

## Testing

Run comprehensive tests:

```bash
cd kernel
cargo test --test sync_primitives
```

Tests cover:
- Single and multiple waiters
- Timeout behavior
- Predicate-based waiting
- Concurrent operations
- Strategy selection
- Edge cases

## Migration Guide

### From Busy-Wait Polling

Before:
```rust
loop {
    if let Some(entry) = queue.try_pop() {
        return Ok(entry);
    }
    std::thread::yield_now(); // Inefficient
}
```

After:
```rust
let wait_queue = WaitQueue::<u64>::with_defaults();

loop {
    if let Some(entry) = queue.try_pop() {
        return Ok(entry);
    }
    wait_queue.wait(seq, Some(timeout))?; // Efficient
}

// On completion:
wait_queue.wake_one(seq);
```

### From tokio::sync::Notify

Before:
```rust
use tokio::sync::Notify;

let notify = Arc::new(Notify::new());
notify.notified().await; // Async only
```

After:
```rust
use ai_os_kernel::core::sync::WaitQueue;

let queue = WaitQueue::<u64>::with_defaults();
queue.wait(key, timeout)?; // Works in sync contexts
```

## Implementation Details

### Futex Strategy

- Uses `parking_lot_core::park/unpark` for futex operations
- Converts keys to parking addresses via hash
- Maintains waiter counts for diagnostics
- Cache-line aligned for performance

### Condvar Strategy

- One condvar per key (lazy allocation)
- Tracks waiter count per key
- Auto-cleanup when last waiter leaves
- Timeout support via `wait_for`

### SpinWait Strategy

- Adaptive spinning with backoff
- Falls back to condvar after spin phase
- Configurable spin duration and count
- Optimized for low-latency scenarios

## Future Enhancements

- Cross-process futex support
- Priority-based wakeup
- Wait-free fast path for uncontended case
- Integration with io_uring IORING_OP_POLL_ADD
- Kernel-level wait queues (when running as actual kernel)

## References

- [Linux futex man page](https://man7.org/linux/man-pages/man2/futex.2.html)
- [parking_lot documentation](https://docs.rs/parking_lot/)
- [io_uring wait mechanisms](https://kernel.dk/io_uring.pdf)
- [Lock-Free Programming](https://preshing.com/20120612/an-introduction-to-lock-free-programming/)
