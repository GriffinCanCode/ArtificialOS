/*!
 * Serde Helper Functions & Modern Patterns (2025)
 *
 * Production-grade serialization using serde_with 3.0+ best practices.
 *
 * # Philosophy
 * - **Type-safe conversions**: Use serde_as for compile-time guarantees
 * - **Zero boilerplate**: Derive-based approach over custom modules
 * - **Validation built-in**: Deserialize with invariant enforcement
 * - **Performance**: Inline, zero-cost abstractions
 *
 * # Quick Reference
 *
 * ```ignore
 * use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};
 *
 * #[serde_as]
 * #[skip_serializing_none]
 * #[derive(Serialize, Deserialize)]
 * struct Event {
 *     // SystemTime as microseconds
 *     #[serde_as(as = "DurationMicroSeconds<u64>")]
 *     timestamp: SystemTime,
 *
 *     // PathBuf as string
 *     #[serde_as(as = "DisplayFromStr")]
 *     path: PathBuf,
 *
 *     // Skip if None
 *     optional_field: Option<String>,
 *
 *     // Skip if empty
 *     #[serde(skip_serializing_if = "Vec::is_empty", default)]
 *     tags: Vec<String>,
 * }
 * ```
 */

use serde::{Deserialize, Deserializer, Serializer};
use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};

// ============================================================================
// Re-exports (Modern serde_with patterns)
// ============================================================================

pub use serde_with::{
    serde_as, skip_serializing_none, DisplayFromStr, DurationMicroSeconds, DurationMilliSeconds,
    DurationSeconds, DurationSecondsWithFrac, TimestampMicroSeconds, TimestampMilliSeconds,
    TimestampSeconds,
};

// Re-export core serde derives
pub use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

// ============================================================================
// Skip Serializing Predicates (for #[serde(skip_serializing_if = "...")])
// ============================================================================

/// Skip serializing if value is default
#[inline]
pub fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

/// Skip serializing if Option is None
#[inline]
pub const fn is_none<T>(value: &Option<T>) -> bool {
    value.is_none()
}

/// Skip serializing if Vec is empty
#[inline]
pub fn is_empty_vec<T>(value: &Vec<T>) -> bool {
    value.is_empty()
}

/// Skip serializing if slice is empty
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

// Numeric zero checks (commonly used)
#[inline]
pub const fn is_zero_u8(value: &u8) -> bool {
    *value == 0
}

#[inline]
pub const fn is_zero_u16(value: &u16) -> bool {
    *value == 0
}

#[inline]
pub const fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

#[inline]
pub const fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

#[inline]
pub const fn is_zero_usize(value: &usize) -> bool {
    *value == 0
}

#[inline]
pub const fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

#[inline]
pub const fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

// Boolean checks
#[inline]
pub const fn is_false(value: &bool) -> bool {
    !*value
}

#[inline]
pub const fn is_true(value: &bool) -> bool {
    *value
}

// ============================================================================
// Validation Deserializers (for #[serde(deserialize_with = "...")])
// ============================================================================

/// Deserialize and validate that a u32 is non-zero
///
/// Use `NonZeroU32` type directly for stronger guarantees in new code.
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

/// Deserialize directly to NonZeroU32 (type-safe alternative)
pub fn deserialize_nonzero_u32_typed<'de, D>(deserializer: D) -> Result<NonZeroU32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u32::deserialize(deserializer)?;
    NonZeroU32::new(value).ok_or_else(|| serde::de::Error::custom("value must be non-zero"))
}

/// Deserialize and validate that a u64 is non-zero
pub fn deserialize_nonzero_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u64::deserialize(deserializer)?;
    if value == 0 {
        return Err(serde::de::Error::custom("value must be non-zero"));
    }
    Ok(value)
}

/// Deserialize directly to NonZeroU64 (type-safe alternative)
pub fn deserialize_nonzero_u64_typed<'de, D>(deserializer: D) -> Result<NonZeroU64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u64::deserialize(deserializer)?;
    NonZeroU64::new(value).ok_or_else(|| serde::de::Error::custom("value must be non-zero"))
}

/// Deserialize and validate that a usize is non-zero
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

