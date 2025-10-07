/*!
 * SIMD Memory Operations Tests
 * Tests for SIMD-accelerated memory operations
 */

use ai_os_kernel::memory::{simd_memcpy, simd_memmove, simd_memcmp, simd_memset, init_simd};
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

