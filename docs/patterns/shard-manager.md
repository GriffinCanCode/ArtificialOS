# Intelligent Shard Configuration Manager

## Overview

The `ShardManager` is a CPU-topology-aware configuration system that calculates optimal shard counts for concurrent data structures (DashMap) based on the host system's hardware characteristics.

## Problem Statement

**Previous Approach (Static):**
- Hardcoded shard counts: 128, 64, 32
- Same configuration on 4-core laptops and 128-core servers
- Manual guesses at contention levels
- No adaptation to hardware reality

**Root Cause:** Static allocation does not adapt to runtime hardware topology and actual workload patterns.

## Solution: CPU-Topology-Aware Dynamic Sharding

### Design Principles

1. **Hardware-Driven**: Shard counts based on actual CPU count
2. **Contention-Aware**: Different multipliers for different access patterns
3. **Zero Runtime Cost**: Computed once at initialization via pure functions with `#[inline]`
4. **Power-of-2 Optimization**: Enables fast modulo via bitwise AND

### Architecture

```rust
// Pure functions with #[inline] for zero-cost abstraction
pub struct ShardManager;

impl ShardManager {
    #[inline]
    pub fn cpu_count() -> usize { /* ... */ }

    #[inline]
    pub fn shards(profile: WorkloadProfile) -> usize { /* ... */ }
}

pub enum WorkloadProfile {
    HighContention,   // 4x CPU cores (blocks, processes, storage)
    MediumContention, // 2x CPU cores (sandboxes, pipes, tracking)
    LowContention,    // 1x CPU cores (metrics, spawn_counts, mmap)
}

// Usage
let shards = ShardManager::shards(WorkloadProfile::HighContention);
let map = DashMap::with_capacity_and_hasher_and_shard_amount(0, RandomState::new(), shards);
```

### Implementation Strategy

Rather than using a singleton pattern, the implementation employs pure functions with `#[inline]` annotations. This allows the compiler to:
- Constant-fold CPU count checks at compile time in many cases
- Inline all calculations into call sites
- Eliminate redundant calls via common subexpression elimination

Result: Faster, simpler, and better inlining than singleton patterns would provide.

### Shard Count Calculation

**Formula:** `(cpu_count * multiplier).next_power_of_two().clamp(8, 512)`

**Multipliers by Profile:**
- **High Contention (4x)**: Heavy concurrent access benefits from maximum parallelism
  - Used by: memory blocks, storage maps, process tables, signal delivery
  - Rationale: These are hot paths with constant concurrent access
  
- **Medium Contention (2x)**: Balanced memory overhead versus parallelism
  - Used by: child tracking, sandboxes, pipes, per-process metrics
  - Rationale: Moderate access patterns do not justify 4x memory overhead
  
- **Low Contention (1x)**: Minimal sharding, saves memory
  - Used by: spawn counts, metrics, mmap
  - Rationale: Rare contention makes extra shards wasteful

**Bounds:**
- Minimum: 8 shards (prevents degeneration on 1-2 core systems)
- Maximum: 512 shards (prevents excessive memory overhead)

### Example Scaling

| System | High | Medium | Low |
|--------|------|--------|-----|
| 4-core laptop | 16 | 8 | 8 |
| 8-core desktop | 32 | 16 | 8 |
| 16-core workstation | 64 | 32 | 16 |
| 32-core server | 128 | 64 | 32 |
| 64-core EPYC | 256 | 128 | 64 |
| 128-core Threadripper | 512 | 256 | 128 |

## Implementation

### Core Module

**File:** `kernel/src/core/sync/management/shard_manager.rs`

Key features:
- Pure functions with `#[inline]` for compiler optimization
- Detects CPU count via `std::thread::available_parallelism()`
- Fallback to sensible defaults if detection fails
- Cache line size constant (64-byte assumption)
- Comprehensive unit tests

### Integrated Components

All DashMap instances across the codebase now use `ShardManager`:

1. **Memory Manager** (`kernel/src/memory/manager/mod.rs`)
   - blocks: HighContention
   - process_tracking: MediumContention
   - memory_storage: HighContention

