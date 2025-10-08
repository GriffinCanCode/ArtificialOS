# Signals Module

UNIX-style signal handling system for AgentOS kernel with real-time signals, executable callbacks, and automatic delivery.

## Overview

The signals module provides comprehensive signal management for inter-process communication and process control, following UNIX signal semantics with modern enhancements:

- **Executable Handlers**: Function callbacks with actual execution logic
- **Real-Time Signals**: Priority-queued RT signals (SIGRTMIN-SIGRTMAX)
- **Automatic Delivery**: Integrated with process scheduler for seamless signal handling
- **Process Integration**: Signal outcomes automatically affect process states

## Architecture

The signals module is organized into logical domain folders:

```
signals/
├── core/                  # Fundamental types and traits
│   ├── types.rs          # Signal definitions and data structures
│   ├── traits.rs         # Signal management interfaces
│   ├── internal_types.rs # Internal manager types
│   └── atomic_stats.rs   # Lock-free statistics
├── handler/              # Signal execution
│   ├── callbacks.rs      # Executable handler registry
│   └── executor.rs       # Signal action executor
├── management/           # Central management
│   ├── manager.rs        # Signal manager implementation
│   └── delivery.rs       # Scheduler integration hook
└── integration_support/  # Process integration
    └── process.rs        # Process state integration helpers
```

### Core Components

#### Core Types (`core/types.rs`)

Signal definitions and data structures:

- **`Signal`** - UNIX signals + RT signals (SIGRT(34-63))
- **`SignalAction`** - Handler actions (Default, Ignore, Handler(id), etc.)
- **`SignalDisposition`** - Signal handling strategy
- **`PendingSignal`** - Queued signal with metadata and priority
- **`SignalStats`** - System-wide signal statistics
- **`ProcessSignalState`** - Per-process signal state

#### Core Traits (`core/traits.rs`)

Fine-grained signal management interfaces:

- **`SignalDelivery`** - Send and deliver signals
- **`SignalHandlerRegistry`** - Register/unregister handlers
- **`SignalMasking`** - Block/unblock signals
- **`SignalQueue`** - Manage pending signals
- **`SignalStateManager`** - Process lifecycle management
- **`SignalManager`** - Combined trait for complete functionality

#### Handler Callbacks (`handler/callbacks.rs`)

Executable handler registry:

- **`CallbackRegistry`** - Thread-safe handler storage
- **`HandlerFn`** - Function pointer type: `Fn(Pid, Signal) -> Result<()>`
- **`register()`** - Register executable callback, returns handler ID
- **`execute()`** - Execute handler by ID with actual code execution

#### Handler Executor (`handler/executor.rs`)

Signal action executor:

- **`SignalHandler`** - Executes signal actions on processes
- **`SignalOutcome`** - Result of signal execution
- Validates signals and determines default actions

#### Signal Manager (`management/manager.rs`)

Central signal coordinator:

- **`SignalManagerImpl`** - Thread-safe signal manager
- Priority-based signal queues (BinaryHeap for RT signals)
- Handler registration with executable callbacks
- Signal delivery with blocking/permission checks
- Automatic priority ordering (RT signals > standard signals)

#### Delivery Hook (`management/delivery.rs`)

Automatic signal delivery integration:

- **`SignalDeliveryHook`** - Scheduler integration hook
- **`deliver_before_schedule()`** - Auto-deliver signals before process runs
- **`should_schedule()`** - Check if process should be scheduled
- **`pending_count()`** - Get pending signal count for priority

#### Process Integration (`integration_support/process.rs`)

Process state integration:

- **`outcome_to_state()`** - Convert signal outcome to process state
- **`requires_immediate_action()`** - Check if outcome needs immediate handling
- **`should_interrupt()`** - Check if outcome interrupts execution

## Features

### Signal Types

31 standard UNIX signals + 30 real-time signals:

