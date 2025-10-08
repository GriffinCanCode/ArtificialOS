# Async Traits Migration Guide

## Overview

This guide documents the migration to native async traits (stabilized in Rust 1.75) from the old sync-based approach with `tokio::spawn_blocking`.

## What Changed

### 1. Core Traits Now Async

All syscall traits in `kernel/src/syscalls/traits.rs` now use native async:

```rust
// OLD (sync)
pub trait FileSystemSyscalls: Send + Sync {
    fn read_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;
}

// NEW (async)
pub trait FileSystemSyscalls: Send + Sync {
    async fn read_file(&self, pid: Pid, path: &PathBuf) -> SyscallResult;
}
```

### 2. Intelligent Dual-Mode Execution

New `AsyncSyscallExecutor` automatically chooses optimal execution:

- **Fast-path**: Synchronous for operations < 100ns (memory stats, process queries)
- **Slow-path**: Async for blocking operations (file I/O, network, IPC, sleep)

Classification is compile-time via `Syscall::classify()`.

### 3. New Components

- `classification.rs`: Compile-time syscall classification
- `async_executor.rs`: Intelligent sync/async dispatcher
- `async_traits.rs`: Comprehensive async trait definitions

## Performance Impact

### Before (Sync + spawn_blocking)

Every syscall, even fast ones, paid async overhead:

```
┌─────────┐
│ Syscall │
└────┬────┘
     │
     ├──> spawn_blocking (always)
     │    ~1-10μs overhead
     │
     └──> Execute
```

### After (Dual-Mode)

Fast operations stay fast, blocking operations benefit from true async:

```
┌─────────┐
│ Syscall │
└────┬────┘
     │
     ├─── classify() ───┐
     │                  │
 ┌───▼────┐      ┌─────▼────┐
 │  Fast  │      │ Blocking │
 └───┬────┘      └─────┬────┘
     │                 │
     │ Direct          │ Async
     │ < 100ns         │ ~1-10μs
     │                 │
     └────┬────────────┘
          │
     ┌────▼────┐
     │ Result  │
     └─────────┘
```

## Migration Steps

### For New Code

Use the async executor directly:

```rust
use ai_os_kernel::syscalls::AsyncSyscallExecutor;

let executor = AsyncSyscallExecutor::new(sync_executor);

// Automatic fast/slow path selection
let result = executor.execute(pid, syscall).await;
```

### For Existing Code

The old `SyscallExecutorWithIpc` remains for backward compatibility.
Migrate incrementally:

1. Update function signatures to async
2. Add `.await` where needed
3. Replace sync executor with async executor

### Example Migration

```rust
// OLD
impl MyHandler {
    fn handle(&self, pid: Pid) -> SyscallResult {
        self.executor.execute(pid, syscall)
    }
}

// NEW
impl MyHandler {
    async fn handle(&self, pid: Pid) -> SyscallResult {
        self.executor.execute(pid, syscall).await
    }
}
```

## Classification Details

### Fast Syscalls (Synchronous)

Operations that complete in < 100ns and never block:

- Memory stats (DashMap lookup)
- Process state queries (cached data)
- System info (cached or simple calculations)
- File descriptor operations (in-memory registry)
- IPC stats (in-memory counter reads)

### Blocking Syscalls (Async)

Operations that involve I/O or can block:

- File operations (read, write, stat) - kernel I/O
- Network operations (socket, send, recv) - network latency
- IPC operations (pipe, shm, queue) - inter-process coordination
- Sleep operations - time-based blocking
- Process spawn - subprocess creation

## Advanced Features

### Batch Execution

Execute multiple syscalls concurrently:

```rust
let results = executor.execute_batch(pid, vec![
    Syscall::ReadFile { path: "file1.txt".into() },
    Syscall::ReadFile { path: "file2.txt".into() },
    Syscall::ReadFile { path: "file3.txt".into() },
]).await;
```

### Pipeline Execution

Chain syscalls with automatic error handling:

```rust
let result = executor.execute_pipeline(pid, vec![
    Syscall::ReadFile { path: "input.txt".into() },
    // ... processing ...
    Syscall::WriteFile { path: "output.txt".into(), data: processed },
]).await;
```

## Future Enhancements

### Phase 1 (Current)
- ✅ Native async traits
- ✅ Intelligent classification
- ✅ Dual-mode executor
- ⏳ spawn_blocking for blocking ops

### Phase 2 (Q2 2025)
- True async I/O with tokio::fs
- Async IPC with tokio channels
- Zero-copy async operations

### Phase 3 (Q3 2025)
- io_uring integration for Linux
- High-performance async I/O
- SIMD-accelerated async operations

## Compatibility

- **Rust version**: 1.75+ (native async traits)
- **Tokio version**: 1.35+
- **Backward compatibility**: Old sync API remains available

## Testing

Run async syscall tests:

```bash
cargo test --test syscalls -- async
```

Benchmark sync vs async:

```bash
cargo bench --bench syscall_async
```

## See Also

- [Code Standards 2025](CODE_STANDARDS_2025.md)
- [Architecture](ARCHITECTURE.md)
- [Graceful-with-Fallback Pattern](GRACEFUL_WITH_FALLBACK_PATTERN.md)

