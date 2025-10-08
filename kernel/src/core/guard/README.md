# RAII Resource Guards

Type-safe, observable, composable resource guards with automatic cleanup.

## Overview

This module provides a comprehensive RAII guard framework that goes beyond simple Drop implementations. Guards encode state in types, emit observability events automatically, and compose cleanly.

## Design Principles

1. **Type-State Pattern**: Resource state encoded in types for compile-time safety
2. **Observable-First**: All guards integrate with the monitoring system
3. **Composable**: Guards can be combined and managed as a unit
4. **Recoverable**: Poisoned guards can be recovered gracefully
5. **Zero-Cost**: Compiles to the same code as manual management

## Quick Start

### Memory Guard

```rust
use ai_os_kernel::core::guard::MemoryGuard;

let manager = Arc::new(MemoryManager::new());
let address = manager.allocate(1024, pid)?;

// Create guard for automatic cleanup
let guard = MemoryGuard::new(address, 1024, pid, manager, None);

// Use the memory
let addr = guard.address();

// Automatically freed on drop
```

### Lock Guard with Type-States

```rust
use ai_os_kernel::core::guard::{LockGuard, Locked, Unlocked};

let unlocked: LockGuard<Data, Unlocked> = LockGuard::new(data);

// Lock changes the type
let locked: LockGuard<Data, Locked> = unlocked.lock()?;

// Can only access when locked (compile-time check!)
let value = locked.access();

// Unlock changes type back
let unlocked = locked.unlock();
```

### Transaction Guard

```rust
use ai_os_kernel::core::guard::{TransactionGuard, Operation};

let mut tx = TransactionGuard::new(
    Some(pid),
    |ops| { /* commit */ Ok(()) },
    |ops| { /* rollback */ Ok(()) },
);

tx.add_operation(Operation::new("insert", data))?;

// Either commit explicitly or auto-rollback on drop
tx.commit()?;
```

### Composite Guard

```rust
use ai_os_kernel::core::guard::CompositeGuardBuilder;

let composite = CompositeGuardBuilder::new()
    .with("memory", memory_guard)
    .with("ipc", ipc_guard)
    .with("socket", socket_guard)
    .build();

// All guards managed together, released in LIFO order
```

## Guard Types

| Guard | Purpose | Key Features |
|-------|---------|--------------|
| `MemoryGuard` | Scoped memory allocations | Auto-deallocation, observability |
| `MemoryGuardRef` | Shared memory ownership | Reference-counted |
| `LockGuard<S>` | Type-safe locking | State in types, poisoning support |
| `IpcGuard` | IPC resources | Pipes, queues, shm, rings |
| `IpcGuardRef` | Shared IPC ownership | Reference-counted |
| `TransactionGuard` | Atomic operations | Auto-rollback, panic recovery |
| `CompositeGuard` | Multiple resources | LIFO cleanup, unified lifecycle |
| `TypedGuard<T,S>` | Generic type-state | State transitions |
| `ObservableGuard` | Add observability | Wraps any guard |

## Core Traits

### Guard

Base trait for all guards:

```rust
pub trait Guard: Send {
    fn resource_type(&self) -> &'static str;
    fn metadata(&self) -> &GuardMetadata;
    fn is_active(&self) -> bool;
    fn release(&mut self) -> GuardResult<()>;
}
```

### GuardDrop

Separates Drop logic for testability:

```rust
pub trait GuardDrop: Guard {
    fn on_drop(&mut self);
}
```

### Recoverable

For guards that can be poisoned and recovered:

```rust
pub trait Recoverable: Guard {
    fn is_poisoned(&self) -> bool;
    fn recover(&mut self) -> GuardResult<()>;
    fn poison(&mut self, reason: String);
}
```

### Observable

Automatic event emission:

```rust
pub trait Observable: Guard {
    fn emit_created(&self);
    fn emit_used(&self, operation: &str);
    fn emit_dropped(&self);
    fn emit_error(&self, error: &GuardError);
}
```

### GuardRef

Reference-counted guards:

```rust
pub trait GuardRef: Guard + Clone {
    fn ref_count(&self) -> usize;
    fn is_last_ref(&self) -> bool;
}
```

