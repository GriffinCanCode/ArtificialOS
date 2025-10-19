# Preemptive Scheduling Implementation

## Overview

This document describes the preemptive scheduling system implemented in the AgentOS kernel, which provides true OS-level process preemption using POSIX signals.

## Architecture

### Components

1. **Scheduler (`kernel/src/process/scheduler.rs`)**
   - Implements multiple scheduling policies (Round-Robin, Priority, Fair)
   - Tracks process virtual runtime (vruntime) for fair scheduling
   - Enforces time quantum-based preemption decisions
   - Supports dynamic priority and quantum adjustment

2. **Preemption Controller (`kernel/src/process/preemption.rs`)**
   - Bridges scheduler decisions with OS-level process control
   - Uses SIGSTOP/SIGCONT to pause/resume processes
   - Manages context switches between OS processes
   - Tracks currently scheduled process

3. **Scheduler Task (`kernel/src/process/scheduler_task.rs`)**
   - Autonomous background task running at quantum intervals
   - Triggers scheduling decisions periodically
   - Supports dynamic configuration (pause/resume, quantum updates)
   - Can operate with or without OS preemption

4. **Process Manager (`kernel/src/process/manager.rs`)**
   - Orchestrates all components
   - Creates PreemptionController when both scheduler and executor are available
   - Manages process lifecycle and cleanup

## How It Works

### Without Executor (Logical Scheduling)

When the ProcessManager is built without `.with_executor()`:

```rust
let process_manager = ProcessManager::builder()
    .with_scheduler(Policy::Fair)
    .build();
```

The system performs logical scheduling only:
- Scheduler tracks which process should be running
- No OS-level process control (no SIGSTOP/SIGCONT)
- Suitable for managing virtual processes without OS execution

### With Executor (OS-Level Preemption)

When the ProcessManager is built with `.with_executor()`:

```rust
let process_manager = ProcessManager::builder()
    .with_executor()
    .with_scheduler(Policy::Fair)
    .build();
```

The system enables true preemptive multitasking:

1. Process Spawning: When a process is created with an ExecutionConfig, an actual OS process is spawned
2. Preemption Controller Created: The ProcessManager creates a PreemptionController that bridges the scheduler with the executor
3. Autonomous Scheduling: The SchedulerTask runs in the background, calling preemption.schedule() every quantum interval
4. OS-Level Context Switch:
   - When quantum expires, scheduler selects next process
   - If different from current, PreemptionController sends SIGSTOP to old process
   - PreemptionController sends SIGCONT to new process
   - Actual OS-level context switch occurs

### Scheduling Flow

```
┌
                     SchedulerTask (Background)                   
                                                                   
  Every quantum μs:                                               
    if PreemptionController available:                           
      1. Call scheduler.schedule()                                
      2. If process changes:                                      
         - Send SIGSTOP to old process (pause)                    
         - Send SIGCONT to new process (resume)                   
    else:                                                         
      - Just call scheduler.schedule() (logical only)             
┘
                              
                              ▼
┌
                         Scheduler                                
                                                                   
  schedule():                                                     
    1. Check if current process quantum expired                   
    2. If expired, preempt and requeue                           
    3. Select next process based on policy:                       
       - RoundRobin: FIFO order                                  
       - Priority: Highest priority first                         
       - Fair: Lowest vruntime first                             
    4. Return selected PID                                        
┘
                              
                              ▼
┌
                   PreemptionController                           
                                                                   
  schedule():                                                     
    1. Get next PID from scheduler                                
    2. Get OS PID from executor                                   
    3. Send SIGSTOP to old OS process                            
    4. Send SIGCONT to new OS process                            
    5. Track new process as current                              
┘
```

## Scheduling Policies

### Round-Robin
- Processes scheduled in FIFO order
- Each process gets equal time quantum
- Simple and fair for similar workloads

### Priority-Based
- Processes with higher priority values scheduled first
- Within same priority, FIFO order
- Can lead to starvation of low-priority processes

