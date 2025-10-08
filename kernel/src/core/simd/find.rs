/*!
 * Vectorized Search Operations
 * SIMD-accelerated search and pattern matching
 */

/// Find first occurrence of a byte in a slice
///
/// Returns the index of the first occurrence, or None if not found
pub fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512bw")
            && haystack.len() >= 64
        {
            unsafe {
                if let Some(pos) = find_byte_avx512(haystack, needle) {
                    return Some(pos);
                }
            }
        }
        if is_x86_feature_detected!("avx2") && haystack.len() >= 32 {
            unsafe {
                if let Some(pos) = find_byte_avx2(haystack, needle) {
                    return Some(pos);
                }
            }
        }
        if is_x86_feature_detected!("sse2") && haystack.len() >= 16 {
            unsafe {
                if let Some(pos) = find_byte_sse2(haystack, needle) {
                    return Some(pos);
                }
            }
        }
    }

    // Fallback to standard search
    haystack.iter().position(|&b| b == needle)
}

/// Count occurrences of a byte in a slice
pub fn count_byte(haystack: &[u8], needle: u8) -> usize {
    if haystack.is_empty() {
        return 0;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512bw")
            && is_x86_feature_detected!("avx512vpopcntdq")
            && haystack.len() >= 64
        {
            unsafe {
                return count_byte_avx512(haystack, needle);
            }
        }
        if is_x86_feature_detected!("avx2") && haystack.len() >= 32 {
            unsafe {
                return count_byte_avx2(haystack, needle);
            }
        }
    }

    // Fallback
    haystack.iter().filter(|&&b| b == needle).count()
}

/// Check if slice contains a specific byte
#[inline]
pub fn contains_byte(haystack: &[u8], needle: u8) -> bool {
    find_byte(haystack, needle).is_some()
}

/// Find last occurrence of a byte in a slice
pub fn rfind_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }

    // For reverse search, fallback is often competitive
    // SIMD reverse search is complex and not always faster
    haystack.iter().rposition(|&b| b == needle)
}

