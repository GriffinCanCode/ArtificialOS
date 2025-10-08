/*!
 * Optimization Utilities
 *
 * Low-level optimization primitives for performance-critical code:
 * - Branch prediction hints (likely/unlikely)
 * - CPU hints for spinloops and optimization barriers
 * - Prefetch hints for cache-aware algorithms
 * - SIMD search operations
 *
 * # Performance
 *
 * - Branch hints: 5-10% speedup on hot paths with predictable branches
 * - Spin loop hints: Reduces power consumption, improves hyper-threading
 * - Prefetch: 20-30% speedup for pointer-chasing algorithms
 * - SIMD search: 4-8x faster than scalar for large datasets
 *
 * # Use Cases
 *
 * - **Branch hints**: Hot paths with predictable outcomes (null checks, validation)
 * - **CPU hints**: Spinlocks, busy-wait loops
 * - **Prefetch**: Hash table probing, tree traversal
 * - **SIMD search**: String searching, hash matching in large tables
 */

mod hints;
mod likely;
mod prefetch;
mod simd_search;

// Re-export all optimization utilities
pub use hints::*;
pub use likely::{likely, unlikely};
pub use prefetch::{prefetch_read, prefetch_write, PrefetchExt};
pub use simd_search::{find_hash_simd, path_starts_with_any};

