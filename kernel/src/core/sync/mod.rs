/*!
 * Synchronization Primitives
 *
 * High-performance wait/notify primitives optimized for different use cases:
 * - Futex-based (Linux) for minimal overhead
 * - Condvar-based (cross-platform) for reliability
 * - Adaptive spinwait for low-latency scenarios
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
 */

mod traits;
mod wait;
mod futex;
mod condvar;
mod spinwait;
mod config;

pub use traits::{WaitStrategy, WakeResult};
pub use wait::{WaitQueue, WaitResult, WaitError};
pub use config::{SyncConfig, StrategyType};

// Re-export specific strategies for advanced users
pub use futex::FutexWait;
pub use condvar::CondvarWait;
pub use spinwait::SpinWait;
