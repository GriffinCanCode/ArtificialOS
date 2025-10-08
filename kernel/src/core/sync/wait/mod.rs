/*!
 * Wait/Notify Primitives
 *
 * High-performance wait/notify synchronization primitives with multiple strategies:
 * - Futex-based (Linux, fastest)
 * - Condvar-based (cross-platform, reliable)
 * - Spinwait-based (low-latency, high-CPU)
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
 */

mod condvar;
mod config;
mod futex;
mod spinwait;
mod traits;
mod wait;

// Re-export public API
pub use config::{StrategyType, SyncConfig};
pub use traits::{WaitStrategy, WakeResult};
pub use wait::{WaitError, WaitQueue, WaitResult};

// Re-export specific strategies for advanced users
pub use condvar::CondvarWait;
pub use futex::FutexWait;
pub use spinwait::SpinWait;
