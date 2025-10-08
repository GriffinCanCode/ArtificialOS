/*!
 * Serde helper functions for custom serialization/deserialization
 * Modernized with serde_with patterns and type-safe helpers
 */

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// Re-export common serde_with utilities for convenience
pub use serde_with::{serde_as, skip_serializing_none, DisplayFromStr, DurationMicroSeconds};

/// Serialize SystemTime as microseconds since UNIX epoch
pub mod system_time_micros {
    use super::*;

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(duration.as_micros() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let micros = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_micros(micros))
    }
}

/// Serialize Option<SystemTime> as Option<microseconds>
/// Preferred: Use serde_with::serde_as with Option<DurationMicroSeconds> for new code
pub mod optional_system_time_micros {
    use super::*;

    pub fn serialize<S>(time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match time {
            Some(t) => {
                let duration = t
                    .duration_since(UNIX_EPOCH)
                    .map_err(serde::ser::Error::custom)?;
                serializer.serialize_some(&(duration.as_micros() as u64))
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<u64>::deserialize(deserializer)?;
        Ok(opt.map(|micros| UNIX_EPOCH + std::time::Duration::from_micros(micros)))
    }
}

/// Serialize PathBuf as string
/// Preferred: Use serde_with::DisplayFromStr for new code
pub mod pathbuf_string {
    use super::*;

    pub fn serialize<S>(path: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        path.to_string_lossy().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(PathBuf::from(s))
    }
}

/// Serialize Option<PathBuf> as Option<String>
/// Preferred: Use serde_with::serde_as with Option<DisplayFromStr> for new code
pub mod optional_pathbuf_string {
    use super::*;

    pub fn serialize<S>(path: &Option<PathBuf>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match path {
            Some(p) => serializer.serialize_some(&p.to_string_lossy().to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        Ok(opt.map(PathBuf::from))
    }
}

// ============================================================================
// Skip serializing helpers (for use with skip_serializing_if)
// ============================================================================

/// Skip serializing if value is default (generic)
#[inline]
pub fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

/// Skip serializing if Option is None
/// Note: Prefer using `skip_serializing_none` from serde_with for new code
#[inline]
pub fn is_none<T>(value: &Option<T>) -> bool {
    value.is_none()
}

/// Skip serializing if Vec is empty
#[inline]
pub fn is_empty_vec<T>(value: &Vec<T>) -> bool {
    value.is_empty()
}

/// Skip serializing if slice is empty (more general than Vec)
#[inline]
pub fn is_empty_slice<T>(value: &[T]) -> bool {
    value.is_empty()
}

/// Skip serializing if String is empty
#[inline]
pub fn is_empty_string(value: &String) -> bool {
    value.is_empty()
}

/// Skip serializing if str is empty
#[inline]
pub fn is_empty_str(value: &str) -> bool {
    value.is_empty()
}

/// Skip serializing if value is zero (u8)
#[inline]
pub fn is_zero_u8(value: &u8) -> bool {
    *value == 0
}

/// Skip serializing if value is zero (u16)
#[inline]
pub fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}

/// Skip serializing if value is zero (u32)
#[inline]
pub fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

/// Skip serializing if value is zero (u64)
#[inline]
pub fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

/// Skip serializing if value is zero (usize)
#[inline]
pub fn is_zero_usize(value: &usize) -> bool {
    *value == 0
}

/// Skip serializing if value is zero (i32)
#[inline]
pub fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

/// Skip serializing if value is zero (i64)
#[inline]
pub fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

/// Skip serializing if value is false
#[inline]
pub fn is_false(value: &bool) -> bool {
    !value
}

/// Skip serializing if value is true
#[inline]
pub fn is_true(value: &bool) -> bool {
    *value
}

// ============================================================================
// Validation helpers (for use with deserialize_with)
// ============================================================================

/// Deserialize and validate that a number is non-zero
pub fn deserialize_nonzero_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u32::deserialize(deserializer)?;
    if value == 0 {
        return Err(serde::de::Error::custom("value must be non-zero"));
    }
    Ok(value)
}

/// Deserialize and validate that a number is non-zero
pub fn deserialize_nonzero_usize<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let value = usize::deserialize(deserializer)?;
    if value == 0 {
        return Err(serde::de::Error::custom("value must be non-zero"));
    }
    Ok(value)
}

/// Deserialize and validate that a string is not empty
pub fn deserialize_nonempty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.is_empty() {
        return Err(serde::de::Error::custom("string must not be empty"));
    }
    Ok(value)
}

