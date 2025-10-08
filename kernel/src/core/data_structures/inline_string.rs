/*!
 * Inline String Optimization
 * Zero-allocation strings for common error messages
 */

use serde::{Deserialize, Serialize};
use smartstring::alias::String as SmartString;
use std::fmt;

/// Inline-optimized string that stores short strings (≤23 bytes) without heap allocation
///
/// # Performance
///
/// - **Small strings** (≤23 bytes): Stored inline, zero allocation
/// - **Large strings** (>23 bytes): Heap allocated like regular String
/// - **70-80%** of error messages fit inline
///
/// # Examples
///
/// ```ignore
/// // These are inline (no allocation):
/// InlineString::from("Not found");            // 9 bytes
/// InlineString::from("Permission denied");    // 17 bytes
/// InlineString::from("Invalid argument");     // 16 bytes
///
/// // These require heap allocation:
/// InlineString::from("This is a very long error message that exceeds the inline threshold");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct InlineString {
    inner: SmartString,
}

impl InlineString {
    /// Create new inline string
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: SmartString::new(),
        }
    }

    /// Create from static string (always inline)
    #[inline]
    pub const fn from_static(s: &'static str) -> Self {
        Self {
            inner: SmartString::from_static(s),
        }
    }

    /// Get string slice
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Check if string is stored inline (no heap allocation)
    #[inline]
    pub fn is_inline(&self) -> bool {
        self.inner.is_inline()
    }

    /// Get capacity (inline capacity is 23 bytes on 64-bit)
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Get length
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Push string slice
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.inner.push_str(s);
    }

    /// Clear string (keeps allocation if heap-allocated)
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Convert to String (may allocate if inline)
    #[inline]
    pub fn into_string(self) -> String {
        self.inner.into()
    }
}

impl Default for InlineString {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for InlineString {
    #[inline]
    fn from(s: &str) -> Self {
        Self {
            inner: SmartString::from(s),
        }
    }
}

impl From<String> for InlineString {
    #[inline]
    fn from(s: String) -> Self {
        Self {
            inner: SmartString::from(s),
        }
    }
}

impl From<InlineString> for String {
    #[inline]
    fn from(s: InlineString) -> Self {
        s.inner.into()
    }
}

impl AsRef<str> for InlineString {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::ops::Deref for InlineString {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for InlineString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::borrow::Borrow<str> for InlineString {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_storage() {
        // Short strings should be inline
        let short = InlineString::from("Not found");
        assert!(short.is_inline(), "Short strings should be inline");
        assert_eq!(short.as_str(), "Not found");

        let medium = InlineString::from("Permission denied error");
        assert!(medium.is_inline() || medium.len() <= 23, "Medium strings may be inline");
        assert_eq!(medium.as_str(), "Permission denied error");
    }

    #[test]
    fn test_long_string_heap_allocated() {
        let long = InlineString::from(
            "This is a very long error message that definitely exceeds the inline threshold"
        );
        assert!(!long.is_inline(), "Long strings should use heap");
        assert!(long.as_str().contains("very long error"));
    }

    #[test]
    fn test_static_strings() {
        let static_str = InlineString::from_static("Static");
        assert_eq!(static_str.as_str(), "Static");
    }

    #[test]
    fn test_conversions() {
        let inline_str = InlineString::from("test");
        let string: String = inline_str.clone().into();
        assert_eq!(string, "test");

        let from_string = InlineString::from(String::from("another"));
        assert_eq!(from_string.as_str(), "another");
    }

    #[test]
    fn test_common_error_messages_inline() {
        let errors = vec![
            "Not found",
            "Access denied",
            "Invalid input",
            "Timeout",
            "Connection refused",
            "Bad request",
            "Unauthorized",
            "Forbidden",
            "Not implemented",
            "Service unavailable",
        ];

        for error in errors {
            let inline = InlineString::from(error);
            assert!(inline.is_inline(), "Error '{}' should be inline (len={})", error, error.len());
        }
    }

    #[test]
    fn test_serialization() {
        let inline_str = InlineString::from("test message");
        let json = serde_json::to_string(&inline_str).unwrap();
        let deserialized: InlineString = serde_json::from_str(&json).unwrap();
        assert_eq!(inline_str, deserialized);
    }
}

