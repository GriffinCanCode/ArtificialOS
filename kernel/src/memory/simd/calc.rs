/*!
 * Vectorized Math Operations
 * SIMD-accelerated mathematical computations
 */

/// Sum all u64 values in a slice
pub fn sum_u64(data: &[u64]) -> u64 {
    if data.is_empty() {
        return 0;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && data.len() >= 8 {
            unsafe {
                return sum_u64_avx512(data);
            }
        }
        if is_x86_feature_detected!("avx2") && data.len() >= 4 {
            unsafe {
                return sum_u64_avx2(data);
            }
        }
    }

    // Fallback
    data.iter().copied().sum()
}

/// Sum all u32 values in a slice
pub fn sum_u32(data: &[u32]) -> u32 {
    if data.is_empty() {
        return 0;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && data.len() >= 16 {
            unsafe {
                return sum_u32_avx512(data);
            }
        }
        if is_x86_feature_detected!("avx2") && data.len() >= 8 {
            unsafe {
                return sum_u32_avx2(data);
            }
        }
    }

    // Fallback
    data.iter().copied().sum()
}

/// Find minimum u64 value in a slice
pub fn min_u64(data: &[u64]) -> Option<u64> {
    if data.is_empty() {
        return None;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && data.len() >= 8 {
            unsafe {
                return Some(min_u64_avx512(data));
            }
        }
        if is_x86_feature_detected!("avx2") && data.len() >= 4 {
            unsafe {
                return Some(min_u64_avx2(data));
            }
        }
    }

    // Fallback
    data.iter().copied().min()
}

/// Find maximum u64 value in a slice
pub fn max_u64(data: &[u64]) -> Option<u64> {
    if data.is_empty() {
        return None;
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && data.len() >= 8 {
            unsafe {
                return Some(max_u64_avx512(data));
            }
        }
        if is_x86_feature_detected!("avx2") && data.len() >= 4 {
            unsafe {
                return Some(max_u64_avx2(data));
            }
        }
    }

    // Fallback
    data.iter().copied().max()
}

/// Calculate average of u64 values
#[inline]
pub fn avg_u64(data: &[u64]) -> Option<u64> {
    if data.is_empty() {
        return None;
    }
    Some(sum_u64(data) / data.len() as u64)
}

