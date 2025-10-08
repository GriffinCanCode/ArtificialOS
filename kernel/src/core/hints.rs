/*!
 * Compiler Optimization Hints
 * Branch prediction hints and performance optimization utilities for kernel
 */

#![allow(unused)]

/// Hint to the compiler that this branch is likely to be taken
///
/// # Performance
/// Helps the compiler optimize branch prediction and code layout.
/// Use for hot paths that are executed frequently.
///
/// # Implementation
/// On nightly with core_intrinsics, uses LLVM's `llvm.expect` intrinsic.
/// On stable, returns the value unchanged but still provides semantic documentation.
///
/// # Example
/// ```ignore
/// if likely(x > 0) {
///     // Hot path - executed 99% of the time
/// } else {
///     // Cold path - rare error case
/// }
/// ```
#[inline(always)]
#[must_use]
pub fn likely(b: bool) -> bool {
    #[cfg(feature = "nightly")]
    {
        // Use unstable intrinsic on nightly for actual branch hints
        unsafe { core::intrinsics::likely(b) }
    }
    #[cfg(not(feature = "nightly"))]
    {
        // On stable, just return the value
        // The #[inline(always)] still helps with optimization
        // and the semantic meaning helps developers
        b
    }
}

/// Hint to the compiler that this branch is unlikely to be taken
///
/// # Performance
/// Helps the compiler optimize branch prediction and code layout.
/// Use for error paths and exceptional cases.
///
/// # Implementation
/// On nightly with core_intrinsics, uses LLVM's `llvm.expect` intrinsic.
/// On stable, returns the value unchanged but still provides semantic documentation.
///
/// # Example
/// ```ignore
/// if unlikely(error_occurred) {
///     // Cold path - rarely executed error handling
/// } else {
///     // Hot path - normal execution
/// }
/// ```
#[inline(always)]
#[must_use]
pub fn unlikely(b: bool) -> bool {
    #[cfg(feature = "nightly")]
    {
        // Use unstable intrinsic on nightly for actual branch hints
        unsafe { core::intrinsics::unlikely(b) }
    }
    #[cfg(not(feature = "nightly"))]
    {
        // On stable, just return the value
        // The #[inline(always)] still helps with optimization
        // and the semantic meaning helps developers
        b
    }
}

/// Prevent the compiler from optimizing away or const-folding a value
///
/// Useful for benchmarking and ensuring certain operations aren't eliminated
pub use std::hint::black_box;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_likely() {
        assert!(likely(true));
        assert!(!likely(false));
    }

    #[test]
    fn test_unlikely() {
        assert!(unlikely(true));
        assert!(!unlikely(false));
    }
}