**Standard Signals (1-31)**:
- **Process Control**: SIGKILL, SIGTERM, SIGSTOP, SIGCONT
- **Interrupts**: SIGINT, SIGQUIT, SIGHUP
- **Errors**: SIGSEGV, SIGILL, SIGBUS, SIGFPE
- **User-Defined**: SIGUSR1, SIGUSR2
- **Terminal**: SIGTSTP, SIGTTIN, SIGTTOU
- **Timers**: SIGALRM, SIGVTALRM, SIGPROF
- **I/O**: SIGIO, SIGURG, SIGPIPE
- **Resources**: SIGXCPU, SIGXFSZ
- **Other**: SIGCHLD, SIGWINCH, SIGTRAP, etc.

**Real-Time Signals (34-63)**:
- **SIGRT(n)**: Priority-ordered signals (higher number = higher priority)
- Always delivered in priority order
- Never coalesced (all instances delivered)
- Can carry additional data

### Signal Handling

- **Default Actions**: Terminate, stop, continue, or ignore
- **Executable Callbacks**: Register functions that actually execute
- **Signal Masking**: Block/unblock specific signals
- **Uncatchable Signals**: SIGKILL and SIGSTOP cannot be caught or blocked
- **Automatic Delivery**: Signals delivered before process execution
- **Priority Ordering**: RT signals delivered before standard signals

### Signal Queue

- Per-process priority queue (max 128 pending signals)
- Binary heap for O(log n) priority operations
- RT signals delivered before standard signals
- Timestamp tracking for same-priority ordering
- Blocked signal queuing
- Automatic cleanup on process termination

### Statistics

Track system-wide metrics:
- Total signals sent/delivered/blocked/queued
- Pending signal count
- Handler registration count

## Usage

### Initialize Signal Manager

```rust
use ai_os_kernel::signals::SignalManagerImpl;

let signal_manager = SignalManagerImpl::new();

// Initialize process
signal_manager.initialize_process(pid)?;
```

### Send Signals

```rust
use ai_os_kernel::signals::{Signal, SignalDelivery};

// Send SIGTERM to process
signal_manager.send(sender_pid, target_pid, Signal::SIGTERM)?;

// Broadcast signal to all processes
let delivered = signal_manager.broadcast(sender_pid, Signal::SIGHUP)?;
```

### Register Handlers

```rust
use ai_os_kernel::signals::{Signal, SignalAction, SignalHandlerRegistry};

// Register executable callback
let callbacks = signal_manager.callbacks();
let handler_id = callbacks.register(|pid, signal| {
    println!("Handler executed for PID {} signal {:?}", pid, signal);
    Ok(())
});

// Register handler with signal manager
signal_manager.register_handler(
    pid,
    Signal::SIGUSR1,
    SignalAction::Handler(handler_id)
)?;

// Ignore signal
signal_manager.register_handler(
    pid,
    Signal::SIGPIPE,
    SignalAction::Ignore
)?;
```

### Send Real-Time Signals

```rust
use ai_os_kernel::signals::{Signal, SignalDelivery};

// Send RT signal with high priority
signal_manager.send(sender_pid, target_pid, Signal::SIGRT(63))?;

// Lower priority RT signal
signal_manager.send(sender_pid, target_pid, Signal::SIGRT(34))?;

// RT signals are delivered in priority order (63 before 34)
```

### Block Signals

```rust
use ai_os_kernel::signals::SignalMasking;

// Block specific signal
signal_manager.block_signal(pid, Signal::SIGINT)?;

// Set signal mask
signal_manager.set_mask(pid, vec![
    Signal::SIGINT,
    Signal::SIGTERM,
])?;

// Unblock signal
signal_manager.unblock_signal(pid, Signal::SIGINT)?;
```

### Automatic Signal Delivery

