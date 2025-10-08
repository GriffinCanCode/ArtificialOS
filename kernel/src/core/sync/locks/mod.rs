/*!
 * Lock-Based Synchronization Primitives
 *
 * Advanced lock-based primitives that reduce contention and improve performance:
 * - Adaptive locks (atomic vs mutex based on data size)
 * - Striped locks (reduce contention via partitioning)
 */

mod adaptive;
mod striped;

// Re-export public API
pub use adaptive::AdaptiveLock;
pub use striped::StripedMap;
