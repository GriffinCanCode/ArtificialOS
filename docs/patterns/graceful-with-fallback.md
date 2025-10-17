# Graceful-with-Fallback Shutdown Pattern

## Overview

The **Graceful-with-Fallback** pattern is a solution to the async Drop problem in Rust, providing safe, ergonomic cleanup for long-lived background tasks. This pattern is implemented in our kernel for components that spawn autonomous async tasks.

## The Problem

Rust's `Drop` trait cannot be async, but many background tasks require async cleanup:

```rust
impl Drop for MyManager {
    fn drop(&mut self) {
        // ❌ Can't do this - Drop is not async!
        // self.background_task.await;
    }
}
```

**Traditional Solutions:**
1. **Leak the task** - Task continues running after drop (resource leak)
2. **Force immediate abort** - `handle.abort()` in Drop (not graceful)
3. **Require manual cleanup** - Document "must call shutdown()" (easy to forget)
4. **Use separate shutdown coordinator** - Complex, requires external orchestration

All have significant drawbacks: resource leaks, ungraceful termination, poor ergonomics, or complexity.

## The Solution: Multi-Layered Shutdown

Our pattern combines **two complementary cleanup paths**:

### Path 1: Graceful Shutdown (Preferred)
```rust
// Explicit, awaitable shutdown
manager.shutdown().await;
```

**What happens:**
1. Sets atomic flag to mark graceful shutdown initiated
2. Sends shutdown signal to background task via channel
3. Awaits task handle for clean completion
4. Logs success confirmation

**Benefits:**
- Clean resource cleanup
- Controlled task termination
- Full async capabilities
- Predictable timing

### Path 2: Fallback Abort (Safety Net)
```rust
// Automatic on Drop if shutdown wasn't called
drop(manager);
```

**What happens:**
1. Checks atomic flag - was graceful shutdown called?
2. If not, immediately aborts task via `JoinHandle::abort()`
3. Logs warning to alert developer
4. Non-blocking, prevents resource leak

**Benefits:**
- Fail-safe: task always stops
- Non-blocking: safe in Drop
- Clear feedback: warning logs
- Prevents resource leaks

## Implementation Pattern

### Core Components

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;

struct BackgroundTaskHandle {
    handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    shutdown_initiated: Arc<AtomicBool>,
}

pub struct Manager {
    // ... other fields ...
    task_handle: Arc<Mutex<BackgroundTaskHandle>>,
}
```

### Spawn Pattern

```rust
impl Manager {
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let shutdown_initiated = Arc::new(AtomicBool::new(false));
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                tokio::select! {
                    // Shutdown signal
                    _ = &mut shutdown_rx => {
                        log::info!("Background task shutting down gracefully");
                        break;
                    }
                    
                    // Periodic work
                    _ = interval.tick() => {
                        // Do background work
                    }
                }
            }
        });
        
        Self {
            task_handle: Arc::new(Mutex::new(BackgroundTaskHandle {
                handle: Some(handle),
                shutdown_tx: Some(shutdown_tx),
                shutdown_initiated,
            })),
        }
    }
}
```

### Shutdown Method

```rust
impl Manager {
    pub async fn shutdown(&self) {
        let mut handle = self.task_handle.lock();
        
        // Check if already shut down (idempotent)
        if handle.shutdown_initiated.load(Ordering::SeqCst) {
            return;
        }
        
        // Mark as initiated
        handle.shutdown_initiated.store(true, Ordering::SeqCst);
        
        // Send shutdown signal
        if let Some(tx) = handle.shutdown_tx.take() {
            let _ = tx.send(());
        }
        
        // Wait for completion
        if let Some(h) = handle.handle.take() {
            match h.await {
                Ok(_) => log::info!("Background task shutdown complete"),
                Err(e) => log::warn!("Background task error: {}", e),
            }
        }
    }
}
```

### Drop Implementation

```rust
impl Drop for BackgroundTaskHandle {
    fn drop(&mut self) {
        // Check if graceful shutdown was called
        if self.shutdown_initiated.load(Ordering::SeqCst) {
            // Graceful path was used - nothing to do
            return;
        }
        
        // Fallback: abort the task
        if let Some(handle) = self.handle.take() {
            log::warn!(
                "Background task dropped without shutdown() - aborting. \
                 Use `manager.shutdown().await` for graceful cleanup."
            );
            handle.abort();
        }
    }
}
```

## When to Use This Pattern

### ✅ **Perfect Fit - Apply Pattern**

Use this pattern when **ALL** of these conditions are true:

1. **Long-lived background task** - Runs for seconds, minutes, or indefinitely
2. **Infinite loop or long iterations** - Not self-terminating
3. **Droppable at any time** - Parent struct can be dropped mid-execution
4. **Needs graceful cleanup** - Logging, state finalization, or resource cleanup
5. **Reusable component** - Part of a library or framework, not one-off app code

**Examples in our codebase:**
- ✅ `SchedulerTask` - Preemptive scheduling loop (100μs intervals, runs forever)
- ✅ `AsyncTaskManager` - Cleanup task (5-minute intervals, runs forever)

### ⚠️ **Consider Carefully**

Think twice about applying if:

- Task has explicit shutdown signals already (gRPC server, HTTP server)
- Application-level coordination exists (main.rs shutdown sequence)
- Task is short-lived (runs once and completes)
- Component is not reusable (one-off application code)

### ❌ **Don't Apply**

Skip this pattern for:

1. **Self-terminating tasks** - Tasks that complete naturally
2. **Short-lived operations** - Individual syscalls, quick computations
3. **Managed lifecycle** - Tasks with external lifecycle coordinators
4. **Data structures only** - Components with no async tasks (managers that are just state)

## Design Rationale

### Why Atomic Bool Instead of Arc<RwLock<bool>>?

**Performance**: Atomic operations are lock-free and extremely fast (single instruction)
```rust
// Atomic: ~1-2 CPU cycles
shutdown_initiated.load(Ordering::SeqCst)

