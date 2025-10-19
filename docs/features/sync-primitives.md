# Synchronization Primitives

## Overview

The kernel provides high-performance synchronization primitives optimized for different use cases. The system automatically selects the best strategy based on platform and workload characteristics.

## Architecture

### Core Components

```
kernel/src/core/sync/
 mod.rs              - Public API and exports
 wait/               - Wait/notify primitives (main abstraction)
   wait.rs           - WaitQueue implementation
   futex.rs          - Linux futex-based implementation
   condvar.rs        - Parking lot condvar fallback
   spinwait.rs       - Adaptive spinwait for low-latency
   config.rs         - Strategy selection logic
   traits.rs         - WaitStrategy trait
 lockfree/           - Lock-free data structures (RCU, seqlock, flat combining)
 locks/              - Advanced lock primitives (adaptive locks, striped maps)
 management/         - Configuration management (ShardManager, WorkloadProfile)
```

### Strategy Selection

The system supports three core strategies selected based on platform:

1. **Futex** (Linux) - Direct futex syscalls with minimal overhead
2. **Condvar** (Cross-platform) - Parking lot condvar for reliable blocking
3. **SpinWait** (Low-latency) - Adaptive spinning before fallback to condvar

Platform-aware automatic selection:
- Linux: Futex strategy
- Other platforms: Condvar strategy

## Performance Characteristics

### Futex Strategy

**When used**: Long waits (greater than 100 microseconds) on Linux

**Performance**:
- Wake latency: 1-2 microseconds
- CPU usage: Zero while waiting
- Platform: Linux only

**Use cases**:
- IPC wait operations
- Long-running syscall completions
- Cross-process synchronization

### Condvar Strategy

**When used**: Cross-platform code, general-purpose waiting

**Performance**:
- Wake latency: 2-5 microseconds
- CPU usage: Zero while waiting
- Platform: All

**Use cases**:
- Default fallback across platforms
- General-purpose blocking operations

### SpinWait Strategy

**When used**: Very short waits (less than 50 microseconds)

**Performance**:
- Wake latency: 0.5-1 microsecond for short waits
- CPU usage: High during spin phase, then falls back to condvar
- Adaptive: Falls back to condvar for longer waits

**Use cases**:
- Lock-free data structure synchronization
- Ultra-low-latency syscall completions

## API Usage

### Basic Usage

```rust
use ai_os_kernel::core::sync::WaitQueue;
use std::time::Duration;

// Create with automatic strategy selection
let queue = WaitQueue::<u64>::with_defaults();

// Wait for a specific key (sequence number)
queue.wait(42, Some(Duration::from_secs(1)))?;

// Wake single waiter
queue.wake_one(42);

// Wake all waiters on a key
queue.wake_all(42);
```

### Configuration

```rust
use ai_os_kernel::core::sync::{WaitQueue, SyncConfig, StrategyType};
use std::time::Duration;

// Platform-aware automatic selection
let queue = WaitQueue::<u64>::with_defaults();

// Custom configuration
let config = SyncConfig {
    strategy: StrategyType::SpinWait,
    spin_duration: Duration::from_micros(100),
    max_spins: 1000,
};
let queue = WaitQueue::<u64>::new(config);
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
        
        // Wake waiters for this sequence
        self.wait_queue.wake_one(seq);
    }

    pub fn wait_completion(&self, seq: u64, timeout: Duration) -> Result<Entry> {
        loop {
            // Check if already complete
            if let Some(entry) = self.try_get(seq) {
                return Ok(entry);
            }

            // Efficient wait
            self.wait_queue.wait(seq, Some(timeout))?;
        }
    }
}
```

### IPC Message Synchronization

```rust
pub struct IpcChannel {
    wait_queue: WaitQueue<MessageId>,
    // ...
}

impl IpcChannel {
    pub fn send(&self, msg_id: MessageId, data: &[u8]) -> Result<()> {
        // Send message...
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

### Wait Duration Selection

| Duration | Strategy | Rationale |
|----------|----------|-----------|
| < 10 microseconds | SpinWait | Spinning faster than syscall overhead |
| 10-100 microseconds | SpinWait | Adaptive spinning avoids syscall |
| 100 microseconds-1 millisecond | Futex/Condvar | Syscall overhead acceptable |
| > 1 millisecond | Futex/Condvar | Zero CPU usage preferred |

### Configuration Recommendations

- Use `WaitQueue::with_defaults()` for general purposes
- Use SpinWait strategy for less than 100 microseconds expected waits
- Use Futex strategy on Linux for maximum efficiency
- Key types should be Copy and hashable (u64, tuples)

## Testing

Run comprehensive tests:

```bash
cd kernel
cargo test --lib sync
cargo test --test sync_primitives
```

Tests cover:
- Single and multiple waiters
- Timeout behavior
- Concurrent operations
- Strategy selection
- Edge cases

## Migration from Polling

Before (inefficient):
```rust
loop {
    if let Some(entry) = queue.try_pop() {
        return Ok(entry);
    }
    std::thread::yield_now();
}
```

After (efficient):
```rust
let wait_queue = WaitQueue::<u64>::with_defaults();

loop {
    if let Some(entry) = queue.try_pop() {
        return Ok(entry);
    }
    wait_queue.wait(seq, Some(timeout))?;
}

// On completion:
wait_queue.wake_one(seq);
```

## Implementation Details

### Futex Strategy

- Direct kernel futex syscalls via parking_lot_core
- Converts keys to parking addresses via hash
- Cache-line aligned for performance
- Zero-cost abstraction via monomorphization

### Condvar Strategy

- One condvar per key (lazy allocation)
- Auto-cleanup when last waiter leaves
- Timeout support via wait_for
- Cross-platform compatible

### SpinWait Strategy

- Adaptive spinning with exponential backoff
- Falls back to condvar after spin phase
- Configurable spin duration and iteration count
- Optimized for low-latency scenarios

## References

- Linux futex man page: https://man7.org/linux/man-pages/man2/futex.2.html
- parking_lot documentation: https://docs.rs/parking_lot/
- Lock-free programming: https://preshing.com/20120612/an-introduction-to-lock-free-programming/
