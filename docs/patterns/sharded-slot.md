# Sharded Slot Pattern

## Overview

A high-performance pattern for synchronization primitives using **fixed pre-allocated sharded slots** instead of dynamic per-key allocation.

## When This Pattern is SMART ✅

### Requirements (ALL must be met):

1. **Pure synchronization** - No unique per-key state
2. **Stable addresses needed** - e.g., `parking_lot_core` requires it
3. **Spurious wakeups acceptable** - Multiple keys can share slots
4. **High allocation cost** - Dynamic allocation is the bottleneck
5. **Lock-free fast path** - Just hash + atomic ops

### Perfect Use Cases:

- ✅ **Futex (`FutexWait`)** - IMPLEMENTED
  - Only needs parking address + waiter count
  - Spurious wakes are fine (futex design allows this)
  - Stable addresses for `parking_lot_core::park()`
  
- ✅ **Semaphores** (if implemented)
  - Just counter + wait mechanism
  - No per-semaphore unique state needed

- ✅ **Event counters**
  - Just atomic counter + parking
  - Spurious wakes on collision = harmless

## When This Pattern is WRONG ❌

### Anti-patterns (DO NOT use):

- ❌ **`CondvarWait`**
  - Needs unique Mutex per key
  - Sharing would cause incorrect blocking behavior
  - Different semantics than futex
  
- ❌ **`SignalManager`**
  - Per-process unique state (handlers, queues, masks)
  - Need precise cleanup on process exit
  - Can't share signal queues across PIDs
  
- ❌ **`PipeManager`**
  - Each pipe has unique buffer + reader/writer state
  - Need dynamic cleanup
  - Can't share buffers across pipes
  
- ❌ **Any manager with unique per-key data**
  - Process managers
  - File descriptor tables
  - Memory regions
  - IPC channels with buffers

## Implementation Details

### Core Structure

```rust
const SLOTS: usize = 512;  // Power of 2
const MASK: usize = SLOTS - 1;

#[repr(C, align(64))]  // Cache-line aligned
struct Slot {
    waiters: AtomicUsize,
}

pub struct ShardedWait<K> {
    slots: Box<[Slot; SLOTS]>,  // Fixed, stable addresses
    _phantom: PhantomData<K>,
}
```

### Key Properties

1. **Zero allocations after init** - All slots pre-allocated
2. **Stable addresses** - Array never moves, slots addressable
3. **Lock-free** - Just hash % SLOTS + atomic ops
4. **Cache-friendly** - 64-byte alignment prevents false sharing
5. **O(1) lookup** - Simple hash & bitwise AND

### Performance Benefits

- **Before**: 2-3 allocations per wait (DashMap entry + Arc + state)
- **After**: 0 allocations, just index into array
- **Futex tests**: Went from hanging forever  passing in <100ms
- **Memory**: Fixed ~32KB (512 slots  64 bytes) vs unbounded growth

## Design Philosophy

This follows **Linux futex design**:
- Hash table of wait queues (not per-address precision)
- Spurious wakes are acceptable (userspace rechecks condition)
- Minimal kernel overhead (no memory allocation in fast path)

## How to Decide

Ask these questions:

1. **Do I need unique state per key?**
   - YES  Use DashMap (e.g., managers, caches)
   - NO  Consider sharded slots

2. **Are spurious wakes acceptable?**
   - YES  Sharded slots OK (check condition after wake)
   - NO  Need precise per-key tracking

3. **Do I need stable memory addresses?**
   - YES  Sharded slots perfect
   - NO  DashMap might be simpler

4. **Is allocation the bottleneck?**
   - YES  Sharded slots will help
   - NO  Premature optimization

## Migration Checklist

If migrating from DashMap to sharded slots:

- [ ] Verify no unique per-key state needed
- [ ] Verify spurious wakes won't break logic
- [ ] Add condition recheck after wake
- [ ] Benchmark allocation overhead savings
- [ ] Test with high concurrency (stress test)
- [ ] Document spurious wake behavior

## Conclusion

**Use sharded slots for:**
- Pure wait/notify primitives
- Parking lot patterns
- Lock-free synchronization

**Keep DashMap for:**
- Managers with per-key state
- Caches with unique data
- Anything needing cleanup logic

The futex fix demonstrates this perfectly: changing from dynamic allocation (DashMap with Arc) to fixed sharded slots solved the hanging tests and improved performance by eliminating all allocation overhead in the hot path.
