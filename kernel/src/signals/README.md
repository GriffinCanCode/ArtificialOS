# Signals Module

UNIX-style signal handling system for AgentOS kernel.

## Overview

The signals module provides comprehensive signal management for inter-process communication and process control, following UNIX signal semantics with modern enhancements.

## Architecture

### Core Components

#### Types (`types.rs`)

Signal definitions and data structures:

- **`Signal`** - UNIX signal enum (SIGHUP, SIGINT, SIGKILL, etc.)
- **`SignalAction`** - Handler actions (Default, Ignore, Custom Handler, etc.)
- **`SignalDisposition`** - Signal handling strategy
- **`PendingSignal`** - Queued signal with metadata
- **`SignalStats`** - System-wide signal statistics
- **`ProcessSignalState`** - Per-process signal state

#### Traits (`traits.rs`)

Fine-grained signal management interfaces:

- **`SignalDelivery`** - Send and deliver signals
- **`SignalHandlerRegistry`** - Register/unregister handlers
- **`SignalMasking`** - Block/unblock signals
- **`SignalQueue`** - Manage pending signals
- **`SignalStateManager`** - Process lifecycle management
- **`SignalManager`** - Combined trait for complete functionality

#### Handler (`handler.rs`)

Signal action executor:

- **`SignalHandler`** - Executes signal actions on processes
- **`SignalOutcome`** - Result of signal execution
- Validates signals and determines default actions

#### Manager (`manager.rs`)

Central signal coordinator:

- **`SignalManagerImpl`** - Thread-safe signal manager
- Per-process signal queues with capacity limits
- Handler registration and masking support
- Signal delivery with blocking/permission checks

## Features

### Signal Types

31 UNIX signals supported:

- **Process Control**: SIGKILL, SIGTERM, SIGSTOP, SIGCONT
- **Interrupts**: SIGINT, SIGQUIT, SIGHUP
- **Errors**: SIGSEGV, SIGILL, SIGBUS, SIGFPE
- **User-Defined**: SIGUSR1, SIGUSR2
- **Terminal**: SIGTSTP, SIGTTIN, SIGTTOU
- **Timers**: SIGALRM, SIGVTALRM, SIGPROF
- **I/O**: SIGIO, SIGURG, SIGPIPE
- **Resources**: SIGXCPU, SIGXFSZ
- **Other**: SIGCHLD, SIGWINCH, SIGTRAP, etc.

### Signal Handling

- **Default Actions**: Terminate, stop, continue, or ignore
- **Custom Handlers**: Register user-defined handlers
- **Signal Masking**: Block/unblock specific signals
- **Uncatchable Signals**: SIGKILL and SIGSTOP cannot be caught or blocked

### Signal Queue

- Per-process FIFO queue (max 128 pending signals)
- Timestamp tracking for delivery order
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
use ai_os_kernel::signals::{SignalAction, SignalHandlerRegistry};

// Register custom handler
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

### Deliver Pending Signals

```rust
use ai_os_kernel::signals::SignalDelivery;

// Deliver all pending signals to process
let delivered = signal_manager.deliver_pending(pid)?;
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

### Uncatchable Signals

SIGKILL (9) and SIGSTOP (19) bypass:
- Handler registration
- Signal blocking
- Queuing (delivered immediately)

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

## Performance

- **Lock Granularity**: Fine-grained locks per operation type
- **O(1) Signal Lookup**: HashMap-based process storage
- **Bounded Queues**: Prevent unbounded memory growth
- **Atomic Counters**: Lock-free statistics tracking

## Future Enhancements

- Real-time signals (SIGRTMIN-SIGRTMAX)
- Signal sets and masks operations (sigemptyset, sigfillset)
- Signal wait operations (sigwait, sigtimedwait)
- Signal handlers with context (siginfo_t)
- Process groups and session signaling
- Async signal-safe operations
