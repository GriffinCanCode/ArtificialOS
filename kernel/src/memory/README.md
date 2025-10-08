# Memory Management

High-performance memory allocator with graceful OOM handling, address recycling, and production-grade garbage collection.

## Overview

The memory subsystem provides a comprehensive memory management solution with:

- **Segregated free list allocator** for optimal allocation performance
- **Address recycling** for immediate reuse of deallocated memory
- **Block splitting and coalescing** to minimize fragmentation
- **Per-process tracking** with peak usage monitoring
- **Memory pressure detection** with automatic warnings
- **Garbage collection** for automatic cleanup
- **Observability integration** for real-time monitoring

## Architecture

```
memory/
├── manager/           # Core memory manager
│   ├── core/         # Types, traits, and allocation logic
│   │   ├── types.rs        # MemoryBlock, MemoryError, MemoryStats
│   │   ├── traits.rs       # Allocator, MemoryInfo, GarbageCollector
│   │   ├── allocator.rs    # Allocation/deallocation logic
│   │   └── free_list.rs    # Segregated free list implementation
│   ├── process/      # Process-specific operations
│   │   ├── process_ops.rs  # Process memory operations
│   │   └── tracking.rs     # Per-process memory tracking
│   ├── storage/      # Physical memory simulation
│   │   └── storage.rs      # Read/write operations
│   ├── gc/           # Garbage collection
│   │   └── gc.rs           # Internal GC logic
│   └── extensions/   # RAII guards and utilities
│       ├── guard_ext.rs    # Memory allocation guards
│       └── flat_counter.rs # Performance counters
├── gc/               # Global garbage collector
│   └── collector.rs  # System-wide GC strategies
└── README.md         # This file
```

## Allocation Performance

The memory manager uses a **segregated free list** allocator for optimal performance:

### Small Blocks (<4KB)
- **O(1) lookup** via power-of-2 bucketing
- 12 buckets: 64B, 128B, 256B, 512B, 1KB, 2KB, 4KB
- Most allocations fall into this category
- Near-instant allocation from pre-bucketed free lists

### Medium Blocks (4KB-64KB)
- **O(1) lookup** via 4KB increment bucketing
- 15 buckets: 8KB, 12KB, 16KB, ..., 64KB
- Common for process stacks and buffers
- Direct bucket access without searching

### Large Blocks (>64KB)
- **O(log n) lookup** via BTreeMap
- Efficient range queries for best-fit allocation
- Automatic block splitting for larger allocations
- Coalescing adjacent free blocks

**Performance Comparison:**
- **Old implementation**: O(n) linear scan of entire free list
- **New implementation**: O(1) for small/medium, O(log n) for large

## Memory Manager Features

### 1. Address Recycling

Deallocated memory is **immediately available** for reuse:

```rust
let addr1 = manager.allocate(1024, pid)?;
manager.deallocate(addr1)?;

// Address is immediately available for reuse
let addr2 = manager.allocate(1024, pid)?;
// addr2 may reuse addr1's space
```

### 2. Block Splitting

Larger blocks are split when smaller allocations are requested:

```rust
// Free list has a 4KB block
let addr = manager.allocate(1024, pid)?;
// 1KB allocated, 3KB remains in free list as separate block
```

### 3. Coalescing

Adjacent free blocks are merged to reduce fragmentation:

```rust
// Allocate three adjacent blocks
let a = manager.allocate(1024, pid)?;
let b = manager.allocate(1024, pid)?;
let c = manager.allocate(1024, pid)?;

// Deallocate middle block
manager.deallocate(b)?;

// Deallocate adjacent blocks - they coalesce
manager.deallocate(a)?;
manager.deallocate(c)?;
// Result: Single 3KB free block instead of three 1KB blocks
```

### 4. Memory Pressure Tracking

Automatic monitoring with configurable thresholds:

- **Low** (<60% usage): Normal operation
- **Medium** (60-79% usage): Minor pressure
- **High** (80-94% usage): **Warning logs emitted**
- **Critical** (≥95% usage): **Critical logs + automatic GC**

