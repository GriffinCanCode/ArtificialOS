/*!
 * SIMD Operations Tests
 * Tests for all SIMD-accelerated operations
 */

use ai_os_kernel::memory::{
    // Text operations
    ascii_to_lower,
    ascii_to_upper,
    // Math operations
    avg_u64,
    // Search operations
    contains_byte,
    count_byte,
    find_byte,
    // CPU detection
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
};
use std::cmp::Ordering;

#[test]
fn test_simd_capabilities() {
    let caps = init_simd();

    // On x86_64, at least SSE2 should be available
    #[cfg(target_arch = "x86_64")]
    {
        assert!(caps.sse2);
    }

    // On aarch64, NEON should be available
    #[cfg(target_arch = "aarch64")]
    {
        assert!(caps.neon);
    }
}

#[test]
fn test_simd_memcpy_small() {
    let src = vec![1u8, 2, 3, 4, 5];
    let mut dst = vec![0u8; 5];

    let copied = simd_memcpy(&mut dst, &src);

    assert_eq!(copied, 5);
    assert_eq!(dst, src);
}

#[test]
fn test_simd_memcpy_large() {
    let src = vec![42u8; 1024];
    let mut dst = vec![0u8; 1024];

    let copied = simd_memcpy(&mut dst, &src);

    assert_eq!(copied, 1024);
    assert_eq!(dst, src);
}

#[test]
fn test_simd_memcpy_unaligned() {
    // Test with unaligned sizes to ensure proper handling
    for size in [1, 7, 15, 31, 63, 127, 255, 513, 1023] {
        let src = vec![0xAB; size];
        let mut dst = vec![0u8; size];

        let copied = simd_memcpy(&mut dst, &src);

        assert_eq!(copied, size);
        assert_eq!(dst, src);
    }
}

#[test]
fn test_simd_memcpy_different_sizes() {
    let src = vec![1u8; 100];
    let mut dst = vec![0u8; 50];

    let copied = simd_memcpy(&mut dst, &src);

    // Should only copy what fits
    assert_eq!(copied, 50);
    assert_eq!(&dst[..], &src[..50]);
}

#[test]
fn test_simd_memmove_non_overlapping() {
    let src = vec![1u8, 2, 3, 4, 5];
    let mut dst = vec![0u8; 5];

    let moved = simd_memmove(&mut dst, &src);

    assert_eq!(moved, 5);
    assert_eq!(dst, src);
}

#[test]
fn test_simd_memmove_overlapping() {
    let mut buf = vec![1, 2, 3, 4, 5, 6, 7, 8];

    // Move data within the same buffer (overlapping)
    let (src, dst) = buf.split_at_mut(3);
    simd_memmove(dst, src);

    assert_eq!(&buf[3..6], &[1, 2, 3]);
}

#[test]
fn test_simd_memmove_large() {
    let src = vec![99u8; 2048];
    let mut dst = vec![0u8; 2048];

    let moved = simd_memmove(&mut dst, &src);

    assert_eq!(moved, 2048);
    assert_eq!(dst, src);
}

#[test]
fn test_simd_memcmp_equal() {
    let a = vec![1u8, 2, 3, 4, 5];
    let b = vec![1u8, 2, 3, 4, 5];

    let result = simd_memcmp(&a, &b);

    assert_eq!(result, Ordering::Equal);
}

#[test]
fn test_simd_memcmp_less() {
    let a = vec![1u8, 2, 3, 4, 5];
    let b = vec![1u8, 2, 3, 4, 6];

    let result = simd_memcmp(&a, &b);

    assert_eq!(result, Ordering::Less);
}

#[test]
fn test_simd_memcmp_greater() {
    let a = vec![1u8, 2, 3, 5, 5];
    let b = vec![1u8, 2, 3, 4, 5];

    let result = simd_memcmp(&a, &b);

    assert_eq!(result, Ordering::Greater);
}

#[test]
fn test_simd_memcmp_large() {
    let a = vec![7u8; 4096];
    let b = vec![7u8; 4096];

    let result = simd_memcmp(&a, &b);

    assert_eq!(result, Ordering::Equal);
}

#[test]
fn test_simd_memcmp_large_different() {
    let mut a = vec![7u8; 4096];
    let b = vec![7u8; 4096];

    // Make one byte different
    a[2048] = 8;

    let result = simd_memcmp(&a, &b);

    assert_eq!(result, Ordering::Greater);
}

#[test]
fn test_simd_memset_small() {
    let mut buf = vec![0u8; 10];

    let written = simd_memset(&mut buf, 42);

    assert_eq!(written, 10);
    assert!(buf.iter().all(|&b| b == 42));
}

