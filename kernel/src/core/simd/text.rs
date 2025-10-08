/*!
 * Vectorized String Operations
 * SIMD-accelerated text processing
 */

/// Convert ASCII string to lowercase in-place
pub fn ascii_to_lower(data: &mut [u8]) {
    if data.is_empty() {
        return;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512bw")
            && data.len() >= 64
        {
            unsafe {
                ascii_to_lower_avx512(data);
                return;
            }
        }
        if is_x86_feature_detected!("avx2") && data.len() >= 32 {
            unsafe {
                ascii_to_lower_avx2(data);
                return;
            }
        }
    }

    // Fallback
    for byte in data.iter_mut() {
        if byte.is_ascii_uppercase() {
            *byte = byte.to_ascii_lowercase();
        }
    }
}

/// Convert ASCII string to uppercase in-place
pub fn ascii_to_upper(data: &mut [u8]) {
    if data.is_empty() {
        return;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512bw")
            && data.len() >= 64
        {
            unsafe {
                ascii_to_upper_avx512(data);
                return;
            }
        }
        if is_x86_feature_detected!("avx2") && data.len() >= 32 {
            unsafe {
                ascii_to_upper_avx2(data);
                return;
            }
        }
    }

    // Fallback
    for byte in data.iter_mut() {
        if byte.is_ascii_lowercase() {
            *byte = byte.to_ascii_uppercase();
        }
    }
}

/// Check if all bytes are valid ASCII
pub fn is_ascii(data: &[u8]) -> bool {
    if data.is_empty() {
        return true;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512bw")
            && data.len() >= 64
        {
            unsafe {
                return is_ascii_avx512(data);
            }
        }
        if is_x86_feature_detected!("avx2") && data.len() >= 32 {
            unsafe {
                return is_ascii_avx2(data);
            }
        }
    }

    // Fallback
    data.iter().all(|&b| b < 128)
}

/// Count whitespace characters
#[inline]
pub fn count_whitespace(data: &[u8]) -> usize {
    data.iter().filter(|&&b| b.is_ascii_whitespace()).count()
}

/// Trim whitespace from start
pub fn trim_start(data: &[u8]) -> &[u8] {
    let start = data
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .unwrap_or(data.len());
    &data[start..]
}

/// Trim whitespace from end
pub fn trim_end(data: &[u8]) -> &[u8] {
    let end = data
        .iter()
        .rposition(|&b| !b.is_ascii_whitespace())
        .map(|i| i + 1)
        .unwrap_or(0);
    &data[..end]
}

/// Trim whitespace from both ends
#[inline]
pub fn trim(data: &[u8]) -> &[u8] {
    trim_end(trim_start(data))
}