```rust
let stats = manager.stats();
match stats.memory_pressure() {
    MemoryPressure::Critical => {
        // Trigger emergency GC
        manager.force_collect();
    }
    MemoryPressure::High => {
        // Consider GC
        if manager.should_collect() {
            manager.collect();
        }
    }
    _ => {}
}
```

### 5. Per-Process Tracking

Monitor memory usage on a per-process basis:

```rust
// Track allocations
let addr1 = manager.allocate(1024, pid)?;
let addr2 = manager.allocate(2048, pid)?;

// Get process stats
let process_usage = manager.process_memory(pid);
assert_eq!(process_usage, 3072);

// Get detailed process allocations
let allocations = manager.process_allocations(pid);
for block in allocations {
    println!("Block: {:?}", block);
}

// Clean up all process memory
let freed = manager.free_process_memory(pid);
println!("Freed {} bytes from PID {}", freed, pid);
```

### 6. Garbage Collection

Automatic and manual garbage collection:

```rust
// Check if GC should run (based on threshold)
if manager.should_collect() {
    let freed_blocks = manager.collect();
    println!("GC freed {} blocks", freed_blocks);
}

// Force immediate GC
let freed = manager.force_collect();

// Set custom GC threshold
manager.set_threshold(500); // Run GC after 500 deallocated blocks
```

**Default threshold**: 1000 deallocated blocks

## Global Garbage Collector

System-wide garbage collection with multiple strategies:

### GC Strategies

1. **Global**: Collect from all processes
2. **Threshold**: Collect only from processes exceeding a size threshold
3. **Targeted**: Collect from a specific process
4. **Unreferenced**: Clean up only unreferenced blocks

### Usage

```rust
use memory::{GlobalGarbageCollector, GcStrategy};

let gc = GlobalGarbageCollector::new(memory_manager);

// Run global collection
let stats = gc.collect(GcStrategy::Global);
println!("Freed {} bytes from {} processes", 
         stats.freed_bytes, stats.processes_cleaned);

// Threshold-based collection
let stats = gc.collect(GcStrategy::Threshold { 
    threshold: 10 * 1024 * 1024 // 10MB
});

// Auto-collect based on memory pressure
if let Some(stats) = gc.auto_collect() {
    println!("Auto-GC completed: {:?}", stats);
}
```

### Auto-Collection

The global GC automatically triggers based on:

- **Time interval**: Minimum 5 seconds between collections
- **Memory pressure**: Triggers at ≥80% usage
- **Manual check**: `auto_collect()` method

## Memory Guards (RAII)

Safe memory allocation with automatic cleanup:

```rust
use memory::MemoryGuardExt;

// Allocate with automatic cleanup on drop
{
    let guard = manager.allocate_guarded(4096, pid)?;
    let address = guard.address();
    
    // Use memory...
    
} // Memory automatically deallocated here
```

## Error Handling

All memory operations return `MemoryResult<T>` with detailed error information:

```rust
match manager.allocate(size, pid) {
    Ok(addr) => { /* Success */ }
    Err(MemoryError::OutOfMemory { requested, available, used, total }) => {
        eprintln!("OOM: requested {} bytes, only {} available", 
                  requested, available);
        // Trigger GC or free memory
    }
    Err(MemoryError::InvalidAddress(addr)) => {
        eprintln!("Invalid address: 0x{:x}", addr);
    }
    Err(e) => {
        eprintln!("Memory error: {}", e);
    }
}
```

### Error Types

- `OutOfMemory`: System has insufficient memory
- `ProcessLimitExceeded`: Process exceeded its memory limit
- `InvalidAddress`: Address is invalid or out of bounds
- `CorruptionDetected`: Memory corruption detected
- `AlignmentError`: Address doesn't meet alignment requirements
- `ProtectionViolation`: Attempted access without proper permissions

## Observability

Memory operations emit events for monitoring:

