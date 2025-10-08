/*!
 * Optimized Binary Serialization with bincode
 * High-performance binary serialization for internal IPC operations
 */

use serde::{de::DeserializeOwned, Serialize};

/// Result type for bincode operations
pub type BincodeResult<T> = Result<T, BincodeError>;

/// Binary serialization errors
#[derive(Debug, thiserror::Error)]
pub enum BincodeError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

// ============================================================================
// Serialization Functions
// ============================================================================

/// Serialize to binary bytes using bincode
///
/// This is 5-10x faster than JSON for binary data and produces much smaller payloads.
/// Use this for internal kernel-to-kernel IPC where human-readability is not required.
#[inline]
pub fn to_vec<T: Serialize>(value: &T) -> BincodeResult<Vec<u8>> {
    bincode::serialize(value).map_err(|e| BincodeError::Serialization(e.to_string()))
}

/// Deserialize from binary bytes using bincode
///
/// Matches the output of `to_vec`. Use for internal kernel-to-kernel IPC.
#[inline]
pub fn from_slice<T: DeserializeOwned>(bytes: &[u8]) -> BincodeResult<T> {
    bincode::deserialize(bytes).map_err(|e| BincodeError::Deserialization(e.to_string()))
}

/// Get the serialized size of a value without actually serializing it
///
/// Useful for pre-allocating buffers or checking size limits.
#[inline]
pub fn serialized_size<T: Serialize>(value: &T) -> BincodeResult<u64> {
    bincode::serialized_size(value).map_err(|e| BincodeError::Serialization(e.to_string()))
}

// ============================================================================
// IPC-Specific Helpers
// ============================================================================

/// Serialize IPC message using bincode (for internal transfers)
///
/// This is significantly faster than JSON for binary payloads like:
/// - Pipe data (Vec<u8>)
/// - Queue messages with binary content
/// - Memory-mapped data structures
#[inline]
pub fn serialize_ipc_message<T: Serialize>(message: &T) -> BincodeResult<Vec<u8>> {
    to_vec(message)
}

/// Deserialize IPC message using bincode
#[inline]
pub fn deserialize_ipc_message<T: DeserializeOwned>(bytes: &[u8]) -> BincodeResult<T> {
    from_slice(bytes)
}

/// Serialize with size prefix (for streaming scenarios)
///
/// Format: [4-byte length][bincode data]
/// This allows reading messages from a stream without knowing the size upfront.
pub fn to_vec_with_size<T: Serialize>(value: &T) -> BincodeResult<Vec<u8>> {
    let data = to_vec(value)?;
    let len = data.len() as u32;

    let mut result = Vec::with_capacity(4 + data.len());
    result.extend_from_slice(&len.to_le_bytes());
    result.extend_from_slice(&data);

    Ok(result)
}

/// Deserialize from size-prefixed format
///
/// Reads the 4-byte length prefix and then deserializes that many bytes.
pub fn from_slice_with_size<T: DeserializeOwned>(bytes: &[u8]) -> BincodeResult<T> {
    if bytes.len() < 4 {
        return Err(BincodeError::Deserialization(
            "Buffer too small for size prefix".to_string(),
        ));
    }

    let len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;

    if bytes.len() < 4 + len {
        return Err(BincodeError::Deserialization(format!(
            "Buffer too small: expected {} bytes, got {}",
            4 + len,
            bytes.len()
        )));
    }

    from_slice(&bytes[4..4 + len])
}

// ============================================================================
// Performance Comparison Utilities
// ============================================================================

/// Compare bincode vs JSON serialization sizes
///
/// Returns (bincode_size, json_size, compression_ratio)
#[cfg(test)]
pub fn compare_with_json<T: Serialize>(value: &T) -> (usize, usize, f64) {
    let bincode_bytes = to_vec(value).unwrap();
    let json_bytes = serde_json::to_vec(value).unwrap();

    let bincode_size = bincode_bytes.len();
    let json_size = json_bytes.len();
    let ratio = json_size as f64 / bincode_size as f64;

    (bincode_size, json_size, ratio)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestMessage {
        id: u64,
        from: u32,
        to: u32,
        data: Vec<u8>,
        timestamp: u64,
    }

    #[test]
    fn test_basic_serialization() {
        let msg = TestMessage {
            id: 42,
            from: 1,
            to: 2,
            data: vec![1, 2, 3, 4, 5],
            timestamp: 1234567890,
        };

        let bytes = to_vec(&msg).unwrap();
        let deserialized: TestMessage = from_slice(&bytes).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_large_binary_payload() {
        let msg = TestMessage {
            id: 42,
            from: 1,
            to: 2,
            data: vec![0u8; 10000], // 10KB binary data
            timestamp: 1234567890,
        };

        let bytes = to_vec(&msg).unwrap();
        let deserialized: TestMessage = from_slice(&bytes).unwrap();
        assert_eq!(msg, deserialized);

        // Bincode should be much smaller than JSON for binary data
        assert!(bytes.len() < 11000); // ~10KB + small overhead
    }

    #[test]
    fn test_size_prefixed_serialization() {
        let msg = TestMessage {
            id: 42,
            from: 1,
            to: 2,
            data: vec![1, 2, 3],
            timestamp: 1234567890,
        };

        let bytes = to_vec_with_size(&msg).unwrap();
        assert!(bytes.len() >= 4); // At least 4 bytes for size prefix

        let deserialized: TestMessage = from_slice_with_size(&bytes).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_serialized_size() {
        let msg = TestMessage {
            id: 42,
            from: 1,
            to: 2,
            data: vec![1, 2, 3, 4, 5],
            timestamp: 1234567890,
        };

        let size = serialized_size(&msg).unwrap();
        let bytes = to_vec(&msg).unwrap();

        assert_eq!(size as usize, bytes.len());
    }

    #[test]
    fn test_bincode_vs_json_compression() {
        let msg = TestMessage {
            id: 42,
            from: 1,
            to: 2,
            data: vec![0u8; 1000], // 1KB binary data
            timestamp: 1234567890,
        };

        let (bincode_size, json_size, ratio) = compare_with_json(&msg);

        println!("Bincode: {} bytes", bincode_size);
        println!("JSON: {} bytes", json_size);
        println!("Compression ratio: {:.2}x", ratio);

        // JSON should be significantly larger for binary data
        assert!(
            ratio > 2.0,
            "Expected JSON to be >2x larger, got {:.2}x",
            ratio
        );
    }

    #[test]
    fn test_error_handling() {
        let invalid_bytes = vec![0xFF, 0xFF, 0xFF];
        let result: BincodeResult<TestMessage> = from_slice(&invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_size_prefix_errors() {
        // Too small buffer
        let result: BincodeResult<TestMessage> = from_slice_with_size(&[0, 0]);
        assert!(result.is_err());

        // Size prefix indicates more data than available
        let result: BincodeResult<TestMessage> = from_slice_with_size(&[100, 0, 0, 0, 1, 2]);
        assert!(result.is_err());
    }
}