// x86_64 AVX-512 implementations
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
unsafe fn find_byte_avx512(haystack: &[u8], needle: u8) -> Option<usize> {
    use std::arch::x86_64::*;

    let pattern = _mm512_set1_epi8(needle as i8);
    let mut offset = 0;
    let len = haystack.len();

    // Process 64 bytes at a time
    while offset + 64 <= len {
        let data_ptr = haystack.as_ptr().add(offset) as *const __m512i;
        let data = _mm512_loadu_si512(data_ptr);

        let mask = _mm512_cmpeq_epi8_mask(data, pattern);

        if mask != 0 {
            // Found at least one match
            let pos = mask.trailing_zeros() as usize;
            return Some(offset + pos);
        }

        offset += 64;
    }

    // Handle remaining bytes with scalar search
    haystack[offset..]
        .iter()
        .position(|&b| b == needle)
        .map(|p| offset + p)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn find_byte_avx2(haystack: &[u8], needle: u8) -> Option<usize> {
    use std::arch::x86_64::*;

    let pattern = _mm256_set1_epi8(needle as i8);
    let mut offset = 0;
    let len = haystack.len();

    // Process 32 bytes at a time
    while offset + 32 <= len {
        let data_ptr = haystack.as_ptr().add(offset) as *const __m256i;
        let data = _mm256_loadu_si256(data_ptr);

        let cmp = _mm256_cmpeq_epi8(data, pattern);
        let mask = _mm256_movemask_epi8(cmp);

        if mask != 0 {
            // Found at least one match
            let pos = mask.trailing_zeros() as usize;
            return Some(offset + pos);
        }

        offset += 32;
    }

    // Handle remaining bytes
    haystack[offset..]
        .iter()
        .position(|&b| b == needle)
        .map(|p| offset + p)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn find_byte_sse2(haystack: &[u8], needle: u8) -> Option<usize> {
    use std::arch::x86_64::*;

    let pattern = _mm_set1_epi8(needle as i8);
    let mut offset = 0;
    let len = haystack.len();

    // Process 16 bytes at a time
    while offset + 16 <= len {
        let data_ptr = haystack.as_ptr().add(offset) as *const __m128i;
        let data = _mm_loadu_si128(data_ptr);

        let cmp = _mm_cmpeq_epi8(data, pattern);
        let mask = _mm_movemask_epi8(cmp);

        if mask != 0 {
            let pos = mask.trailing_zeros() as usize;
            return Some(offset + pos);
        }

        offset += 16;
    }

    // Handle remaining bytes
    haystack[offset..]
        .iter()
        .position(|&b| b == needle)
        .map(|p| offset + p)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw,avx512vpopcntdq")]
unsafe fn count_byte_avx512(haystack: &[u8], needle: u8) -> usize {
    use std::arch::x86_64::*;

    let pattern = _mm512_set1_epi8(needle as i8);
    let mut count = 0;
    let mut offset = 0;
    let len = haystack.len();

    // Process 64 bytes at a time
    while offset + 64 <= len {
        let data_ptr = haystack.as_ptr().add(offset) as *const __m512i;
        let data = _mm512_loadu_si512(data_ptr);

        let mask = _mm512_cmpeq_epi8_mask(data, pattern);
        count += mask.count_ones() as usize;

        offset += 64;
    }

    // Handle remaining bytes
    count + haystack[offset..].iter().filter(|&&b| b == needle).count()
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn count_byte_avx2(haystack: &[u8], needle: u8) -> usize {
    use std::arch::x86_64::*;

    let pattern = _mm256_set1_epi8(needle as i8);
    let mut count = 0;
    let mut offset = 0;
    let len = haystack.len();

    // Process 32 bytes at a time
    while offset + 32 <= len {
        let data_ptr = haystack.as_ptr().add(offset) as *const __m256i;
        let data = _mm256_loadu_si256(data_ptr);

        let cmp = _mm256_cmpeq_epi8(data, pattern);
        let mask = _mm256_movemask_epi8(cmp);
        count += mask.count_ones() as usize;

        offset += 32;
    }

    // Handle remaining bytes
    count + haystack[offset..].iter().filter(|&&b| b == needle).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_byte_simple() {
        let data = b"hello world";
        assert_eq!(find_byte(data, b'h'), Some(0));
        assert_eq!(find_byte(data, b'o'), Some(4));
        assert_eq!(find_byte(data, b'd'), Some(10));
        assert_eq!(find_byte(data, b'x'), None);
    }

    #[test]
    fn test_find_byte_large() {
        let mut data = vec![0u8; 1024];
        data[512] = 42;
        assert_eq!(find_byte(&data, 42), Some(512));
        assert_eq!(find_byte(&data, 99), None);
    }

    #[test]
    fn test_count_byte() {
        let data = b"hello world";
        assert_eq!(count_byte(data, b'l'), 3);
        assert_eq!(count_byte(data, b'o'), 2);
        assert_eq!(count_byte(data, b'h'), 1);
        assert_eq!(count_byte(data, b'x'), 0);
    }

    #[test]
    fn test_count_byte_large() {
        let data = vec![42u8; 1000];
        assert_eq!(count_byte(&data, 42), 1000);
        assert_eq!(count_byte(&data, 0), 0);
    }

    #[test]
    fn test_contains_byte() {
        let data = b"hello world";
        assert!(contains_byte(data, b'h'));
        assert!(contains_byte(data, b'o'));
        assert!(!contains_byte(data, b'x'));
    }

    #[test]
    fn test_rfind_byte() {
        let data = b"hello world";
        assert_eq!(rfind_byte(data, b'o'), Some(7));
        assert_eq!(rfind_byte(data, b'l'), Some(9));
        assert_eq!(rfind_byte(data, b'h'), Some(0));
        assert_eq!(rfind_byte(data, b'x'), None);
    }

    #[test]
    fn test_empty() {
        let data = b"";
        assert_eq!(find_byte(data, b'a'), None);
        assert_eq!(count_byte(data, b'a'), 0);
        assert!(!contains_byte(data, b'a'));
    }
}