// RwLock: ~50-100+ cycles (acquire lock, check, release)
*shutdown_initiated.read()
```

**Simplicity**: No deadlock potential, no lock contention
**Sufficient**: Boolean flag doesn't need RwLock's capabilities

### Why Abort Instead of Channel Signal in Drop?

**Non-blocking requirement**: Drop cannot await
```rust
// ❌ Can't do this in Drop
let _ = shutdown_tx.send(());
handle.await; // Drop is not async!

// ✅ Can do this in Drop
handle.abort(); // Immediate, non-blocking
```

**Safety**: Abort is instant and guaranteed to stop the task

### Why Both Graceful and Abort Paths?

**Ergonomics vs Safety tradeoff**:
- Graceful path: Best developer experience, clean shutdown
- Abort path: Safety net, prevents leaks when graceful forgotten

**Real-world usage**:
- Production: Graceful shutdown in main.rs shutdown sequence
- Tests: Often forget explicit shutdown, abort prevents hangs
- Libraries: Users might not read docs, abort protects them

## Performance Characteristics

### Memory Overhead
- **1 atomic bool** (1 byte, typically padded to 8)
- **1 oneshot channel** (~32 bytes)
- **Total per component**: ~40 bytes

### Graceful Shutdown Timing
- **Best case**: 0-1ms (task immediately responsive to signal)
- **Typical**: 0-10ms (waits for current interval tick)
- **Worst case**: Up to interval duration (e.g., 5 min for cleanup task)

### Abort Shutdown Timing
- **Always**: <1ms (immediate abort, no waiting)

### Runtime Overhead
- **During normal operation**: Zero (flag only checked on shutdown/drop)
- **On graceful shutdown**: Single atomic load + channel send + await
- **On abort**: Single atomic load + abort call

## Testing Strategy

### Test Coverage

1. **Graceful shutdown works**
   ```rust
   #[tokio::test]
   async fn test_graceful_shutdown() {
       let manager = Manager::new();
       manager.shutdown().await; // Should complete cleanly
   }
   ```

2. **Shutdown is idempotent**
   ```rust
   #[tokio::test]
   async fn test_shutdown_twice() {
       let manager = Manager::new();
       manager.shutdown().await;
       manager.shutdown().await; // Should be no-op
   }
   ```

3. **Drop without shutdown aborts**
   ```rust
   #[tokio::test]
   async fn test_drop_aborts() {
       {
           let manager = Manager::new();
           // Don't call shutdown
       } // Should log warning and abort
       tokio::time::sleep(Duration::from_millis(10)).await;
   }
   ```

4. **Works with Clone (if applicable)**
   ```rust
   #[tokio::test]
   async fn test_shutdown_with_clones() {
       let manager = Manager::new();
       let clone = manager.clone();
       manager.shutdown().await; // Shuts down shared task
       clone.shutdown().await; // Should be no-op
   }
   ```

## Common Pitfalls

### ❌ Pitfall 1: Forgetting to Call Shutdown in Tests

```rust
#[tokio::test]
async fn test_something() {
    let manager = Manager::new();
    // Do test...
    // ❌ Forgot to call shutdown - will trigger abort warning
}
```

**Fix**: Always shutdown in tests
```rust
#[tokio::test]
async fn test_something() {
    let manager = Manager::new();
    // Do test...
    manager.shutdown().await; // ✅ Graceful cleanup
}
```

### ❌ Pitfall 2: Calling Shutdown on Every Clone

For Clone managers, only call shutdown once:
```rust
let manager1 = Manager::new();
let manager2 = manager1.clone();