// x86_64 AVX-512 implementations
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn sum_u64_avx512(data: &[u64]) -> u64 {
    use std::arch::x86_64::*;

    let mut offset = 0;
    let len = data.len();
    let mut acc = _mm512_setzero_si512();

    // Process 8 u64 values at a time
    while offset + 8 <= len {
        let ptr = data.as_ptr().add(offset) as *const __m512i;
        let values = _mm512_loadu_si512(ptr);
        acc = _mm512_add_epi64(acc, values);
        offset += 8;
    }

    // Horizontal sum
    let mut sum = 0u64;
    let acc_arr: [u64; 8] = std::mem::transmute(acc);
    sum += acc_arr.iter().sum::<u64>();

    // Handle remaining elements
    sum + data[offset..].iter().copied().sum::<u64>()
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn sum_u64_avx2(data: &[u64]) -> u64 {
    use std::arch::x86_64::*;

    let mut offset = 0;
    let len = data.len();
    let mut acc = _mm256_setzero_si256();

    // Process 4 u64 values at a time
    while offset + 4 <= len {
        let ptr = data.as_ptr().add(offset) as *const __m256i;
        let values = _mm256_loadu_si256(ptr);
        acc = _mm256_add_epi64(acc, values);
        offset += 4;
    }

    // Horizontal sum
    let mut sum = 0u64;
    let acc_arr: [u64; 4] = std::mem::transmute(acc);
    sum += acc_arr.iter().sum::<u64>();

    // Handle remaining elements
    sum + data[offset..].iter().copied().sum::<u64>()
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn sum_u32_avx512(data: &[u32]) -> u32 {
    use std::arch::x86_64::*;

    let mut offset = 0;
    let len = data.len();
    let mut acc = _mm512_setzero_si512();

    // Process 16 u32 values at a time
    while offset + 16 <= len {
        let ptr = data.as_ptr().add(offset) as *const __m512i;
        let values = _mm512_loadu_si512(ptr);
        acc = _mm512_add_epi32(acc, values);
        offset += 16;
    }

    // Horizontal sum
    let mut sum = 0u32;
    let acc_arr: [u32; 16] = std::mem::transmute(acc);
    sum += acc_arr.iter().sum::<u32>();

    // Handle remaining elements
    sum + data[offset..].iter().copied().sum::<u32>()
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn sum_u32_avx2(data: &[u32]) -> u32 {
    use std::arch::x86_64::*;

    let mut offset = 0;
    let len = data.len();
    let mut acc = _mm256_setzero_si256();

    // Process 8 u32 values at a time
    while offset + 8 <= len {
        let ptr = data.as_ptr().add(offset) as *const __m256i;
        let values = _mm256_loadu_si256(ptr);
        acc = _mm256_add_epi32(acc, values);
        offset += 8;
    }

    // Horizontal sum
    let mut sum = 0u32;
    let acc_arr: [u32; 8] = std::mem::transmute(acc);
    sum += acc_arr.iter().sum::<u32>();

    // Handle remaining elements
    sum + data[offset..].iter().copied().sum::<u32>()
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn min_u64_avx512(data: &[u64]) -> u64 {
    use std::arch::x86_64::*;

    let mut offset = 0;
    let len = data.len();
    let mut min_vec = _mm512_set1_epi64(i64::MAX);

    // Process 8 u64 values at a time
    while offset + 8 <= len {
        let ptr = data.as_ptr().add(offset) as *const __m512i;
        let values = _mm512_loadu_si512(ptr);
        min_vec = _mm512_min_epu64(min_vec, values);
        offset += 8;
    }

    // Find minimum in vector
    let min_arr: [u64; 8] = std::mem::transmute(min_vec);
    let mut min_val = *min_arr.iter().min()
        .expect("AVX-512 min array should contain 8 elements");

    // Handle remaining elements (guaranteed non-empty by offset < len check)
    if offset < len {
        min_val = min_val.min(*data[offset..].iter().min()
            .expect("Remaining slice guaranteed non-empty"));
    }

    min_val
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn min_u64_avx2(data: &[u64]) -> u64 {
    use std::arch::x86_64::*;

    let mut offset = 0;
    let len = data.len();

    // AVX2 doesn't have native min_epu64, so we do it differently
    let mut min_val = data[0];

    // Process elements in chunks
    for chunk in data.chunks_exact(4) {
        let ptr = chunk.as_ptr() as *const __m256i;
        let values = _mm256_loadu_si256(ptr);
        let vals: [u64; 4] = std::mem::transmute(values);
        min_val = min_val.min(*vals.iter().min()
            .expect("AVX2 chunk contains exactly 4 elements"));
    }

    // Handle remaining elements (guaranteed non-empty by remainder check)
    let remainder_start = (data.len() / 4) * 4;
    if remainder_start < data.len() {
        min_val = min_val.min(*data[remainder_start..].iter().min()
            .expect("Remainder slice guaranteed non-empty"));
    }

    min_val
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn max_u64_avx512(data: &[u64]) -> u64 {
    use std::arch::x86_64::*;

    let mut offset = 0;
    let len = data.len();
    let mut max_vec = _mm512_setzero_si512();

    // Process 8 u64 values at a time
    while offset + 8 <= len {
        let ptr = data.as_ptr().add(offset) as *const __m512i;
        let values = _mm512_loadu_si512(ptr);
        max_vec = _mm512_max_epu64(max_vec, values);
        offset += 8;
    }

    // Find maximum in vector
    let max_arr: [u64; 8] = std::mem::transmute(max_vec);
    let mut max_val = *max_arr.iter().max()
        .expect("AVX-512 max array should contain 8 elements");

    // Handle remaining elements (guaranteed non-empty by offset < len check)
    if offset < len {
        max_val = max_val.max(*data[offset..].iter().max()
            .expect("Remaining slice guaranteed non-empty"));
    }

    max_val
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn max_u64_avx2(data: &[u64]) -> u64 {
    use std::arch::x86_64::*;

    let mut max_val = data[0];

    // Process elements in chunks
    for chunk in data.chunks_exact(4) {
        let ptr = chunk.as_ptr() as *const __m256i;
        let values = _mm256_loadu_si256(ptr);
        let vals: [u64; 4] = std::mem::transmute(values);
        max_val = max_val.max(*vals.iter().max()
            .expect("AVX2 chunk contains exactly 4 elements"));
    }

    // Handle remaining elements (guaranteed non-empty by remainder check)
    let remainder_start = (data.len() / 4) * 4;
    if remainder_start < data.len() {
        max_val = max_val.max(*data[remainder_start..].iter().max()
            .expect("Remainder slice guaranteed non-empty"));
    }

    max_val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_u64() {
        let data = vec![1u64, 2, 3, 4, 5];
        assert_eq!(sum_u64(&data), 15);

        let large = vec![100u64; 1000];
        assert_eq!(sum_u64(&large), 100_000);
    }

    #[test]
    fn test_sum_u32() {
        let data = vec![1u32, 2, 3, 4, 5];
        assert_eq!(sum_u32(&data), 15);

        let large = vec![50u32; 1000];
        assert_eq!(sum_u32(&large), 50_000);
    }

    #[test]
    fn test_min_max_u64() {
        let data = vec![5u64, 2, 8, 1, 9, 3];
        assert_eq!(min_u64(&data), Some(1));
        assert_eq!(max_u64(&data), Some(9));

        let single = vec![42u64];
        assert_eq!(min_u64(&single), Some(42));
        assert_eq!(max_u64(&single), Some(42));

        let empty: Vec<u64> = vec![];
        assert_eq!(min_u64(&empty), None);
        assert_eq!(max_u64(&empty), None);
    }

    #[test]
    fn test_avg_u64() {
        let data = vec![1u64, 2, 3, 4, 5];
        assert_eq!(avg_u64(&data), Some(3));

        let data2 = vec![10u64, 20, 30];
        assert_eq!(avg_u64(&data2), Some(20));

        let empty: Vec<u64> = vec![];
        assert_eq!(avg_u64(&empty), None);
    }

    #[test]
    fn test_large_arrays() {
        let data: Vec<u64> = (0..10000).collect();
        let sum = sum_u64(&data);
        let expected: u64 = data.iter().sum();
        assert_eq!(sum, expected);

        let min = min_u64(&data);
        assert_eq!(min, Some(0));

        let max = max_u64(&data);
        assert_eq!(max, Some(9999));
    }
}
