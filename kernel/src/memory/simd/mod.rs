/*!
 * SIMD-Accelerated Memory Operations
 *
 * High-performance memory operations using SIMD instructions for bulk data transfer.
 * Provides optimized implementations of memcpy, memmove, memcmp, and memset.
 */

mod operations;
mod platform;

pub use operations::{simd_memcpy, simd_memmove, simd_memcmp, simd_memset};
pub use platform::{detect_simd_support, SimdCapabilities};

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
            avx512 = caps.avx512,
            neon = caps.neon,
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

