/*!
 * CPU Feature Detection
 * Runtime detection of SIMD instruction sets
 */

/// SIMD capabilities available on the platform
#[derive(Debug, Clone, Copy)]
pub struct SimdCapabilities {
    /// SSE2 support (x86/x86_64) - 128-bit
    pub sse2: bool,
    /// SSE4.2 support (x86/x86_64) - 128-bit with string ops
    pub sse4_2: bool,
    /// AVX support (x86/x86_64) - 256-bit
    pub avx: bool,
    /// AVX2 support (x86/x86_64) - 256-bit integer ops
    pub avx2: bool,
    /// AVX-512F foundation (x86/x86_64) - 512-bit
    pub avx512f: bool,
    /// AVX-512BW byte/word ops (x86/x86_64)
    pub avx512bw: bool,
    /// AVX-512DQ dword/qword ops (x86/x86_64)
    pub avx512dq: bool,
    /// AVX-512VL vector length extensions (x86/x86_64)
    pub avx512vl: bool,
    /// NEON support (ARM) - 128-bit
    pub neon: bool,
}

impl SimdCapabilities {
    /// Check if AVX-512 is fully supported (F + BW + DQ + VL)
    #[inline]
    pub const fn has_avx512_full(&self) -> bool {
        self.avx512f && self.avx512bw && self.avx512dq && self.avx512vl
    }

    /// Get maximum vector width in bytes
    #[inline]
    pub const fn max_vector_bytes(&self) -> usize {
        if self.has_avx512_full() {
            64
        } else if self.avx2 || self.avx {
            32
        } else if self.sse2 {
            16
        } else if self.neon {
            16
        } else {
            8
        }
    }

    /// Get optimal alignment for SIMD operations
    #[inline]
    pub const fn optimal_alignment(&self) -> usize {
        self.max_vector_bytes()
    }
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
            avx512f: is_x86_feature_detected!("avx512f"),
            avx512bw: is_x86_feature_detected!("avx512bw"),
            avx512dq: is_x86_feature_detected!("avx512dq"),
            avx512vl: is_x86_feature_detected!("avx512vl"),
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
            avx512f: false,
            avx512bw: false,
            avx512dq: false,
            avx512vl: false,
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
            avx512f: false,
            avx512bw: false,
            avx512dq: false,
            avx512vl: false,
            neon: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_methods() {
        let caps = detect_simd_support();

        // Max vector bytes should be reasonable
        let max = caps.max_vector_bytes();
        assert!(max == 8 || max == 16 || max == 32 || max == 64);

        // Alignment should match vector size
        assert_eq!(caps.optimal_alignment(), max);

        // If AVX-512 full is supported, all components must be present
        if caps.has_avx512_full() {
            assert!(caps.avx512f);
            assert!(caps.avx512bw);
            assert!(caps.avx512dq);
            assert!(caps.avx512vl);
        }
    }
}