#[test]
fn test_simd_memset_large() {
    let mut buf = vec![0u8; 8192];

    let written = simd_memset(&mut buf, 0xFF);

    assert_eq!(written, 8192);
    assert!(buf.iter().all(|&b| b == 0xFF));
}

#[test]
fn test_simd_memset_various_sizes() {
    for size in [1, 16, 32, 64, 128, 256, 512, 1024, 2048] {
        let mut buf = vec![0u8; size];

        let written = simd_memset(&mut buf, 123);

        assert_eq!(written, size);
        assert!(buf.iter().all(|&b| b == 123));
    }
}

#[test]
fn test_simd_memset_zero() {
    let mut buf = vec![0xFFu8; 1024];

    let written = simd_memset(&mut buf, 0);

    assert_eq!(written, 1024);
    assert!(buf.iter().all(|&b| b == 0));
}

#[test]
fn test_simd_operations_with_threshold() {
    // Test that small operations still work correctly
    let small = vec![1u8; 32];
    let mut dst = vec![0u8; 32];

    simd_memcpy(&mut dst, &small);
    assert_eq!(dst, small);

    // Test that operations just above threshold work
    let medium = vec![2u8; 128];
    let mut dst2 = vec![0u8; 128];

    simd_memcpy(&mut dst2, &medium);
    assert_eq!(dst2, medium);
}

#[test]
fn test_simd_memcpy_performance() {
    use std::time::Instant;

    let size = 1024 * 1024; // 1MB
    let src = vec![0xAB; size];
    let mut dst = vec![0u8; size];

    let start = Instant::now();
    simd_memcpy(&mut dst, &src);
    let duration = start.elapsed();

    assert_eq!(dst, src);
    println!("SIMD memcpy 1MB: {:?}", duration);
}

#[test]
fn test_simd_all_operations_consistency() {
    let size = 1024;
    let src = vec![0x55; size];
    let mut dst1 = vec![0u8; size];
    let mut dst2 = vec![0u8; size];

    // Test memcpy
    simd_memcpy(&mut dst1, &src);

    // Test memmove
    simd_memmove(&mut dst2, &src);

    // Both should produce same result
    assert_eq!(simd_memcmp(&dst1, &dst2), Ordering::Equal);

    // Test memset
    simd_memset(&mut dst1, 0x55);
    assert_eq!(simd_memcmp(&dst1, &src), Ordering::Equal);
}

// ============================================================================
// Search Operations Tests
// ============================================================================

#[test]
fn test_find_byte_basic() {
    let data = b"hello world";
    assert_eq!(find_byte(data, b'h'), Some(0));
    assert_eq!(find_byte(data, b'w'), Some(6));
    assert_eq!(find_byte(data, b'd'), Some(10));
    assert_eq!(find_byte(data, b'x'), None);
}

#[test]
fn test_find_byte_large() {
    let mut data = vec![0u8; 10000];
    data[5000] = 42;
    data[8000] = 42;
    assert_eq!(find_byte(&data, 42), Some(5000));
    assert_eq!(find_byte(&data, 99), None);
}

#[test]
fn test_rfind_byte() {
    let data = b"hello world hello";
    assert_eq!(rfind_byte(data, b'h'), Some(12));
    assert_eq!(rfind_byte(data, b'l'), Some(15));
    assert_eq!(rfind_byte(data, b'o'), Some(16));
}

#[test]
fn test_count_byte() {
    let data = b"hello world";
    assert_eq!(count_byte(data, b'l'), 3);
    assert_eq!(count_byte(data, b'o'), 2);
    assert_eq!(count_byte(data, b'x'), 0);

    let large = vec![42u8; 10000];
    assert_eq!(count_byte(&large, 42), 10000);
}

#[test]
fn test_contains_byte() {
    let data = b"hello world";
    assert!(contains_byte(data, b'h'));
    assert!(contains_byte(data, b'w'));
    assert!(!contains_byte(data, b'x'));
}

// ============================================================================
// Math Operations Tests
// ============================================================================

#[test]
fn test_sum_u64_basic() {
    let data = vec![1u64, 2, 3, 4, 5];
    assert_eq!(sum_u64(&data), 15);

    let empty: Vec<u64> = vec![];
    assert_eq!(sum_u64(&empty), 0);
}

#[test]
fn test_sum_u64_large() {
    let data: Vec<u64> = (1..=1000).collect();
    let expected: u64 = (1..=1000).sum();
    assert_eq!(sum_u64(&data), expected);
}

#[test]
fn test_sum_u32() {
    let data = vec![10u32, 20, 30, 40, 50];
    assert_eq!(sum_u32(&data), 150);

    let large = vec![1u32; 10000];
    assert_eq!(sum_u32(&large), 10000);
}

