/*!
 * Platform-specific SIMD Detection
 * Detects available SIMD instruction sets
 */

/// SIMD capabilities available on the platform
#[derive(Debug, Clone)]
pub struct SimdCapabilities {
    /// SSE2 support (x86/x86_64)
    pub sse2: bool,
    /// SSE4.2 support (x86/x86_64)
    pub sse4_2: bool,
    /// AVX support (x86/x86_64)
    pub avx: bool,
    /// AVX2 support (x86/x86_64)
    pub avx2: bool,
    /// AVX-512 support (x86/x86_64)
    pub avx512: bool,
    /// NEON support (ARM)
    pub neon: bool,
}

/// Detect available SIMD instruction sets
pub fn detect_simd_support() -> SimdCapabilities {
    #[cfg(target_arch = "x86_64")]
    {
        SimdCapabilities {
            sse2: is_x86_feature_detected!("sse2"),
            sse4_2: is_x86_feature_detected!("sse4.2"),
            avx: is_x86_feature_detected!("avx"),
            avx2: is_x86_feature_detected!("avx2"),
            avx512: is_x86_feature_detected!("avx512f"),
            neon: false,
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        SimdCapabilities {
            sse2: false,
            sse4_2: false,
            avx: false,
            avx2: false,
            avx512: false,
            neon: std::arch::is_aarch64_feature_detected!("neon"),
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        SimdCapabilities {
            sse2: false,
            sse4_2: false,
            avx: false,
            avx2: false,
            avx512: false,
            neon: false,
        }
    }
}

