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
pub use simd::{
    // Memory operations
    simd_memcmp, simd_memcpy, simd_memmove, simd_memset,
    // CPU detection
    capabilities as simd_capabilities, init_simd, SimdCapabilities,
    // Search operations
    contains_byte, count_byte, find_byte, rfind_byte,
    // Math operations
    avg_u64, max_u64, min_u64, sum_u32, sum_u64,
    // Text operations
    ascii_to_lower, ascii_to_upper, count_whitespace, is_ascii, trim, trim_end, trim_start,
};
pub use traits::*;
pub use types::*;
