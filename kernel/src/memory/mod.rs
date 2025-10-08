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
pub use manager::{MemoryGuardExt, MemoryManager};
pub use simd::{
    // Text operations
    ascii_to_lower,
    ascii_to_upper,
    // Math operations
    avg_u64,
    // CPU detection
    capabilities as simd_capabilities,
    // Search operations
    contains_byte,
    count_byte,
    count_whitespace,
    find_byte,
    init_simd,
    is_ascii,
    max_u64,
    min_u64,
    rfind_byte,
    // Memory operations
    simd_memcmp,
    simd_memcpy,
    simd_memmove,
    simd_memset,
    sum_u32,
    sum_u64,
    trim,
    trim_end,
    trim_start,
    SimdCapabilities,
};
pub use traits::*;
pub use types::*;