```rust
let manager = MemoryManager::new()
    .with_collector(collector);

// Events emitted automatically:
// - memory::allocated (address, size, pid)
// - memory::deallocated (address, size)
// - memory::oom (requested, available)
// - memory::pressure_high (usage_percentage)
// - memory::gc_triggered (freed_blocks)
```

## Performance Characteristics

### Allocation
- **Small (<4KB)**: O(1) - direct bucket lookup
- **Medium (4KB-64KB)**: O(1) - direct bucket lookup
- **Large (>64KB)**: O(log n) - BTreeMap lookup

### Deallocation
- **Simple**: O(1) - mark block as free
- **With coalescing**: O(1) - check adjacent blocks

### Memory Operations
- **Read**: O(1) - direct DashMap lookup
- **Write**: O(1) - direct DashMap access
- **Query stats**: O(1) - atomic reads

### Concurrency
- **Lock-free reads**: DashMap with 128 shards for high contention
- **Atomic operations**: Lock-free counters for used_memory, next_address
- **Cache-line aligned**: 64-byte alignment prevents false sharing

## Configuration

### Memory Pool Size

```rust
// Default: 1GB
let manager = MemoryManager::new();

// Custom size
let manager = MemoryManager::with_capacity(512 * 1024 * 1024); // 512MB
```

### GC Thresholds

```rust
// Set internal GC threshold (deallocated blocks)
manager.set_threshold(500);

// Set global GC threshold (bytes)
gc.set_threshold(50 * 1024 * 1024); // 50MB
```

### Pressure Thresholds

Configured in manager constructor:
- Warning threshold: 80%
- Critical threshold: 95%

## Testing

Comprehensive test coverage in `kernel/tests/memory/`:

- `memory_test.rs`: Core allocation/deallocation tests
- `gc_test.rs`: Garbage collection tests
- `pressure_test.rs`: Memory pressure scenarios

Run tests:

```bash
cd kernel
cargo test --test memory
```

## Integration Example

```rust
use kernel::memory::{MemoryManager, GcStrategy, GlobalGarbageCollector};

// Create memory manager
let mut manager = MemoryManager::new();

// Optional: Add observability
manager.set_collector(collector);

// Create global GC
let gc = GlobalGarbageCollector::new(manager.clone());

// Allocate memory
let addr = manager.allocate(4096, pid)?;

// Write data
manager.write(addr, &data)?;

// Read data
let read_data = manager.read(addr, data.len())?;

// Deallocate
manager.deallocate(addr)?;

// Run GC periodically
if gc.should_collect() {
    let stats = gc.auto_collect();
}
```

## Best Practices

1. **Always check allocation results**: Handle OOM gracefully
2. **Use RAII guards**: Prefer `allocate_guarded()` for automatic cleanup
3. **Monitor memory pressure**: Set up alerts for High/Critical pressure
4. **Configure GC thresholds**: Tune based on workload
5. **Track per-process memory**: Monitor processes with high usage
6. **Use observability**: Enable event collection for production monitoring
7. **Test OOM scenarios**: Simulate memory exhaustion in tests

## Performance Tips

1. **Batch allocations**: Reduce contention by allocating larger blocks
2. **Reuse addresses**: Deallocate and reallocate to maximize recycling
3. **Tune shard counts**: Increase for high contention workloads
4. **Profile GC**: Adjust thresholds based on allocation patterns
5. **Monitor fragmentation**: Check `fragmentation_ratio()` regularly

## Future Enhancements

- [ ] NUMA-aware allocation
- [ ] Memory pools per process
- [ ] Compaction for defragmentation
- [ ] Memory protection (read-only, execute)
- [ ] Shared memory regions
- [ ] Copy-on-write support
- [ ] Memory-mapped files

## References

- [Segregated free list allocator](https://en.wikipedia.org/wiki/Free_list)
- [jemalloc](https://github.com/jemalloc/jemalloc) - Inspiration for bucketing strategy
- [Memory pressure handling in Linux](https://www.kernel.org/doc/html/latest/admin-guide/mm/concepts.html)

