# io_uring-style Async Syscall Completion

This document describes the io_uring-inspired async syscall completion system for efficient batched I/O operations.

## Overview

The io_uring-style completion system provides efficient batched syscall submission and completion for I/O-heavy operations. It augments the existing AsyncSyscallExecutor with a more efficient batched model for I/O operations, without replacing existing patterns.

## Architecture

### Key Components

1. **IoUringManager**: Main interface for creating and managing completion rings
2. **SyscallCompletionRing**: Per-process ring with submission and completion queues
3. **SyscallSubmissionEntry**: Represents a syscall to be executed asynchronously
4. **SyscallCompletionEntry**: Represents the result of an executed syscall
5. **IoUringExecutor**: Executes operations from submission queues
6. **IoUringHandler**: Integrates with existing syscall handler registry

### Queues

- **Submission Queue (SQ)**: Holds pending syscall operations
  - Default size: 256 entries
  - Operations are assigned sequence numbers for tracking
  
- **Completion Queue (CQ)**: Holds completed syscall results
  - Default size: 512 entries
  - Ring buffer behavior (oldest entries dropped if full)

## Best Candidates for io_uring Completion

### File I/O Operations
- `ReadFile` - Asynchronous file reads
- `WriteFile` - Asynchronous file writes
- `Open` - Asynchronous file opening
- `Close` - Asynchronous file closing
- `Fsync` - Asynchronous file synchronization
- `Lseek` - Asynchronous seek operations

### Network I/O Operations
- `Send` - Asynchronous socket send
- `Recv` - Asynchronous socket receive
- `Accept` - Asynchronous connection accept
- `Connect` - Asynchronous connection establishment
- `SendTo` - Asynchronous UDP send
- `RecvFrom` - Asynchronous UDP receive

## Usage Patterns

### Basic Submission

```rust
use ai_os_kernel::syscalls::{IoUringManager, SyscallSubmissionEntry, SyscallOpType};

let manager = IoUringManager::new();

// Create a file read operation
let entry = SyscallSubmissionEntry::new(
    pid,
    SyscallOpType::Read { fd: 3, size: 4096 },
    0, // user_data for correlation
);

let seq = manager.submit(pid, entry)?;

// Later, reap completions
let completions = manager.reap_completions(pid, Some(32))?;
for completion in completions {
    if completion.status.is_success() {
        let result = &completion.result;
    }
}
```

### Blocking Wait for Completion

```rust
let seq = manager.submit(pid, entry)?;

// Wait for a specific operation to complete
let completion = manager.wait_completion(pid, seq)?;
```

## Integration with Existing Patterns

### Coexistence with AsyncSyscallExecutor

The io_uring system complements the existing AsyncSyscallExecutor:

- **AsyncSyscallExecutor**: General-purpose async syscall execution
  - Supports all syscalls
  - Flexible, but less optimized for I/O batching
  
- **io_uring**: Optimized for I/O-bound operations
  - File reads/writes
  - Network I/O
  - Operations that benefit from batching

Both can be used simultaneously without conflict.

### Integration with Handler Registry

The `IoUringHandler` integrates with the existing `SyscallHandlerRegistry`:

```rust
let manager = Arc::new(IoUringManager::new());
let handler = IoUringHandler::new(manager, true); // blocking_mode=true
// handler can be registered with handler registry
```

## Performance Benefits

### Batching
- Submit multiple operations at once
- Amortize submission overhead
- Improve cache locality

### Async Completion
- Non-blocking I/O operations
- Better CPU utilization
- Reduced context switching

### Efficient Polling
- Reap multiple completions in one call
- Minimize syscall overhead

## Configuration

### Ring Sizes

```rust
// Default sizes are used by most workloads
let manager = IoUringManager::new();

// Custom sizes can be configured per process
manager.create_ring(
    pid,
    Some(512),  // SQ size
    Some(1024), // CQ size
)?;
```

## Error Handling

Completions can have three statuses:
- `Success`: Operation completed successfully
- `Error(String)`: Operation failed with error message
- `Cancelled`: Operation was cancelled

```rust
match completion.status {
    SyscallCompletionStatus::Success => {
        // Handle success
    }
    SyscallCompletionStatus::Error(msg) => {
        // Handle error
    }
    SyscallCompletionStatus::Cancelled => {
        // Handle cancellation
    }
}
```

## Statistics

Monitor io_uring performance:

```rust
let stats = manager.stats();
println!("Active rings: {}", stats.active_rings);
println!("Submissions: {}", stats.total_submissions);
println!("Completions: {}", stats.total_completions);
println!("Pending: {}", stats.pending);
```

## Implementation Details

### Operation Types

The `SyscallOpType` enum covers I/O-bound operations:

```rust
pub enum SyscallOpType {
    // File I/O
    Read { fd: i32, size: usize },
    Write { fd: i32, data: Vec<u8> },
    Open { path: String, flags: u32 },
    Close { fd: i32 },
    Fsync { fd: i32 },
    Lseek { fd: i32, offset: i64, whence: i32 },
    
    // Network I/O
    Send { sockfd: i32, size: usize, flags: u32 },
    Recv { sockfd: i32, size: usize, flags: u32 },
    Accept { sockfd: i32 },
    Connect { sockfd: i32, addr: Vec<u8> },
    SendTo { sockfd: i32, size: usize, flags: u32 },
    RecvFrom { sockfd: i32, size: usize, flags: u32 },
}
```

### Executor

The IoUringExecutor handles actual execution:

```rust
pub struct IoUringExecutor {
    // Tokio-based executor for async operations
    // (not true kernel io_uring, but provides similar interface)
}
```

## Comparison with Alternatives

### vs AsyncSyscallExecutor
- **io_uring**: Lower overhead, batched, optimized for I/O
- **AsyncSyscallExecutor**: More flexible, supports all syscalls

### vs Synchronous Syscalls
- **io_uring**: Non-blocking, better throughput
- **Synchronous**: Simpler, blocks calling task

## Testing

```bash
cargo test --lib iouring
cargo test --test iouring
```

## References

- Linux io_uring documentation
- io_uring man pages
- Ring buffer design patterns
