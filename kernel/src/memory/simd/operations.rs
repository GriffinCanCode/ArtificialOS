/*!
 * SIMD Memory Operations
 * High-performance memory operations using SIMD instructions
 */

use std::cmp::Ordering;

/// Threshold for using SIMD operations (bytes)
/// Below this, standard operations are faster due to setup overhead
use crate::core::limits::MEMORY_SIMD_THRESHOLD as SIMD_THRESHOLD;

/// SIMD-accelerated memcpy
/// Copies `len` bytes from `src` to `dst` using SIMD when beneficial
///
/// # Safety
/// - `src` must be valid for reads of `len` bytes
/// - `dst` must be valid for writes of `len` bytes
/// - `src` and `dst` must not overlap (use simd_memmove for overlapping regions)
pub fn simd_memcpy(dst: &mut [u8], src: &[u8]) -> usize {
    let len = dst.len().min(src.len());

    if len < SIMD_THRESHOLD {
        // For small copies, use standard memcpy
        dst[..len].copy_from_slice(&src[..len]);
        return len;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
            unsafe {
                return simd_memcpy_avx512(dst, src, len);
            }
        }
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return simd_memcpy_avx2(dst, src, len);
            }
        }
        if is_x86_feature_detected!("sse2") {
            unsafe {
                return simd_memcpy_sse2(dst, src, len);
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            unsafe {
                return simd_memcpy_neon(dst, src, len);
            }
        }
    }

    // Fallback to standard memcpy
    dst[..len].copy_from_slice(&src[..len]);
    len
}

/// SIMD-accelerated memmove
/// Copies `len` bytes from `src` to `dst`, handling overlapping regions correctly
pub fn simd_memmove(dst: &mut [u8], src: &[u8]) -> usize {
    let len = dst.len().min(src.len());

    // For overlapping regions, we need to be careful about direction
    let dst_ptr = dst.as_ptr() as usize;
    let src_ptr = src.as_ptr() as usize;

    if dst_ptr == src_ptr {
        return len; // Nothing to do
    }

    // Use safe overlapping copy
    if len < SIMD_THRESHOLD {
        // For small moves, use standard approach
        if dst_ptr < src_ptr || dst_ptr >= src_ptr + len {
            // Non-overlapping or dst before src - forward copy is safe
            dst[..len].copy_from_slice(&src[..len]);
        } else {
            // Overlapping with dst after src - need backward copy
            for i in (0..len).rev() {
                dst[i] = src[i];
            }
        }
        return len;
    }

    // For large moves, use SIMD if available and non-overlapping
    if dst_ptr < src_ptr || dst_ptr >= src_ptr + len {
        simd_memcpy(dst, src)
    } else {
        // Overlapping - use backward copy
        for i in (0..len).rev() {
            dst[i] = src[i];
        }
        len
    }
}

/// SIMD-accelerated memcmp
/// Compares `len` bytes from `a` and `b` using SIMD when beneficial
pub fn simd_memcmp(a: &[u8], b: &[u8]) -> Ordering {
    let len = a.len().min(b.len());

    if len < SIMD_THRESHOLD {
        return a[..len].cmp(&b[..len]);
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
            unsafe {
                if let Some(ord) = simd_memcmp_avx512(a, b, len) {
                    return ord;
                }
            }
        }
        if is_x86_feature_detected!("avx2") {
            unsafe {
                if let Some(ord) = simd_memcmp_avx2(a, b, len) {
                    return ord;
                }
            }
        }
        if is_x86_feature_detected!("sse2") {
            unsafe {
                if let Some(ord) = simd_memcmp_sse2(a, b, len) {
                    return ord;
                }
            }
        }
    }

    // Fallback
    a[..len].cmp(&b[..len])
}

/// SIMD-accelerated memset
/// Sets `len` bytes in `dst` to `value` using SIMD when beneficial
pub fn simd_memset(dst: &mut [u8], value: u8) -> usize {
    let len = dst.len();

    if len < SIMD_THRESHOLD {
        dst.fill(value);
        return len;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
            unsafe {
                return simd_memset_avx512(dst, value, len);
            }
        }
        if is_x86_feature_detected!("avx2") {
            unsafe {
                return simd_memset_avx2(dst, value, len);
            }
        }
        if is_x86_feature_detected!("sse2") {
            unsafe {
                return simd_memset_sse2(dst, value, len);
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            unsafe {
                return simd_memset_neon(dst, value, len);
            }
        }
    }

    // Fallback
    dst.fill(value);
    len
}

