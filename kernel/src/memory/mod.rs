/*!
 * Memory Module
 * Memory management and allocation
 */

pub mod gc;
pub mod manager;
pub mod simd; // SIMD-accelerated memory operations
pub mod traits;
pub mod types;

// Re-export for convenience
pub use gc::{GcStats, GcStrategy, GlobalGarbageCollector};
pub use manager::MemoryManager;
pub use simd::{capabilities as simd_capabilities, init_simd, simd_memcmp, simd_memcpy, simd_memmove, simd_memset, SimdCapabilities};
pub use traits::*;
pub use types::*;
