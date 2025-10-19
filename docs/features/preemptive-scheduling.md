# Preemptive Scheduling Implementation

## Overview

This document describes the CPU scheduler in the AgentOS kernel, which manages process scheduling with multiple policies including round-robin, priority-based, and fair scheduling.

## Architecture

### Components

1. **Scheduler (`kernel/src/process/scheduler/mod.rs`)**
   - Implements multiple scheduling policies (Round-Robin, Priority, Fair)
   - Tracks process virtual runtime (vruntime) for fair scheduling
   - Enforces time quantum-based preemption
   - Lock-free atomic statistics for high-frequency updates
   - Cache-line aligned for optimal performance

2. **Scheduling Policies**
   - Round-robin scheduling with VecDeque queue
   - Priority-based scheduling with binary heap
   - Fair scheduling (CFS-inspired) with vruntime tracking

3. **Process Entries**
   - Tracks per-process scheduling state (quantum remaining, vruntime, priority)
   - Location index for O(1) lookup across queues
   - Integration with process manager lifecycle

4. **Scheduler Task (`kernel/src/process/scheduler/task.rs`)**
   - Autonomous background task running at quantum intervals
   - Periodically invokes scheduling decisions
   - Supports pause/resume for testing and control

## Scheduling Policies

### Round-Robin

- Processes scheduled in FIFO order
- Each process gets equal time quantum
- Simple and fair for similar workloads
- Queue-based: VecDeque for O(1) enqueue/dequeue

### Priority-Based

- Processes with higher priority values scheduled first
- Within same priority, FIFO order
- Can lead to starvation of low-priority processes
- Queue-based: BinaryHeap (max-heap by priority)

### Fair (CFS-inspired)

- Tracks virtual runtime (vruntime) for each process
- Lower priority processes accumulate vruntime slower (get more CPU time)
- Prevents starvation while respecting priorities
- Queue-based: BinaryHeap (min-heap by vruntime)
- Recommended for general-purpose workloads

## How It Works

### Scheduler Operation

1. **Scheduling Decision**: At each quantum interval, the scheduler checks if the current process's quantum has expired
2. **Preemption Check**: If expired, the current process is preempted and requeued
3. **Selection**: Next process selected based on active policy
4. **Update Tracking**: Statistics updated for process location and policy state

### Integration with Process Manager

The scheduler is integrated with the ProcessManager:

```rust
let process_manager = ProcessManager::builder()
    .with_scheduler(Policy::Fair)
    .build();
```

### SchedulerTask

The autonomous scheduler task runs at configurable intervals:

```rust
// Pause scheduling (processes keep running, no new scheduling decisions)
if let Some(task) = process_manager.scheduler_task() {
    task.pause();
}

// Resume scheduling
if let Some(task) = process_manager.scheduler_task() {
    task.resume();
}

// Trigger immediate scheduling decision
if let Some(task) = process_manager.scheduler_task() {
    task.trigger();
}
```

## Configuration

### Time Quantum

Default: 10 milliseconds (10,000 microseconds)

Can be adjusted:
```rust
process_manager.set_time_quantum(5_000)?; // 5 milliseconds
```

Smaller quantum: More responsive but higher overhead
Larger quantum: Less overhead but less responsive

### Priority

Priorities range from 0-10 (default: 5)

```rust
// Boost priority (increase by 1, max 10)
process_manager.boost_process_priority(pid)?;

// Lower priority (decrease by 1, min 0)
process_manager.lower_process_priority(pid)?;

// Set directly
process_manager.set_process_priority(pid, 8);
```

## Statistics

### Scheduler Statistics

```rust
if let Some(stats) = process_manager.get_scheduler_stats() {
    println!("Total scheduled: {}", stats.total_scheduled);
    println!("Context switches: {}", stats.context_switches);
    println!("Active processes: {}", stats.active_processes);
}
```

### Per-Process Statistics

```rust
if let Some(stats) = process_manager.get_process_stats(pid) {
    println!("CPU time: {} microseconds", stats.cpu_time_micros);
    println!("Priority: {}", stats.priority);
}
```

## Implementation Characteristics

### Performance Optimizations

- **Cache-line Aligned**: Scheduler structure is 64-byte aligned to prevent false sharing
- **Lock-free Statistics**: AtomicSchedulerStats for zero-contention monitoring
- **O(log n) Operations**: Binary heaps for priority and fair queues
- **O(1) Location Lookup**: DashMap index for process location queries

### Observability

- Integration with monitoring::Collector for event streaming
- Scheduling decisions can emit observability events
- Statistics tracked via atomic counters

## Use Cases

### Virtual Process Scheduling (No OS Execution)

```rust
let process_manager = ProcessManager::builder()
    .with_scheduler(Policy::Fair)
    .build();

// Create virtual processes
let pid1 = process_manager.create_process("app1".to_string(), 5);
let pid2 = process_manager.create_process("app2".to_string(), 7);

// Scheduler runs in background
// Tracks which process should be active
```

### Logical Scheduling Only

When ProcessManager is built without `.with_executor()`:

```rust
let process_manager = ProcessManager::builder()
    .with_scheduler(Policy::Fair)
    .build();
```

The system performs logical scheduling only:
- Scheduler tracks which process should be running
- No OS-level process control
- Suitable for managing virtual processes

## Platform Support

**Unix/Linux**: Full support with configurable scheduling

**Windows**: Compatible with process management

**macOS**: Full support with configurable scheduling

## Limitations

1. **No CPU Affinity**: Processes can migrate between cores
2. **No I/O Scheduling**: Only CPU scheduling is managed
3. **Limited cgroups**: No cgroups v2 integration
4. **No Real-Time**: SCHED_FIFO/SCHED_RR not implemented

## Example Usage

### Basic Setup

```rust
let process_manager = ProcessManager::builder()
    .with_scheduler(Policy::Fair)
    .build();

// Create virtual processes
let pid1 = process_manager.create_process("app1".to_string(), 5);
let pid2 = process_manager.create_process("app2".to_string(), 7);

// Scheduler runs autonomously in background
// Tracks which process should be active based on policy
```

### Policy Changes

```rust
// Change scheduling policy at runtime
process_manager.set_scheduler_policy(Policy::Priority);

// This requeues all processes under the new policy
```

## Testing

Run scheduler tests:
```bash
cargo test --lib scheduler
cargo test --test process
```

Tests cover:
- Round-robin scheduling
- Priority-based scheduling
- Fair scheduling (vruntime)
- Process addition/removal
- Policy changes
- Quantum enforcement

## References

- Linux CFS (Completely Fair Scheduler): Inspiration for vruntime tracking
- Process scheduling algorithms
- Lock-free data structures for high-frequency updates