// x86_64 AVX-512 implementations
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
unsafe fn simd_memcpy_avx512(dst: &mut [u8], src: &[u8], len: usize) -> usize {
    use std::arch::x86_64::*;

    let mut offset = 0;

    // Process 64 bytes at a time with AVX-512
    while offset + 64 <= len {
        let src_ptr = src.as_ptr().add(offset) as *const __m512i;
        let dst_ptr = dst.as_mut_ptr().add(offset) as *mut __m512i;

        let data = _mm512_loadu_si512(src_ptr);
        _mm512_storeu_si512(dst_ptr, data);

        offset += 64;
    }

    // Handle remaining bytes with scalar copy
    while offset < len {
        *dst.get_unchecked_mut(offset) = *src.get_unchecked(offset);
        offset += 1;
    }

    len
}

// x86_64 AVX2 implementations
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn simd_memcpy_avx2(dst: &mut [u8], src: &[u8], len: usize) -> usize {
    use std::arch::x86_64::*;

    let mut offset = 0;

    // Process 32 bytes at a time with AVX2
    while offset + 32 <= len {
        let src_ptr = src.as_ptr().add(offset) as *const __m256i;
        let dst_ptr = dst.as_mut_ptr().add(offset) as *mut __m256i;

        let data = _mm256_loadu_si256(src_ptr);
        _mm256_storeu_si256(dst_ptr, data);

        offset += 32;
    }

    // Handle remaining bytes
    while offset < len {
        *dst.get_unchecked_mut(offset) = *src.get_unchecked(offset);
        offset += 1;
    }

    len
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn simd_memcpy_sse2(dst: &mut [u8], src: &[u8], len: usize) -> usize {
    use std::arch::x86_64::*;

    let mut offset = 0;

    // Process 16 bytes at a time with SSE2
    while offset + 16 <= len {
        let src_ptr = src.as_ptr().add(offset) as *const __m128i;
        let dst_ptr = dst.as_mut_ptr().add(offset) as *mut __m128i;

        let data = _mm_loadu_si128(src_ptr);
        _mm_storeu_si128(dst_ptr, data);

        offset += 16;
    }

    // Handle remaining bytes
    while offset < len {
        *dst.get_unchecked_mut(offset) = *src.get_unchecked(offset);
        offset += 1;
    }

    len
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
unsafe fn simd_memcmp_avx512(a: &[u8], b: &[u8], len: usize) -> Option<Ordering> {
    use std::arch::x86_64::*;

    let mut offset = 0;

    // Process 64 bytes at a time
    while offset + 64 <= len {
        let a_ptr = a.as_ptr().add(offset) as *const __m512i;
        let b_ptr = b.as_ptr().add(offset) as *const __m512i;

        let a_data = _mm512_loadu_si512(a_ptr);
        let b_data = _mm512_loadu_si512(b_ptr);

        let mask = _mm512_cmpeq_epi8_mask(a_data, b_data);

        if mask != 0xFFFFFFFFFFFFFFFF {
            // Found difference - do byte-by-byte comparison
            for i in offset..offset + 64 {
                match a[i].cmp(&b[i]) {
                    Ordering::Equal => continue,
                    other => return Some(other),
                }
            }
        }

        offset += 64;
    }

    // Handle remaining bytes
    Some(a[offset..len].cmp(&b[offset..len]))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn simd_memcmp_avx2(a: &[u8], b: &[u8], len: usize) -> Option<Ordering> {
    use std::arch::x86_64::*;

    let mut offset = 0;

    // Process 32 bytes at a time
    while offset + 32 <= len {
        let a_ptr = a.as_ptr().add(offset) as *const __m256i;
        let b_ptr = b.as_ptr().add(offset) as *const __m256i;

        let a_data = _mm256_loadu_si256(a_ptr);
        let b_data = _mm256_loadu_si256(b_ptr);

        let cmp = _mm256_cmpeq_epi8(a_data, b_data);
        let mask = _mm256_movemask_epi8(cmp);

        if mask != -1 {
            // Found difference - do byte-by-byte comparison
            for i in offset..offset + 32 {
                match a[i].cmp(&b[i]) {
                    Ordering::Equal => continue,
                    other => return Some(other),
                }
            }
        }

        offset += 32;
    }

    // Handle remaining bytes
    Some(a[offset..len].cmp(&b[offset..len]))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn simd_memcmp_sse2(a: &[u8], b: &[u8], len: usize) -> Option<Ordering> {
    use std::arch::x86_64::*;

    let mut offset = 0;

    // Process 16 bytes at a time
    while offset + 16 <= len {
        let a_ptr = a.as_ptr().add(offset) as *const __m128i;
        let b_ptr = b.as_ptr().add(offset) as *const __m128i;

        let a_data = _mm_loadu_si128(a_ptr);
        let b_data = _mm_loadu_si128(b_ptr);

        let cmp = _mm_cmpeq_epi8(a_data, b_data);
        let mask = _mm_movemask_epi8(cmp);

        if mask != 0xFFFF {
            // Found difference
            for i in offset..offset + 16 {
                match a[i].cmp(&b[i]) {
                    Ordering::Equal => continue,
                    other => return Some(other),
                }
            }
        }

        offset += 16;
    }

    // Handle remaining bytes
    Some(a[offset..len].cmp(&b[offset..len]))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
unsafe fn simd_memset_avx512(dst: &mut [u8], value: u8, len: usize) -> usize {
    use std::arch::x86_64::*;

    let pattern = _mm512_set1_epi8(value as i8);
    let mut offset = 0;

    // Process 64 bytes at a time
    while offset + 64 <= len {
        let dst_ptr = dst.as_mut_ptr().add(offset) as *mut __m512i;
        _mm512_storeu_si512(dst_ptr, pattern);
        offset += 64;
    }

    // Handle remaining bytes
    while offset < len {
        *dst.get_unchecked_mut(offset) = value;
        offset += 1;
    }

    len
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn simd_memset_avx2(dst: &mut [u8], value: u8, len: usize) -> usize {
    use std::arch::x86_64::*;

    let pattern = _mm256_set1_epi8(value as i8);
    let mut offset = 0;

    // Process 32 bytes at a time
    while offset + 32 <= len {
        let dst_ptr = dst.as_mut_ptr().add(offset) as *mut __m256i;
        _mm256_storeu_si256(dst_ptr, pattern);
        offset += 32;
    }

    // Handle remaining bytes
    while offset < len {
        *dst.get_unchecked_mut(offset) = value;
        offset += 1;
    }

    len
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn simd_memset_sse2(dst: &mut [u8], value: u8, len: usize) -> usize {
    use std::arch::x86_64::*;

    let pattern = _mm_set1_epi8(value as i8);
    let mut offset = 0;

    // Process 16 bytes at a time
    while offset + 16 <= len {
        let dst_ptr = dst.as_mut_ptr().add(offset) as *mut __m128i;
        _mm_storeu_si128(dst_ptr, pattern);
        offset += 16;
    }

    // Handle remaining bytes
    while offset < len {
        *dst.get_unchecked_mut(offset) = value;
        offset += 1;
    }

    len
}

// ARM NEON implementations
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn simd_memcpy_neon(dst: &mut [u8], src: &[u8], len: usize) -> usize {
    use std::arch::aarch64::*;

    let mut offset = 0;

    // Process 16 bytes at a time with NEON
    while offset + 16 <= len {
        let src_ptr = src.as_ptr().add(offset);
        let dst_ptr = dst.as_mut_ptr().add(offset);

        let data = vld1q_u8(src_ptr);
        vst1q_u8(dst_ptr, data);

        offset += 16;
    }

    // Handle remaining bytes
    while offset < len {
        *dst.get_unchecked_mut(offset) = *src.get_unchecked(offset);
        offset += 1;
    }

    len
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn simd_memset_neon(dst: &mut [u8], value: u8, len: usize) -> usize {
    use std::arch::aarch64::*;

    let pattern = vdupq_n_u8(value);
    let mut offset = 0;

    // Process 16 bytes at a time
    while offset + 16 <= len {
        let dst_ptr = dst.as_mut_ptr().add(offset);
        vst1q_u8(dst_ptr, pattern);
        offset += 16;
    }

    // Handle remaining bytes
    while offset < len {
        *dst.get_unchecked_mut(offset) = value;
        offset += 1;
    }

    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_memcpy() {
        let src = vec![1u8; 1024];
        let mut dst = vec![0u8; 1024];

        let copied = simd_memcpy(&mut dst, &src);
        assert_eq!(copied, 1024);
        assert_eq!(dst, src);
    }

    #[test]
    fn test_simd_memcmp() {
        let a = vec![1u8; 1024];
        let b = vec![1u8; 1024];
        let c = vec![2u8; 1024];

        assert_eq!(simd_memcmp(&a, &b), Ordering::Equal);
        assert_eq!(simd_memcmp(&a, &c), Ordering::Less);
    }

    #[test]
    fn test_simd_memset() {
        let mut dst = vec![0u8; 1024];

        let written = simd_memset(&mut dst, 42);
        assert_eq!(written, 1024);
        assert!(dst.iter().all(|&b| b == 42));
    }

    #[test]
    fn test_simd_memmove_overlapping() {
        let mut buf = vec![1, 2, 3, 4, 5, 6, 7, 8];

        // Move overlapping region
        let (src, dst) = buf.split_at_mut(3);
        simd_memmove(dst, src);

        assert_eq!(&buf[3..6], &[1, 2, 3]);
    }
}
