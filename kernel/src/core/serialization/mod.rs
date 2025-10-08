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

// Re-export bincode functions (zero-copy optimized)
pub use bincode::{
    deserialize_ipc_message as deserialize_bincode_ipc, from_bytes as from_bincode_bytes,
    from_slice as from_bincode, from_slice_with_header, serialize_ipc_message as serialize_bincode_ipc,
    serialized_size, to_bytes_pooled as to_bincode_pooled, to_vec as to_bincode,
    to_vec_with_header, BincodeError, BincodeResult,
};

// Re-export JSON functions (adaptive SIMD)
pub use json::{
    deserialize_ipc_message as deserialize_json_ipc, deserialize_syscall_input,
    from_bytes as from_json_bytes, from_slice as from_json, from_str as from_json_str,
    get_simd_threshold, serialize_ipc_message as serialize_json_ipc, serialize_syscall_result,
    serialize_syscall_result_pooled, serialize_vfs_batch, set_simd_threshold,
    to_bytes_pooled as to_json_pooled, to_string as to_json_string,
    to_string_pretty as to_json_pretty, to_vec as to_json, would_use_simd, JsonError, JsonResult,
};

// Re-export serde helpers (modern patterns)
pub use serde::{
    deserialize_nonzero_u32, deserialize_nonzero_u32_typed, deserialize_nonzero_u64,
    deserialize_nonzero_u64_typed, deserialize_nonzero_usize, deserialize_nonzero_usize_typed,
    deserialize_nonempty_string, deserialize_nonempty_vec, is_empty_slice, is_empty_str,
    is_empty_string, is_empty_vec, is_false, is_none, is_true, is_zero_i32, is_zero_i64,
    is_zero_u32, is_zero_u64, is_zero_u8, is_zero_usize, optional_system_time_micros, serde_as,
    skip_serializing_none, system_time_micros, DisplayFromStr, DurationMicroSeconds,
    SerdeDeserialize, SerdeSerialize,
};