/// Deserialize and validate that a vec is not empty
pub fn deserialize_nonempty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let value = Vec::<T>::deserialize(deserializer)?;
    if value.is_empty() {
        return Err(serde::de::Error::custom("vec must not be empty"));
    }
    Ok(value)
}

// ============================================================================
// Bounded value helpers
// ============================================================================

/// Deserialize a value with a maximum bound
pub fn deserialize_bounded_u32<'de, D>(max: u32) -> impl FnOnce(D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    move |deserializer: D| {
        let value = u32::deserialize(deserializer)?;
        if value > max {
            return Err(serde::de::Error::custom(format!(
                "value {} exceeds maximum {}",
                value, max
            )));
        }
        Ok(value)
    }
}

/// Deserialize a value within a range
pub fn deserialize_ranged_u8<'de, D>(min: u8, max: u8) -> impl FnOnce(D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    move |deserializer: D| {
        let value = u8::deserialize(deserializer)?;
        if value < min || value > max {
            return Err(serde::de::Error::custom(format!(
                "value {} is outside range [{}, {}]",
                value, min, max
            )));
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(with = "system_time_micros")]
        time: SystemTime,
        #[serde(with = "optional_system_time_micros", skip_serializing_if = "is_none")]
        optional_time: Option<SystemTime>,
    }

    #[test]
    fn test_system_time_serialization() {
        let now = SystemTime::now();
        let test = TestStruct {
            time: now,
            optional_time: Some(now),
        };

        let json = serde_json::to_string(&test).unwrap();
        let deserialized: TestStruct = serde_json::from_str(&json).unwrap();

        // Times should be equal within microsecond precision
        let original_micros = now.duration_since(UNIX_EPOCH).unwrap().as_micros();
        let deserialized_micros = deserialized
            .time
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros();
        assert_eq!(original_micros, deserialized_micros);
    }

    #[test]
    fn test_skip_serializing_helpers() {
        assert!(is_default(&0u64));
        assert!(!is_default(&1u64));
        assert!(is_none::<String>(&None));
        assert!(!is_none(&Some(1)));
        assert!(is_empty_vec::<i32>(&vec![]));
        assert!(!is_empty_vec(&vec![1]));
        assert!(is_empty_slice::<i32>(&[]));
        assert!(!is_empty_slice(&[1]));
        assert!(is_empty_string(&String::new()));
        assert!(!is_empty_string(&"test".to_string()));
        assert!(is_zero_u8(&0));
        assert!(is_zero_u16(&0));
        assert!(is_zero_u32(&0));
        assert!(is_zero_u64(&0));
        assert!(is_zero_usize(&0));
        assert!(!is_zero_u64(&1));
        assert!(is_false(&false));
        assert!(!is_false(&true));
        assert!(is_true(&true));
        assert!(!is_true(&false));
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ValidationTest {
        #[serde(deserialize_with = "deserialize_nonzero_u32")]
        nonzero: u32,
        #[serde(deserialize_with = "deserialize_nonempty_string")]
        name: String,
    }

    #[test]
    fn test_validation_success() {
        let json = r#"{"nonzero": 42, "name": "test"}"#;
        let result: ValidationTest = serde_json::from_str(json).unwrap();
        assert_eq!(result.nonzero, 42);
        assert_eq!(result.name, "test");
    }

    #[test]
    fn test_validation_failure_zero() {
        let json = r#"{"nonzero": 0, "name": "test"}"#;
        let result: Result<ValidationTest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_failure_empty_string() {
        let json = r#"{"nonzero": 42, "name": ""}"#;
        let result: Result<ValidationTest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_pathbuf_serialization() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct PathTest {
            #[serde(with = "pathbuf_string")]
            path: PathBuf,
        }

        let test = PathTest {
            path: PathBuf::from("/tmp/test"),
        };
        let json = serde_json::to_string(&test).unwrap();
        let deserialized: PathTest = serde_json::from_str(&json).unwrap();
        assert_eq!(test, deserialized);
    }
}