// x86_64 AVX-512 implementations
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
unsafe fn ascii_to_lower_avx512(data: &mut [u8]) {
    use std::arch::x86_64::*;

    let upper_a = _mm512_set1_epi8(b'A' as i8);
    let upper_z = _mm512_set1_epi8(b'Z' as i8);
    let to_lower = _mm512_set1_epi8(32);

    let mut offset = 0;
    let len = data.len();

    // Process 64 bytes at a time
    while offset + 64 <= len {
        let ptr = data.as_mut_ptr().add(offset) as *mut __m512i;
        let chars = _mm512_loadu_si512(ptr);

        // Check if char >= 'A' and char <= 'Z'
        let ge_a = _mm512_cmpge_epu8_mask(chars, upper_a);
        let le_z = _mm512_cmple_epu8_mask(chars, upper_z);
        let is_upper = ge_a & le_z;

        // Add 32 to uppercase letters
        let lower = _mm512_mask_add_epi8(chars, is_upper, chars, to_lower);
        _mm512_storeu_si512(ptr, lower);

        offset += 64;
    }

    // Handle remaining bytes
    for byte in &mut data[offset..] {
        if byte.is_ascii_uppercase() {
            *byte = byte.to_ascii_lowercase();
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn ascii_to_lower_avx2(data: &mut [u8]) {
    use std::arch::x86_64::*;

    let upper_a = _mm256_set1_epi8(b'A' as i8);
    let upper_z = _mm256_set1_epi8(b'Z' as i8);
    let to_lower = _mm256_set1_epi8(32);

    let mut offset = 0;
    let len = data.len();

    // Process 32 bytes at a time
    while offset + 32 <= len {
        let ptr = data.as_mut_ptr().add(offset) as *mut __m256i;
        let chars = _mm256_loadu_si256(ptr);

        // Check if char >= 'A' and char <= 'Z'
        let ge_a = _mm256_cmpgt_epi8(chars, _mm256_sub_epi8(upper_a, _mm256_set1_epi8(1).into()));
        let le_z = _mm256_cmpgt_epi8(_mm256_add_epi8(upper_z, _mm256_set1_epi8(1).into()), chars);
        let is_upper = _mm256_and_si256(ge_a, le_z);

        // Add 32 to uppercase letters
        let add_val = _mm256_and_si256(is_upper, to_lower);
        let lower = _mm256_add_epi8(chars, add_val);
        _mm256_storeu_si256(ptr, lower);

        offset += 32;
    }

    // Handle remaining bytes
    for byte in &mut data[offset..] {
        if byte.is_ascii_uppercase() {
            *byte = byte.to_ascii_lowercase();
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
unsafe fn ascii_to_upper_avx512(data: &mut [u8]) {
    use std::arch::x86_64::*;

    let lower_a = _mm512_set1_epi8(b'a' as i8);
    let lower_z = _mm512_set1_epi8(b'z' as i8);
    let to_upper = _mm512_set1_epi8(32);

    let mut offset = 0;
    let len = data.len();

    // Process 64 bytes at a time
    while offset + 64 <= len {
        let ptr = data.as_mut_ptr().add(offset) as *mut __m512i;
        let chars = _mm512_loadu_si512(ptr);

        // Check if char >= 'a' and char <= 'z'
        let ge_a = _mm512_cmpge_epu8_mask(chars, lower_a);
        let le_z = _mm512_cmple_epu8_mask(chars, lower_z);
        let is_lower = ge_a & le_z;

        // Subtract 32 from lowercase letters
        let upper = _mm512_mask_sub_epi8(chars, is_lower, chars, to_upper);
        _mm512_storeu_si512(ptr, upper);

        offset += 64;
    }

    // Handle remaining bytes
    for byte in &mut data[offset..] {
        if byte.is_ascii_lowercase() {
            *byte = byte.to_ascii_uppercase();
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn ascii_to_upper_avx2(data: &mut [u8]) {
    use std::arch::x86_64::*;

    let lower_a = _mm256_set1_epi8(b'a' as i8);
    let lower_z = _mm256_set1_epi8(b'z' as i8);
    let to_upper = _mm256_set1_epi8(32);

    let mut offset = 0;
    let len = data.len();

    // Process 32 bytes at a time
    while offset + 32 <= len {
        let ptr = data.as_mut_ptr().add(offset) as *mut __m256i;
        let chars = _mm256_loadu_si256(ptr);

        // Check if char >= 'a' and char <= 'z'
        let ge_a = _mm256_cmpgt_epi8(chars, _mm256_sub_epi8(lower_a, _mm256_set1_epi8(1).into()));
        let le_z = _mm256_cmpgt_epi8(_mm256_add_epi8(lower_z, _mm256_set1_epi8(1).into()), chars);
        let is_lower = _mm256_and_si256(ge_a, le_z);

        // Subtract 32 from lowercase letters
        let sub_val = _mm256_and_si256(is_lower, to_upper);
        let upper = _mm256_sub_epi8(chars, sub_val);
        _mm256_storeu_si256(ptr, upper);

        offset += 32;
    }

    // Handle remaining bytes
    for byte in &mut data[offset..] {
        if byte.is_ascii_lowercase() {
            *byte = byte.to_ascii_uppercase();
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
unsafe fn is_ascii_avx512(data: &[u8]) -> bool {
    use std::arch::x86_64::*;

    let mut offset = 0;
    let len = data.len();

    // Process 64 bytes at a time
    while offset + 64 <= len {
        let ptr = data.as_ptr().add(offset) as *const __m512i;
        let chars = _mm512_loadu_si512(ptr);

        // Check if any byte has high bit set
        let mask = _mm512_movepi8_mask(chars);
        if mask != 0 {
            return false;
        }

        offset += 64;
    }

    // Handle remaining bytes
    data[offset..].iter().all(|&b| b < 128)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn is_ascii_avx2(data: &[u8]) -> bool {
    use std::arch::x86_64::*;

    let mut offset = 0;
    let len = data.len();

    // Process 32 bytes at a time
    while offset + 32 <= len {
        let ptr = data.as_ptr().add(offset) as *const __m256i;
        let chars = _mm256_loadu_si256(ptr);

        // Check if any byte has high bit set
        let mask = _mm256_movemask_epi8(chars);
        if mask != 0 {
            return false;
        }

        offset += 32;
    }

    // Handle remaining bytes
    data[offset..].iter().all(|&b| b < 128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_to_lower() {
        let mut data = b"HELLO WORLD".to_vec();
        ascii_to_lower(&mut data);
        assert_eq!(&data, b"hello world");

        let mut mixed = b"HeLLo WoRLd 123".to_vec();
        ascii_to_lower(&mut mixed);
        assert_eq!(&mixed, b"hello world 123");
    }

    #[test]
    fn test_ascii_to_upper() {
        let mut data = b"hello world".to_vec();
        ascii_to_upper(&mut data);
        assert_eq!(&data, b"HELLO WORLD");

        let mut mixed = b"HeLLo WoRLd 123".to_vec();
        ascii_to_upper(&mut mixed);
        assert_eq!(&mixed, b"HELLO WORLD 123");
    }

    #[test]
    fn test_is_ascii() {
        assert!(is_ascii(b"hello world"));
        assert!(is_ascii(b"ABC 123 xyz"));
        assert!(!is_ascii(&[0x80, 0x81, 0x82]));
        assert!(!is_ascii(b"hello\xFFworld"));
    }

    #[test]
    fn test_large_strings() {
        let mut data = vec![b'A'; 1000];
        ascii_to_lower(&mut data);
        assert!(data.iter().all(|&b| b == b'a'));

        ascii_to_upper(&mut data);
        assert!(data.iter().all(|&b| b == b'A'));
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim(b"  hello  "), b"hello");
        assert_eq!(trim(b"hello"), b"hello");
        assert_eq!(trim(b"   "), b"");
        assert_eq!(trim(b""), b"");
    }

    #[test]
    fn test_trim_start() {
        assert_eq!(trim_start(b"  hello"), b"hello");
        assert_eq!(trim_start(b"hello  "), b"hello  ");
    }

    #[test]
    fn test_trim_end() {
        assert_eq!(trim_end(b"hello  "), b"hello");
        assert_eq!(trim_end(b"  hello"), b"  hello");
    }

    #[test]
    fn test_count_whitespace() {
        assert_eq!(count_whitespace(b"hello world"), 1);
        assert_eq!(count_whitespace(b"  hello  world  "), 6);
        assert_eq!(count_whitespace(b"helloworld"), 0);
    }
}