## Advanced Patterns

### Type-State Transitions

```rust
struct Process { /* ... */ }

// Start uninitialized
let guard: TypedGuard<Process, Uninitialized> = TypedGuard::new(process, "process");

// Transition to initialized
let guard: TypedGuard<Process, Initialized> = guard.with_transition(|p| {
    p.initialize()?;
    Ok(())
})?;

// Transition to running
let guard: TypedGuard<Process, Running> = guard.transition();

// Type system prevents invalid transitions at compile time!
```

### Observable Wrapper

Wrap any guard to add observability:

```rust
let guard = SomeGuard::new(...);
let observable = ObservableGuard::wrap(guard, collector);

// Now emits events automatically
observable.with_operation("operation_name", |g| {
    // Do work
});
```

### Error Handling

```rust
let mut guard = create_guard()?;

match guard.release() {
    Ok(()) => { /* Success */ },
    Err(GuardError::AlreadyReleased) => { /* Already cleaned up */ },
    Err(GuardError::OperationFailed(e)) => { /* Cleanup failed */ },
    Err(e) => { /* Other error */ },
}
```

### Poisoning and Recovery

```rust
let mut guard = LockGuard::new(data);

if guard.is_poisoned() {
    match guard.recover() {
        Ok(()) => { /* Recovered! */ },
        Err(e) => { /* Recovery failed */ },
    }
}
```

## Performance

- **Zero-cost abstractions**: Type-state pattern compiles to same code as manual management
- **No allocations**: Guards store data inline or in Arc
- **Lock-free**: Where possible (e.g., MemoryGuard)
- **Cache-friendly**: Aligned structures for hot paths
- **Minimal overhead**: ~24-32 bytes per guard

## Testing

Comprehensive test suite in `tests/guard/`:

- Memory guards (auto-cleanup, ref-counting)
- Lock guards (type-states, poisoning)
- IPC guards (all resource types)
- Transaction guards (commit/rollback)
- Composite guards (LIFO ordering)
- Typed guards (state transitions)
- Observable guards (event emission)
- Integration tests (combined patterns)

## Comparison to Standard RAII

| Feature | Standard Rust | This Implementation |
|---------|--------------|---------------------|
| Auto-cleanup | ✅ Drop trait | ✅ Drop trait |
| Type-state | ❌ Manual | ✅ Automatic |
| Observability | ❌ Manual | ✅ Automatic |
| Composable | ❌ Manual | ✅ Built-in |
| Recoverable | ❌ No | ✅ Yes |
| Transactions | ❌ Manual | ✅ Built-in |
| Ref-counted | ✅ Arc | ✅ GuardRef trait |

## Best Practices

1. **Use type-states** for resources with clear state machines
2. **Enable observability** for production guards
3. **Compose guards** instead of manually managing multiple resources
4. **Prefer guards** over manual cleanup (reduces tech debt)
5. **Use transactions** for atomic multi-step operations
6. **Handle poisoning** gracefully in long-running services

## Integration

### With Memory Manager

```rust
// Extension trait for MemoryManager
impl MemoryGuardExt for MemoryManager {
    fn allocate_guard(&self, size: Size, pid: Pid) 
        -> Result<MemoryGuard, MemoryError>;
}

let guard = memory_manager.allocate_guard(1024, pid)?;
```

### With Observability

```rust
// All guards can be observable
let guard = MemoryGuard::new(addr, size, pid, manager, Some(collector));

// Or wrap existing guards
let observable = ObservableGuard::wrap(guard, collector);
```

### With Process Resources

Guards complement the `ResourceCleanup` system:

- **Guards**: Scoped, per-operation cleanup
- **ResourceCleanup**: Process-wide cleanup on termination

Both work together to ensure no resource leaks.

## Future Enhancements

- Async guards (for async operations)
- Guards for file descriptors
- Guards for process handles
- Guards for GPU resources
- Integration with process lifecycle manager

## References

- **RAII**: Resource Acquisition Is Initialization
- **Type-State Pattern**: State encoded in types
- **Rust Drop Trait**: Automatic cleanup
- **Observability-First**: Built-in monitoring

## License

Part of the AgentOS Kernel - Production-grade userspace process orchestrator.
