/*!
 * Synchronization Primitives
 *
 * High-performance synchronization primitives optimized for different use cases:
 * - Wait/notify primitives (futex, condvar, spinwait)
 * - Lock-free data structures (RCU, seqlock, flat combining)
 * - Lock-based primitives (adaptive locks, striped maps)
 * - Configuration management (shard manager, workload profiling)
 *
 * # Architecture
 *
 * This module is organized by domain:
 *
 * - `wait/`: Wait/notify primitives for blocking synchronization
 * - `lockfree/`: Lock-free data structures for read-heavy workloads
 * - `locks/`: Advanced lock-based primitives with contention reduction
 * - `management/`: Configuration and management utilities
 *
 * # Performance
 *
 * - Zero-cost abstractions via monomorphization
 * - Lock-free fast paths for try_wait operations
 * - Platform-optimized implementations (futex on Linux)
 * - Cache-line aligned to prevent false sharing
 *
 * # Use Cases
 *
 * - Ring buffers: Wait for specific sequence numbers
 * - Completion queues: Block until operation completes
 * - IPC synchronization: Coordinate between processes
 * - High-contention counters: Flat combining for reduced cache line transfers
 * - Read-heavy workloads: RCU for lock-free reads
 */

// Domain modules
pub mod lockfree;
pub mod locks;
pub mod management;
pub mod wait;

// Re-export commonly used items at top level for convenience

// Wait/notify primitives
pub use wait::{
    CondvarWait, FutexWait, SpinWait, StrategyType, SyncConfig, WaitError, WaitQueue, WaitResult,
    WaitStrategy, WakeResult,
};

// Lock-free primitives
pub use lockfree::{FlatCombiningCounter, RcuCell, SeqlockStats};

// Lock-based primitives
pub use locks::{AdaptiveLock, StripedMap};

// Configuration management
pub use management::{ShardManager, WorkloadProfile};