2. **Process Manager** (`kernel/src/process/manager*.rs`)
   - processes: HighContention
   - child_counts: MediumContention

3. **Pipe Manager** (`kernel/src/ipc/pipe/manager.rs`)
   - pipes: MediumContention
   - process_pipes: LowContention

4. **Sandbox Manager** (`kernel/src/security/sandbox/manager.rs`)
   - sandboxes: MediumContention
   - spawned_counts: LowContention

5. **Signal Manager** (`kernel/src/signals/manager.rs`)
   - processes: HighContention

6. **Metrics Collector** (`kernel/src/monitoring/metrics.rs`)
   - counters, gauges, histograms: LowContention

7. **Mmap Manager** (`kernel/src/ipc/mmap.rs`)
   - mappings: LowContention

## Benefits

### 1. Self-Tuning
- Automatically adapts to hardware; works optimally on any system
- No manual tuning required for different deployments

### 2. Principled Design
- Based on actual CPU topology, not arbitrary guesses
- Clear semantic intent (High/Medium/Low contention)

### 3. Performance
- Better cache locality through CPU-aware allocation
- Reduced lock contention on high-core systems
- Lower memory overhead on low-core systems

### 4. Maintainability
- Centralized configuration (single source of truth)
- Self-documenting code (workload profiles explain intent)
- Easy to adjust multipliers based on profiling

### 5. Future-Proof
- Scales automatically to future hardware (256+ cores)
- No code changes needed for new deployments

## Advanced Optimization Opportunities

### 1. Adaptive Sharding (Future)
Monitor actual contention and dynamically adjust:
```rust
// Hypothetical future enhancement
ShardManager::with_adaptive_monitoring(metrics_collector)
    .auto_adjust_on_contention_threshold(0.80)
```

### 2. NUMA-Aware Sharding
Align shards with NUMA nodes for better locality:
```rust
// Could detect NUMA topology
let numa_nodes = detect_numa_topology();
let shards = align_to_numa(cpu_count * multiplier, numa_nodes);
```

### 3. Workload-Specific Data Structures
Instead of one-size-fits-all DashMap:
- **Read-heavy** (blocks): Use RCU or epoch-based structures
- **Write-heavy** (counters): Use per-CPU counters with lazy aggregation
- **Mixed** (processes): Keep sharded DashMap

### 4. Compile-Time Profiling
Profile during build to determine optimal configurations:
```rust
// Hypothetical: benchmark at build time
const OPTIMAL_SHARDS: usize = benchmark_and_determine!();
```

## Testing

Run tests with:
```bash
cd kernel
cargo test shard_manager
```

Tests verify:
- Power-of-2 property (required for efficient modulo)
- Minimum/maximum bounds enforcement
- Contention level ordering (high >= medium >= low)
- Consistency across calls

## Migration Guide

**Old Code:**
```rust
let map = DashMap::with_capacity_and_hasher_and_shard_amount(
    0,
    RandomState::new(),
    128,  // hardcoded
);
```

**New Code:**
```rust
use crate::core::{ShardManager, WorkloadProfile};

let map = DashMap::with_capacity_and_hasher_and_shard_amount(
    0,
    RandomState::new(),
    ShardManager::shards(WorkloadProfile::HighContention), // CPU-aware
);
```

## Performance Impact

**Expected improvements:**
- **4-16 core systems**: 10-20% better throughput (was over-sharded at 128)
- **64+ core systems**: 20-40% better throughput (was under-sharded at 128)
- **Memory usage**: 20-50% reduction on low-core systems

**Benchmarking:**
```bash
cd kernel
cargo bench --bench sync_benchmark
```

## Related Patterns

This implementation uses several design patterns:
- **Pure Function Pattern**: Zero-cost abstraction via `#[inline]`
- **Strategy Pattern**: Different workload profiles
- **Template Method**: Calculation formula with variable multipliers
- **Compiler Optimization**: Let the compiler constant-fold calculations

## References

- Sharded Slot Pattern: `docs/patterns/sharded-slot.md`
- DashMap Documentation: https://docs.rs/dashmap/
- CPU Topology Detection: https://doc.rust-lang.org/std/thread/fn.available_parallelism.html

