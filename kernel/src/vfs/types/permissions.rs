/*!
 * VFS Permissions
 * Unix-style file permissions with validation
 */

use serde::{Deserialize, Deserializer, Serialize};

/// File permissions (Unix-style) with validation
///
/// # Performance
/// - Packed C layout for efficient permission checks
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(deserialize_with = "deserialize_permission_mode")]
    pub mode: u32,
}

impl Permissions {
    /// Create permissions with mode validation (masks to valid bits)
    #[inline]
    #[must_use]
    pub const fn new(mode: u32) -> Self {
        Self {
            mode: mode & 0o7777,
        }
    }

    /// Create read-only permissions (0o444)
    #[inline]
    #[must_use]
    pub const fn readonly() -> Self {
        Self { mode: 0o444 }
    }

    /// Create read-write permissions (0o644)
    #[inline]
    #[must_use]
    pub const fn readwrite() -> Self {
        Self { mode: 0o644 }
    }

    /// Create executable permissions (0o755)
    #[inline]
    #[must_use]
    pub const fn executable() -> Self {
        Self { mode: 0o755 }
    }

    /// Check if permissions are read-only (no write bits set)
    ///
    /// # Performance
    /// Hot path - frequently called in VFS operations
    #[inline(always)]
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        self.mode & 0o200 == 0
    }

    /// Set read-only mode by clearing all write bits
    ///
    /// # Performance
    /// Hot path - called during permission modifications
    #[inline(always)]
    pub fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            self.mode &= !0o222;
        } else {
            self.mode |= 0o200;
        }
    }

    /// Check if any execute bit is set
    ///
    /// # Performance
    /// Hot path - frequently checked in process execution
    #[inline(always)]
    #[must_use]
    pub const fn is_executable(&self) -> bool {
        self.mode & 0o111 != 0
    }

    /// Get user permissions (rwx)
    #[inline]
    #[must_use]
    pub const fn user_permissions(&self) -> u32 {
        (self.mode >> 6) & 0o7
    }

    /// Get group permissions (rwx)
    #[inline]
    #[must_use]
    pub const fn group_permissions(&self) -> u32 {
        (self.mode >> 3) & 0o7
    }

    /// Get other permissions (rwx)
    #[inline]
    #[must_use]
    pub const fn other_permissions(&self) -> u32 {
        self.mode & 0o7
    }
}

/// Deserialize and validate permission mode (must be <= 0o7777)
fn deserialize_permission_mode<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let mode = u32::deserialize(deserializer)?;
    if mode > 0o7777 {
        return Err(serde::de::Error::custom(format!(
            "invalid permission mode: 0o{:o} exceeds maximum 0o7777",
            mode
        ).into()));
    }
    Ok(mode)
}

impl Default for Permissions {
    fn default() -> Self {
        Self::readwrite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions() {
        let mut perms = Permissions::readwrite();
        assert!(!perms.is_readonly());
        assert_eq!(perms.mode, 0o644);

        perms.set_readonly(true);
        assert!(perms.is_readonly());

        perms.set_readonly(false);
        assert!(!perms.is_readonly());

        // Test validation
        let perms = Permissions::new(0o12777); // Should mask to 0o2777
        assert_eq!(perms.mode, 0o2777);

        // Test executable
        let perms = Permissions::executable();
        assert!(perms.is_executable());
        assert_eq!(perms.mode, 0o755);
    }

    #[test]
    fn test_permission_components() {
        let perms = Permissions::new(0o754);
        assert_eq!(perms.user_permissions(), 0o7); // rwx
        assert_eq!(perms.group_permissions(), 0o5); // r-x
        assert_eq!(perms.other_permissions(), 0o4); // r--
    }

    #[test]
    fn test_permission_serialization() {
        let perms = Permissions::readwrite();
        let json = serde_json::to_string(&perms).unwrap();
        let deserialized: Permissions = serde_json::from_str(&json).unwrap();
        assert_eq!(perms, deserialized);

        // Test invalid mode
        let json = r#"{"mode": 99999}"#;
        let result: Result<Permissions, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
