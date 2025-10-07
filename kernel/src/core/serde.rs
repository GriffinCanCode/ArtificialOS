/// Serde helper functions for custom serialization/deserialization
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::{SystemTime, UNIX_EPOCH};

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

/// Skip serializing if value is default (for use with skip_serializing_if)
pub fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

/// Skip serializing if Option is None
pub fn is_none<T>(value: &Option<T>) -> bool {
    value.is_none()
}

/// Skip serializing if Vec is empty
pub fn is_empty_vec<T>(value: &Vec<T>) -> bool {
    value.is_empty()
}

/// Skip serializing if value is zero
pub fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

/// Skip serializing if value is zero
pub fn is_zero_usize(value: &usize) -> bool {
    *value == 0
}

/// Skip serializing if value is false
pub fn is_false(value: &bool) -> bool {
    !value
}

/// Serialize PathBuf as string
pub mod pathbuf_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::path::PathBuf;

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
pub mod optional_pathbuf_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(with = "system_time_micros")]
        time: SystemTime,
        #[serde(
            with = "optional_system_time_micros",
            skip_serializing_if = "is_none"
        )]
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
        assert!(is_zero_u64(&0));
        assert!(!is_zero_u64(&1));
        assert!(is_false(&false));
        assert!(!is_false(&true));
    }
}
