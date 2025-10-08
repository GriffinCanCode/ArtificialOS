/*!
 * Memory Module
 * Memory management and allocation
 *
 * Note: SIMD operations have been moved to `crate::core::simd`
 */

pub mod gc;
pub mod manager;
pub mod traits;
pub mod types;

// Re-export for convenience
pub use gc::{GcStats, GcStrategy, GlobalGarbageCollector};
pub use manager::{MemoryGuardExt, MemoryManager};
pub use traits::*;
pub use types::*;