### Fair (CFS-inspired)
- Tracks virtual runtime (vruntime) for each process
- Lower priority processes accumulate vruntime slower (get more CPU time)
- Prevents starvation while respecting priorities
- Most sophisticated and recommended for general use

## Configuration

### Time Quantum

Default: 10ms (10,000 microseconds)

Can be adjusted:
```rust
process_manager.set_time_quantum(5_000)?; // 5ms
```

Smaller quantum = More responsive but higher overhead
Larger quantum = Less overhead but less responsive

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

### Scheduler Control

```rust
// Pause scheduling (processes keep running, no preemption)
if let Some(task) = process_manager.scheduler_task() {
    task.pause();
}

// Resume scheduling
if let Some(task) = process_manager.scheduler_task() {
    task.resume();
}

// Trigger immediate schedule decision
if let Some(task) = process_manager.scheduler_task() {
    task.trigger();
}
```

## Statistics

### Scheduler Statistics

```rust
if let Some(stats) = process_manager.get_scheduler_stats() {
    println!("Total scheduled: {}", stats.total_scheduled);
    println!("Context switches: {}", stats.context_switches);
    println!("Preemptions: {}", stats.preemptions);
    println!("Active processes: {}", stats.active_processes);
}
```

### Per-Process Statistics

```rust
if let Some(stats) = process_manager.get_process_stats(pid) {
    println!("CPU time: {} μs", stats.cpu_time_micros);
    println!("Virtual runtime: {}", stats.vruntime);
    println!("Priority: {}", stats.priority);
}
```

## Platform Support

**Unix/Linux**: Full support with SIGSTOP/SIGCONT

**Windows**: Graceful degradation - logical scheduling only (no OS preemption)

**macOS**: Full support with SIGSTOP/SIGCONT

## Limitations & Future Work

### Current Limitations

1. **No CPU Affinity**: Processes can migrate between cores
2. **No I/O Scheduling**: Only CPU scheduling is managed
3. **Basic cgroups**: Limited cgroups v2 integration
4. **No Real-Time**: SCHED_FIFO/SCHED_RR not implemented

### Potential Improvements

1. **Signal Integration**: Hook scheduler into signal delivery (SignalDeliveryHook exists but not yet integrated)
2. **I/O Scheduler**: Integrate with async I/O subsystem
3. **Advanced cgroups**: CPU quotas, I/O limits, memory pressure
4. **Real-Time Scheduling**: POSIX real-time scheduling classes
5. **Multicore Awareness**: Load balancing across cores

## Example Usage

### Basic Setup (No OS Execution)

```rust
let process_manager = ProcessManager::builder()
    .with_scheduler(Policy::Fair)
    .build();

// Create virtual processes
let pid1 = process_manager.create_process("app1".to_string(), 5);
let pid2 = process_manager.create_process("app2".to_string(), 7);

// Scheduler runs autonomously in background
// Tracks which process should be active
```

### Full Setup (With OS Preemption)

```rust
let process_manager = ProcessManager::builder()
    .with_executor()
    .with_limits()
    .with_scheduler(Policy::Fair)
    .build();

// Spawn actual OS processes
let config = ExecutionConfig::new("my-app".to_string())
    .with_args(vec!["--worker".to_string()]);

let pid = process_manager.create_process_with_command(
    "worker".to_string(),
    7, // priority
    Some(config),
);

// OS process spawned, scheduler automatically preempts at quantum intervals
// SIGSTOP/SIGCONT used for true preemptive multitasking
```

## Testing

Run scheduler tests:
```bash
cargo test --lib scheduler
cargo test --lib preemption
cargo test --lib scheduler_task
```

Integration tests:
```bash
cargo test --test integration_process_test
```

## References

- Linux CFS (Completely Fair Scheduler): Inspiration for vruntime tracking
- POSIX Signals: SIGSTOP, SIGCONT for process control
- Real-Time Linux: Priority handling concepts