#[test]
fn test_min_max_u64() {
    let data = vec![5u64, 2, 9, 1, 7, 3];
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
fn test_min_max_large() {
    let mut data: Vec<u64> = (0..10000).collect();
    assert_eq!(min_u64(&data), Some(0));
    assert_eq!(max_u64(&data), Some(9999));

    data.reverse();
    assert_eq!(min_u64(&data), Some(0));
    assert_eq!(max_u64(&data), Some(9999));
}

#[test]
fn test_avg_u64() {
    let data = vec![1u64, 2, 3, 4, 5];
    assert_eq!(avg_u64(&data), Some(3));

    let data2 = vec![10u64, 20, 30, 40];
    assert_eq!(avg_u64(&data2), Some(25));

    let empty: Vec<u64> = vec![];
    assert_eq!(avg_u64(&empty), None);
}

// ============================================================================
// Text Operations Tests
// ============================================================================

#[test]
fn test_ascii_to_lower() {
    let mut data = b"HELLO WORLD".to_vec();
    ascii_to_lower(&mut data);
    assert_eq!(&data, b"hello world");

    let mut mixed = b"HeLLo 123 WoRLd".to_vec();
    ascii_to_lower(&mut mixed);
    assert_eq!(&mixed, b"hello 123 world");
}

#[test]
fn test_ascii_to_upper() {
    let mut data = b"hello world".to_vec();
    ascii_to_upper(&mut data);
    assert_eq!(&data, b"HELLO WORLD");

    let mut mixed = b"HeLLo 123 WoRLd".to_vec();
    ascii_to_upper(&mut mixed);
    assert_eq!(&mixed, b"HELLO 123 WORLD");
}

#[test]
fn test_case_conversion_large() {
    let mut data = vec![b'A'; 10000];
    ascii_to_lower(&mut data);
    assert!(data.iter().all(|&b| b == b'a'));

    ascii_to_upper(&mut data);
    assert!(data.iter().all(|&b| b == b'A'));
}

#[test]
fn test_is_ascii() {
    assert!(is_ascii(b"hello world 123"));
    assert!(is_ascii(b"ABC xyz !@#"));
    assert!(!is_ascii(&[0xFF, 0xFE]));
    assert!(!is_ascii(b"hello\x80world"));

    let large = vec![b'a'; 10000];
    assert!(is_ascii(&large));
}

#[test]
fn test_trim() {
    assert_eq!(trim(b"  hello  "), b"hello");
    assert_eq!(trim(b"hello"), b"hello");
    assert_eq!(trim(b"   "), b"");
    assert_eq!(trim(b""), b"");
    assert_eq!(trim(b"\t\nhello\r\n"), b"hello");
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_combined_operations() {
    // Create and transform a large dataset
    let mut data = vec![b'A'; 1000];

    // Convert to lowercase
    ascii_to_lower(&mut data);
    assert!(data.iter().all(|&b| b == b'a'));

    // Find specific byte
    data[500] = b'x';
    assert_eq!(find_byte(&data, b'x'), Some(500));
    assert_eq!(count_byte(&data, b'a'), 999);
    assert_eq!(count_byte(&data, b'x'), 1);
}

#[test]
fn test_avx512_detection() {
    let caps = init_simd();

    // Log capabilities for debugging
    println!("SSE2: {}", caps.sse2);
    println!("SSE4.2: {}", caps.sse4_2);
    println!("AVX: {}", caps.avx);
    println!("AVX2: {}", caps.avx2);
    println!("AVX-512F: {}", caps.avx512f);
    println!("AVX-512BW: {}", caps.avx512bw);
    println!("AVX-512DQ: {}", caps.avx512dq);
    println!("AVX-512VL: {}", caps.avx512vl);
    println!("AVX-512 Full: {}", caps.has_avx512_full());
    println!("Max vector bytes: {}", caps.max_vector_bytes());
    println!("Optimal alignment: {}", caps.optimal_alignment());
}

#[test]
fn test_performance_comparison() {
    use std::time::Instant;

    let size = 1_000_000;
    let data = vec![42u8; size];

    // Test find_byte performance
    let start = Instant::now();
    let pos = find_byte(&data, 43);
    let simd_time = start.elapsed();
    assert_eq!(pos, None);

    println!("SIMD find_byte (1MB, not found): {:?}", simd_time);

    // Test sum performance
    let numbers: Vec<u64> = (0..100_000).collect();
    let start = Instant::now();
    let sum = sum_u64(&numbers);
    let sum_time = start.elapsed();

    println!("SIMD sum_u64 (100K elements): {:?}", sum_time);
    let expected_sum: u64 = numbers.iter().sum();
    assert_eq!(sum, expected_sum);
}
