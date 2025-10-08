/*!
 * Serialization Utilities
 *
 * High-performance serialization for internal and external APIs:
 * - Bincode for internal IPC (5-10x faster than JSON, compact binary)
 * - JSON with SIMD optimization for external APIs (>1KB payloads)
 * - Custom serde helpers for common patterns
 *
 * # Performance
 *
 * - Bincode: 5-10x faster than JSON, ~80% smaller payloads
 * - SIMD JSON: 2-3x faster than std JSON for large payloads (>1KB)
 * - Zero-copy deserialization where possible
 *
 * # Use Cases
 *
 * - **Bincode**: Internal kernel-to-kernel IPC, process state serialization
 * - **JSON**: External APIs, configuration files, debugging output
 * - **Serde helpers**: SystemTime serialization, optional field skipping
 */

pub mod bincode;
pub mod json;
pub mod serde;

// Re-export commonly used functions
pub use bincode::{from_slice as from_bincode, serialized_size, to_vec as to_bincode};
pub use json::{from_slice as from_json, to_string as to_json_string, to_vec as to_json};
pub use serde::{
    is_zero_u32, is_zero_u64, is_zero_usize, skip_serializing_none, system_time_micros,
};

