/*!
 * Lock-Free Synchronization Primitives
 *
 * High-performance lock-free data structures for read-heavy workloads:
 * - RCU (Read-Copy-Update) for zero-contention reads
 * - Seqlock for lock-free statistics
 * - Flat combining for reduced cache line transfers
 */

mod flat_combining;
mod rcu;
mod seqlock_stats;

// Re-export public API
pub use flat_combining::FlatCombiningCounter;
pub use rcu::RcuCell;
pub use seqlock_stats::SeqlockStats;

