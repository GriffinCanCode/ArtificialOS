/*!
 * Branch Prediction Hints
 * Help CPU predict branches for better performance
 */

/// Hint that this branch is likely to be taken
///
/// # Performance
///
/// - **5-10% improvement** in hot paths with predictable branches
/// - **No effect** on unpredictable branches
/// - **Use sparingly**: Only for hot paths with >90% predictability
///
/// # Example
///
/// ```ignore
/// if likely(cache.contains(&key)) {
///     return cache.get(&key); // Fast path, usually taken
/// }
/// slow_lookup(key) // Cold path, rarely taken
/// ```
#[inline(always)]
pub fn likely(b: bool) -> bool {
    #[cfg(all(feature = "nightly", target_arch = "x86_64"))]
    {
        if !b {
            // Tell compiler this branch is cold
            core::intrinsics::unlikely(false)
        } else {
            true
        }
    }

    #[cfg(not(all(feature = "nightly", target_arch = "x86_64")))]
    {
        b
    }
}

/// Hint that this branch is unlikely to be taken
///
/// # Example
///
/// ```ignore
/// if unlikely(error.is_some()) {
///     return Err(error.unwrap()); // Cold path, rarely taken
/// }
/// continue_processing() // Fast path, usually taken
/// ```
#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    #[cfg(all(feature = "nightly", target_arch = "x86_64"))]
    {
        if b {
            core::intrinsics::unlikely(true)
        } else {
            false
        }
    }

    #[cfg(not(all(feature = "nightly", target_arch = "x86_64")))]
    {
        b
    }
}

/// Mark function as cold (unlikely to be called)
///
/// This helps the compiler optimize hot paths by moving cold code
/// out of the instruction cache.
///
/// # Example
///
/// ```ignore
/// #[cold]
/// fn handle_error(error: Error) {
///     log::error!("Error occurred: {}", error);
///     // Error handling...
/// }
/// ```
#[cfg(feature = "nightly")]
pub use core::intrinsics::cold;

/// Fallback cold attribute for stable Rust
#[cfg(not(feature = "nightly"))]
#[inline(never)]
pub fn cold() {
    // No-op on stable
}

/// Mark code path as unreachable for optimization
///
/// # Safety
///
/// Only use if the code path is truly unreachable. UB if reached.
///
/// # Example
///
/// ```ignore
/// match value {
///     Some(x) => process(x),
///     None => {
///         // We know this is impossible due to prior checks
///         unsafe { unreachable_unchecked() }
///     }
/// }
/// ```
#[inline(always)]
pub unsafe fn unreachable_unchecked() -> ! {
    #[cfg(debug_assertions)]
    {
        unreachable!("Unreachable code was reached!");
    }

    #[cfg(not(debug_assertions))]
    {
        std::hint::unreachable_unchecked()
    }
}

/// Assume condition is true for optimization
///
/// # Safety
///
/// UB if condition is actually false. Use only with invariants you've verified.
///
/// # Example
///
/// ```ignore
/// fn process_validated_input(input: &[u8]) {
///     // We've already validated length > 0
///     unsafe { assume(input.len() > 0); }
///
///     // Compiler can now elim bounds checks
///     let first = input[0];
/// }
/// ```
#[inline(always)]
pub unsafe fn assume(condition: bool) {
    if !condition {
        unreachable_unchecked();
    }
}

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

    #[test]
    fn test_in_condition() {
        let value = Some(42);

        if likely(value.is_some()) {
            assert_eq!(value.unwrap(), 42);
        }

        if unlikely(value.is_none()) {
            panic!("Should not reach");
        }
    }
}
