/*!
 * SIMD-Accelerated Operations
 *
 * High-performance operations using AVX-512/AVX2/SSE2 SIMD instructions.
 * Provides memory, search, math, and text operations with automatic CPU feature detection.
 */

mod calc;
mod find;
mod operations;
mod platform;
mod text;

// Memory operations
pub use operations::{simd_memcmp, simd_memcpy, simd_memmove, simd_memset};

// CPU detection
pub use platform::{detect_simd_support, SimdCapabilities};

// Search operations
pub use find::{contains_byte, count_byte, find_byte, rfind_byte};

// Math operations
pub use calc::{avg_u64, max_u64, min_u64, sum_u32, sum_u64};

// Text operations
pub use text::{
    ascii_to_lower, ascii_to_upper, count_whitespace, is_ascii, trim, trim_end, trim_start,
};

use std::sync::OnceLock;

/// Global SIMD capabilities
static SIMD_CAPS: OnceLock<SimdCapabilities> = OnceLock::new();

/// Initialize SIMD capabilities detection
pub fn init_simd() -> &'static SimdCapabilities {
    SIMD_CAPS.get_or_init(|| {
        let caps = platform::detect_simd_support();
        tracing::info!(
            sse2 = caps.sse2,
            sse4_2 = caps.sse4_2,
            avx = caps.avx,
            avx2 = caps.avx2,
            avx512f = caps.avx512f,
            avx512bw = caps.avx512bw,
            avx512dq = caps.avx512dq,
            avx512vl = caps.avx512vl,
            avx512_full = caps.has_avx512_full(),
            neon = caps.neon,
            max_vector_bytes = caps.max_vector_bytes(),
            "SIMD capabilities detected"
        );
        caps
    })
}

/// Get SIMD capabilities
pub fn capabilities() -> &'static SimdCapabilities {
    SIMD_CAPS.get_or_init(platform::detect_simd_support)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_detection() {
        let caps = capabilities();
        // At least SSE2 should be available on x86_64, or NEON on aarch64
        #[cfg(target_arch = "x86_64")]
        assert!(caps.sse2);

        #[cfg(target_arch = "aarch64")]
        assert!(caps.neon);
    }
}
