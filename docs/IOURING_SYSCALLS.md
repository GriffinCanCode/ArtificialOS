# io_uring-style Async Syscall Completion

This document describes the io_uring-inspired async syscall completion system for AgentOS.

## Overview

The io_uring-style completion system provides efficient batched syscall submission and completion for I/O-heavy operations. It augments the existing AsyncTaskManager with a more efficient model for operations that benefit from it, without replacing existing patterns.

## Architecture

### Key Components

1. **IoUringManager**: Main interface for creating and managing completion rings
2. **SyscallCompletionRing**: Per-process ring with submission and completion queues
3. **SyscallSubmissionEntry**: Represents a syscall to be executed asynchronously
4. **SyscallCompletionEntry**: Represents the result of an executed syscall
5. **IoUringExecutor**: Executes operations from submission queues

### Queues

- **Submission Queue (SQ)**: Holds pending syscall operations
  - Default size: 256 entries
  - Operations are assigned sequence numbers for tracking
  
- **Completion Queue (CQ)**: Holds completed syscall results
  - Default size: 512 entries
  - Ring buffer behavior (oldest entries dropped if full)

## Best Candidates for io_uring Completion

The following syscalls benefit most from io_uring-style completion:

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

### IPC Operations
- `IpcSend` - Asynchronous IPC message send
- `IpcRecv` - Asynchronous IPC message receive

## Usage Patterns

### Basic Submission and Completion

```rust
use ai_os_kernel::syscalls::{IoUringManager, SyscallSubmissionEntry};

// Submit a file read operation
let entry = SyscallSubmissionEntry::read_file(
    pid,
    PathBuf::from("/path/to/file"),
    user_data, // For correlation
);

let seq = manager.submit(pid, entry)?;

// Later, reap completions
let completions = manager.reap_completions(pid, Some(32))?;
for completion in completions {
    if completion.status.is_success() {
        // Handle success
        let result = completion.result;
    }
}
```

### Batch Submission

```rust
// Submit multiple operations at once
let entries = vec![
    SyscallSubmissionEntry::read_file(pid, path1, 1),
    SyscallSubmissionEntry::read_file(pid, path2, 2),
    SyscallSubmissionEntry::write_file(pid, path3, data, 3),
];

let seqs = manager.submit_batch(pid, entries)?;

// All operations execute concurrently
```

### Blocking Wait for Specific Completion

```rust
// Wait for a specific operation to complete
let seq = manager.submit(pid, entry)?;
let completion = manager.wait_completion(pid, seq)?;
```

## Integration with Existing Patterns

### Coexistence with AsyncTaskManager

The io_uring system **complements** the existing AsyncTaskManager:

- **AsyncTaskManager**: Best for long-running, complex operations
  - Process spawning
  - Long-running computations
  - Operations requiring progress tracking
  
- **io_uring**: Best for I/O-bound operations
  - File reads/writes
  - Network I/O
  - Operations that benefit from batching

Both can be used simultaneously without conflict.

### Integration with AsyncSyscallHandler

The `IoUringAsyncHandler` integrates with the existing `AsyncSyscallHandlerRegistry`:

```rust
let registry = AsyncSyscallHandlerRegistry::new()
    .register(Arc::new(IoUringAsyncHandler::new(manager)));
```

This allows io_uring operations to be used through the async handler interface.

### Integration with Zero-Copy IPC

io_uring IPC operations (`IpcSend`, `IpcRecv`) integrate with the existing `ZeroCopyIpc` system, providing async completion for zero-copy transfers.

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
- Better for high-throughput scenarios

## Configuration

### Ring Sizes

```rust
// Create ring with custom sizes
manager.create_ring(
    pid,
    Some(512),  // SQ size
    Some(1024), // CQ size
)?;
```

### Blocking vs Non-blocking

```rust
// Blocking mode: wait for completion immediately
let handler = IoUringHandler::new(manager, true);

// Non-blocking mode: return task ID for later polling
let handler = IoUringHandler::new(manager, false);
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

## Comparison with Other Approaches

### vs AsyncTaskManager
- **io_uring**: Lower overhead, better for I/O-bound operations
- **AsyncTaskManager**: Better for complex, long-running operations

### vs Synchronous Syscalls
- **io_uring**: Non-blocking, better throughput
- **Synchronous**: Simpler, better for single operations

### vs Streaming Syscalls
- **io_uring**: Better for multiple small operations
- **Streaming**: Better for large data transfers

## Future Enhancements

Potential improvements:
1. True kernel-level io_uring support (currently uses tokio)
2. Registered buffers for zero-copy I/O
3. Linked operations (dependencies between operations)
4. Timeout support per operation
5. Priority queues for urgent operations