```rust
use ai_os_kernel::signals::{SignalDeliveryHook, SignalDelivery};

// Create delivery hook
let hook = SignalDeliveryHook::new(signal_manager.clone());

// Automatically deliver before scheduling (called by scheduler)
let (count, terminated, stopped, continued) = hook.deliver_before_schedule(pid);

// Or manually deliver all pending signals
let delivered = signal_manager.deliver_pending(pid)?;
```

### Process State Integration

```rust
use ai_os_kernel::signals::{SignalOutcome, outcome_to_state};

// Convert signal outcome to process state
let outcome = SignalOutcome::Stopped;
if let Some(new_state) = outcome_to_state(outcome) {
    process_manager.set_state(pid, new_state)?;
}
```

### Query Signal State

```rust
use ai_os_kernel::signals::{SignalQueue, SignalStateManager};

// Check for pending signals
if signal_manager.has_pending(pid) {
    let signals = signal_manager.pending_signals(pid);
    println!("Pending: {:?}", signals);
}

// Get full process state
let state = signal_manager.get_state(pid)?;
println!("Blocked: {:?}", state.blocked_signals);
println!("Handlers: {:?}", state.handlers);

// Get statistics
let stats = signal_manager.stats();
println!("Total sent: {}", stats.total_signals_sent);
```

## Integration with Syscalls

The signal system integrates with the syscall layer through `syscalls/signals.rs`:

```rust
// Send signal (requires KillProcess capability)
executor.send_signal(pid, target_pid, 15)?; // SIGTERM

// Register handler
executor.register_signal_handler(pid, 10, handler_id)?; // SIGUSR1

// Block/unblock
executor.block_signal(pid, 2)?; // SIGINT
executor.unblock_signal(pid, 2)?;

// Query pending
let pending = executor.get_pending_signals(pid)?;
```

## Configuration

### Limits

- `MAX_PENDING_SIGNALS`: 128 signals per process
- `MAX_HANDLERS_PER_PROCESS`: 32 custom handlers per process
- `SIGRTMIN`: 34 (first RT signal)
- `SIGRTMAX`: 63 (last RT signal)

### Uncatchable Signals

SIGKILL (9) and SIGSTOP (19) bypass:
- Handler registration
- Signal blocking
- Queuing (delivered immediately)

### Priority System

- RT signals: priority = 1000 + signal_number (1034-1063)
- Standard signals: priority = signal_number (1-31)
- Higher priority = delivered first
- Same priority = older timestamp first

## Testing

```bash
# Run signal tests
cargo test --lib signals

# Run integration tests
cargo test --test '*' -- --nocapture signals
```

## Safety and Security

- **Permission Checks**: Signal sending requires `KillProcess` capability
- **Resource Limits**: Queue size prevents memory exhaustion
- **Thread-Safe**: All operations use `Arc<RwLock<T>>` for safe concurrent access
- **Process Isolation**: Per-process signal state prevents cross-contamination
- **Handler Isolation**: Callbacks executed in isolated context
- **Priority Enforcement**: RT signals cannot be deprioritized

## Performance

- **Lock Granularity**: Fine-grained locks per operation type
- **O(1) Signal Lookup**: HashMap-based process storage
- **O(log n) Priority Queue**: Binary heap for efficient priority delivery
- **Bounded Queues**: Prevent unbounded memory growth
- **Atomic Counters**: Lock-free statistics tracking
- **Zero-Copy Callbacks**: Arc-based function pointers

## Key Improvements

✅ **Real-Time Signals**: 30 RT signals (34-63) with priority queuing  
✅ **Executable Handlers**: Actual function callbacks instead of string IDs  
✅ **Automatic Delivery**: Integration with scheduler for seamless signal handling  
✅ **Process Integration**: Signal outcomes automatically affect process states  

## Future Enhancements

- Signal sets and masks operations (sigemptyset, sigfillset)
- Signal wait operations (sigwait, sigtimedwait)
- Signal handlers with context (siginfo_t)
- Process groups and session signaling
- Async signal-safe operations
- Signal data payload for RT signals