/// Deserialize directly to NonZeroUsize (type-safe alternative)
pub fn deserialize_nonzero_usize_typed<'de, D>(deserializer: D) -> Result<NonZeroUsize, D::Error>
where
    D: Deserializer<'de>,
{
    let value = usize::deserialize(deserializer)?;
    NonZeroUsize::new(value).ok_or_else(|| serde::de::Error::custom("value must be non-zero"))
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
// Bounded Value Deserializers
// ============================================================================

/// Deserialize a u32 with maximum bound validation
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

/// Deserialize a u64 with maximum bound validation
pub fn deserialize_bounded_u64<'de, D>(max: u64) -> impl FnOnce(D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    move |deserializer: D| {
        let value = u64::deserialize(deserializer)?;
        if value > max {
            return Err(serde::de::Error::custom(format!(
                "value {} exceeds maximum {}",
                value, max
            )));
        }
        Ok(value)
    }
}

/// Deserialize a u8 within a range [min, max]
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

/// Deserialize a u32 within a range [min, max]
pub fn deserialize_ranged_u32<'de, D>(min: u32, max: u32) -> impl FnOnce(D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    move |deserializer: D| {
        let value = u32::deserialize(deserializer)?;
        if value < min || value > max {
            return Err(serde::de::Error::custom(format!(
                "value {} is outside range [{}, {}]",
                value, min, max
            )));
        }
        Ok(value)
    }
}

// ============================================================================
// Modern Serialization Modules (Using serde_with internally)
// ============================================================================

/// SystemTime as microseconds since UNIX epoch
///
/// Modern replacement: Just use `#[serde_as(as = "DurationMicroSeconds<u64>")]`
pub mod system_time_micros {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[inline]
    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(duration.as_micros() as u64)
    }

    #[inline]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let micros = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_micros(micros))
    }
}

/// Option<SystemTime> as Option<microseconds>
///
/// Modern replacement: `#[serde_as(as = "Option<DurationMicroSeconds<u64>>")]`
pub mod optional_system_time_micros {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[inline]
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

    #[inline]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<u64>::deserialize(deserializer)?;
        Ok(opt.map(|micros| UNIX_EPOCH + std::time::Duration::from_micros(micros)))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::time::SystemTime;

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct ModernStruct {
        #[serde_as(as = "DurationMicroSeconds<u64>")]
        timestamp: SystemTime,

        #[serde_as(as = "Option<DurationMicroSeconds<u64>>")]
        #[serde(skip_serializing_if = "is_none")]
        optional_time: Option<SystemTime>,

        #[serde(skip_serializing_if = "is_zero_u64")]
        counter: u64,
    }

    #[test]
    fn test_modern_serde_as_pattern() {
        let now = SystemTime::now();
        let data = ModernStruct {
            timestamp: now,
            optional_time: Some(now),
            counter: 42,
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: ModernStruct = serde_json::from_str(&json).unwrap();

        // Times should match within microsecond precision
        let original_micros = data
            .timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        let deserialized_micros = deserialized
            .timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        assert_eq!(original_micros, deserialized_micros);
    }

    #[test]
    fn test_skip_serializing_helpers() {
        assert!(is_default(&0u64));
        assert!(!is_default(&1u64));
        assert!(is_none::<String>(&None));
        assert!(!is_none(&Some(1).into()));
        assert!(is_empty_vec::<i32>(&vec![]));
        assert!(!is_empty_vec(&vec![1]));
        assert!(is_empty_slice::<i32>(&[]));
        assert!(!is_empty_slice(&[1]));
        assert!(is_empty_string(&String::new().into()));
        assert!(!is_empty_string(&"test".to_string().into()));
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
    fn test_nonzero_typed_deserialization() {
        #[derive(Deserialize)]
        struct TypedTest {
            #[serde(deserialize_with = "deserialize_nonzero_u32_typed")]
            value: NonZeroU32,
        }

        let json = r#"{"value": 42}"#;
        let result: TypedTest = serde_json::from_str(json).unwrap();
        assert_eq!(result.value.get(), 42);

        let json_zero = r#"{"value": 0}"#;
        let result: Result<TypedTest, _> = serde_json::from_str(json_zero);
        assert!(result.is_err());
    }

    #[test]
    fn test_legacy_system_time_modules() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct LegacyStruct {
            #[serde(with = "system_time_micros")]
            time: SystemTime,
        }

        let now = SystemTime::now();
        let data = LegacyStruct { time: now };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: LegacyStruct = serde_json::from_str(&json).unwrap();

        let original_micros = data
            .time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        let deserialized_micros = deserialized
            .time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        assert_eq!(original_micros, deserialized_micros);
    }
}
