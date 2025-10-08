/*!
 * Synchronization Primitives
 *
 * High-performance synchronization primitives optimized for different use cases:
 * - Wait/notify primitives (futex, condvar, spinwait)
 * - Lock-free data structures (RCU, seqlock, striped maps)
 * - Adaptive synchronization (adaptive locks, flat combining)
 *
 * # Architecture
 *
 * This module provides a unified `WaitQueue` abstraction that can wait on
 * arbitrary keys (like sequence numbers) with multiple waiting strategies.
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
 * - **Ring buffers**: Wait for specific sequence numbers
 * - **Completion queues**: Block until operation completes
 * - **IPC synchronization**: Coordinate between processes
 * - **High-contention counters**: Flat combining for reduced cache line transfers
 * - **Read-heavy workloads**: RCU for lock-free reads
 */

// Wait/notify primitives
mod condvar;
mod config;
mod futex;
mod spinwait;
mod traits;
mod wait;

// Advanced synchronization primitives
mod adaptive;
mod flat_combining;
mod rcu;
mod seqlock_stats;
mod shard_manager;
mod striped;

// Wait/notify exports
pub use config::{StrategyType, SyncConfig};
pub use traits::{WaitStrategy, WakeResult};
pub use wait::{WaitError, WaitQueue, WaitResult};

// Re-export specific strategies for advanced users
pub use condvar::CondvarWait;
pub use futex::FutexWait;
pub use spinwait::SpinWait;

// Advanced synchronization exports
pub use adaptive::AdaptiveLock;
pub use flat_combining::FlatCombiningCounter;
pub use rcu::RcuCell;
pub use seqlock_stats::SeqlockStats;
pub use shard_manager::{ShardManager, WorkloadProfile};
pub use striped::StripedMap;