// ✅ Good: Shutdown once
manager1.shutdown().await;

// ⚠️ Unnecessary but harmless: Shutdown is idempotent
manager2.shutdown().await; // No-op, already shut down
```

### ❌ Pitfall 3: Not Handling Task Errors in Loop

Ensure your background task handles errors gracefully:
```rust
loop {
    tokio::select! {
        _ = &mut shutdown_rx => break,
        _ = interval.tick() => {
            // ❌ Bad: Panic will abort task
            do_work().unwrap();
            
            // ✅ Good: Handle errors
            if let Err(e) = do_work() {
                log::error!("Background work failed: {}", e);
            }
        }
    }
}
```

## Implementation Checklist

When implementing this pattern:

- [ ] Add `AtomicBool` for shutdown_initiated flag
- [ ] Create oneshot channel for shutdown signal
- [ ] Spawn background task with `tokio::select!` on shutdown signal
- [ ] Store JoinHandle and shutdown_tx in struct
- [ ] Implement `shutdown()` method that:
  - [ ] Checks flag (idempotent)
  - [ ] Sets flag
  - [ ] Sends signal
  - [ ] Awaits handle
- [ ] Implement Drop that:
  - [ ] Checks flag
  - [ ] Aborts if not set
  - [ ] Logs warning
- [ ] Add tests for:
  - [ ] Graceful shutdown
  - [ ] Idempotency
  - [ ] Drop abort
  - [ ] Clone behavior (if Clone)
- [ ] Document shutdown requirement in struct docs
- [ ] Update integration/shutdown code to call shutdown()

## Examples in Codebase

### SchedulerTask (Canonical Example)

**Location**: `kernel/src/process/scheduler_task.rs`

**Use case**: Autonomous preemptive scheduling that runs every 100μs

**Key aspects**:
- Consumes self on shutdown (not Clone)
- Very tight loop (microsecond intervals)
- Critical for system responsiveness
- Must stop cleanly on kernel shutdown

### AsyncTaskManager (Clone Variant)

**Location**: `kernel/src/api/execution/async_task.rs`

**Use case**: Background cleanup of expired async tasks every 5 minutes

**Key aspects**:
- Clone-able (shared cleanup task across all clones)
- Loose intervals (5 minutes)
- Cleanup task is shared state
- Shutdown on any clone stops task for all

## Future Extensions

### Potential Enhancements

1. **Graceful timeout**: Auto-abort if graceful shutdown takes too long
   ```rust
   tokio::time::timeout(Duration::from_secs(10), handle.await)
   ```

2. **Shutdown callback**: Allow custom cleanup logic
   ```rust
   pub async fn shutdown_with<F>(&self, cleanup: F)
   where F: FnOnce() -> Future<Output = ()>
   ```

3. **Reusable trait**: Extract pattern into reusable trait
   ```rust
   pub trait GracefulShutdown {
       async fn shutdown(&self);
   }
   ```

## Conclusion

The Graceful-with-Fallback pattern elegantly solves the async Drop problem by:

- **Preferring ergonomics**: Graceful shutdown for best practices
- **Ensuring safety**: Abort fallback prevents resource leaks
- **Providing feedback**: Clear warnings when fallback used
- **Zero cost**: Minimal overhead, only pays when needed

Apply this pattern judiciously to long-lived background tasks in reusable components. It's not a universal solution, but where it fits, it provides excellent ergonomics with strong safety guarantees.

---

**References**:
- SchedulerTask implementation: `kernel/src/process/scheduler_task.rs`
- AsyncTaskManager implementation: `kernel/src/api/execution/async_task.rs`
- Test examples: `kernel/tests/syscalls/async_task_test.rs`
