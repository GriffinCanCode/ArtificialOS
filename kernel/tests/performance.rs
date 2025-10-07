/*!
 * Performance tests entry point
 */

#[path = "performance/simd_test.rs"]
mod simd_test;

#[path = "performance/jit_test.rs"]
mod jit_test;

#[path = "performance/dashmap_stress_test.rs"]
mod dashmap_stress_test;

#[path = "performance/ahash_test.rs"]
mod ahash_test;
