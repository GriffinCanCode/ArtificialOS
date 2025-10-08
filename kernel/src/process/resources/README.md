# Process Resource Cleanup System

Comprehensive per-process resource tracking and cleanup orchestration to prevent resource leaks during process termination.

## Problem Statement

Previously, process termination only cleaned up 4 resource types:
- OS process handles
- Memory allocations
- IPC queues
- Scheduler entries

This left behind critical resources that would leak under high process churn in production:
- **File descriptors** ✓ *Now cleaned*
- **Network sockets** ✓ *Now cleaned*
- **Memory mappings (mmap)** ✓ *Now cleaned*
- **Zero-copy rings** ✓ *Now cleaned*
- **io_uring rings** ✓ *Now cleaned*
- **Signal callbacks** ✓ *Now cleaned*
- **Async tasks** ✓ *Now cleaned*

## Architecture

### Core Components

#### 1. `ResourceCleanup` Trait (`mod.rs`)

Core trait that all resource types implement:

```rust
pub trait ResourceCleanup: Send + Sync {
    /// Cleanup all resources owned by a process
    fn cleanup(&self, pid: Pid) -> CleanupStats;
    
    /// Resource type name for logging
    fn resource_type(&self) -> &'static str;
    
    /// Check if process has any resources
    fn has_resources(&self, pid: Pid) -> bool;
}
```

#### 2. `ResourceOrchestrator` (`mod.rs`)

Coordinates cleanup across all resource types in well-defined order (LIFO) to prevent deadlocks:

```rust
let orchestrator = ResourceOrchestrator::new()
    .register(SocketResource::new(socket_manager))
    .register(SignalResource::new(signal_manager))
    .register(RingResource::new()
        .with_zerocopy(zerocopy_ipc)
        .with_iouring(iouring_manager))
    .register(TaskResource::new(async_task_manager))
    .register(MappingResource::new(mmap_manager));

let result = orchestrator.cleanup_process(pid);
```

Resources are cleaned in **reverse registration order** (LIFO) to handle dependencies properly.

#### 3. Resource Wrappers

Each resource type has a dedicated wrapper module:

- **`sockets.rs`** - Network socket cleanup via `SocketManager`
- **`signals.rs`** - Signal handler cleanup via `SignalManager`
- **`rings.rs`** - Zero-copy and io_uring ring cleanup
- **`tasks.rs`** - Async task cancellation and cleanup
- **`mappings.rs`** - Memory mapping (mmap) cleanup

### Per-Process Tracking

Each resource manager now maintains per-process tracking:

```rust
// SocketManager example
pub struct SocketManager {
    tcp_listeners: Arc<DashMap<u32, TcpListener>>,
    tcp_streams: Arc<DashMap<u32, TcpStream>>,
    udp_sockets: Arc<DashMap<u32, UdpSocket>>,
    process_sockets: Arc<DashMap<Pid, Vec<u32>>>,  // NEW: Per-process tracking
}

impl SocketManager {
    fn track_socket(&self, pid: Pid, sockfd: u32);
    fn untrack_socket(&self, pid: Pid, sockfd: u32);
    pub fn cleanup_process_sockets(&self, pid: Pid) -> usize;
    pub fn has_process_sockets(&self, pid: Pid) -> bool;
}
```

Similar tracking added to:
- `SignalManager` - tracks handlers and pending signals
- `AsyncTaskManager` - tracks task IDs per process
- `MmapManager` - already tracked via `owner_pid` field
- `ZeroCopyIpc` - rings stored with PID as key
- `IoUringManager` - rings stored with PID as key

## Integration

### ProcessManager Integration

The `ProcessManager` now includes a `ResourceOrchestrator`:

```rust
pub struct ProcessManager {
    // ... existing fields ...
    resource_orchestrator: Option<ResourceOrchestrator>,
}

pub fn terminate_process(&self, pid: Pid) -> bool {
    // Core cleanup (backwards compatible)
    cleanup::cleanup_os_process(&process, pid, ...);
    cleanup::cleanup_memory(pid, ...);
    cleanup::cleanup_ipc(pid, ...);
    cleanup::cleanup_scheduler(pid, ...);
    cleanup::cleanup_preemption(pid, ...);
    cleanup::cleanup_file_descriptors(pid, ...);
    
    // Comprehensive resource cleanup (new)
    cleanup::cleanup_comprehensive(pid, &self.resource_orchestrator);
}
```

### Builder Pattern

Use `ProcessManagerBuilder` to configure comprehensive cleanup:

```rust
let orchestrator = ResourceOrchestrator::new()
    .register(SocketResource::new(socket_manager))
    .register(SignalResource::new(signal_manager))
    // ... register other resources ...
    ;

let process_manager = ProcessManager::builder()
    .with_memory_manager(memory_manager)
    .with_executor()
    .with_scheduler(SchedulingPolicy::RoundRobin)
    .with_resource_orchestrator(orchestrator)  // NEW
    .build();
```

## Cleanup Order

Resources are cleaned in **LIFO order** (reverse of registration):

1. **Memory mappings** - Unmap files, sync shared mappings
2. **Async tasks** - Cancel running tasks, cleanup completed tasks
3. **Rings (io_uring, zero-copy)** - Deallocate ring buffers
4. **Signal handlers** - Remove handlers, clear pending signals
5. **Network sockets** - Close all TCP/UDP sockets

This order ensures:
- Tasks are cancelled before their I/O resources are freed
- Rings are freed before the underlying memory
- Signals are cleaned before process-level state

## Statistics

Each cleanup operation returns detailed statistics:

```rust
pub struct CleanupStats {
    pub resources_freed: usize,
    pub bytes_freed: usize,
    pub errors_encountered: usize,
}

pub struct CleanupResult {
    pub pid: Pid,
    pub stats: CleanupStats,
    pub errors: Vec<String>,
}
```

## Testing

Comprehensive tests ensure cleanup works correctly:

```rust
#[test]
fn test_socket_cleanup() {
    let socket_manager = SocketManager::new();
    // Create sockets for process 1
    socket_manager.track_socket(1, 1000);
    socket_manager.track_socket(1, 1001);
    
    // Cleanup
    let count = socket_manager.cleanup_process_sockets(1);
    assert_eq!(count, 2);
    assert!(!socket_manager.has_process_sockets(1));
}
```

## Performance Considerations

- **Lock-free tracking**: Uses `DashMap` for concurrent access
- **Batch cleanup**: Collects resource IDs first, then cleanups
- **No blocking**: Cleanup operations are non-blocking
- **Minimal overhead**: Per-process tracking adds ~8 bytes per resource

## Future Enhancements

- Add cleanup hooks for custom resource types
- Implement priority-based cleanup ordering
- Add metrics for resource leak detection
- Support graceful shutdown with configurable timeouts

## References

- Original issue: Process termination incomplete cleanup
- Related: `ProcessManager`, `ProcessCleanup`, Resource managers
- Pattern: RAII (Resource Acquisition Is Initialization)
