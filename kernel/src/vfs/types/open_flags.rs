/*!
 * VFS Open Flags and Mode
 * Flags and modes for file opening operations
 */

use super::errors::VfsError;
use super::permissions::Permissions;
use crate::core::serialization::serde::{is_default, is_false};
use serde::{Deserialize, Serialize};

/// File open flags with optimized serialization (skips false values)
///
/// Using serde default and skip_serializing_if to create compact JSON representations.
/// Only true flags are serialized, reducing payload size significantly.
///
/// # Performance
/// - Packed C layout for efficient passing to VFS operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case", default, deny_unknown_fields)]
pub struct OpenFlags {
    #[serde(skip_serializing_if = "is_false")]
    pub read: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub write: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub append: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub truncate: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub create: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub create_new: bool,
}

impl OpenFlags {
    /// Create read-only flags
    #[inline]
    #[must_use]
    pub fn read_only() -> Self {
        Self {
            read: true,
            ..Default::default()
        }
    }

    /// Create write-only flags
    #[inline]
    #[must_use]
    pub fn write_only() -> Self {
        Self {
            write: true,
            ..Default::default()
        }
    }

    /// Create read-write flags
    #[inline]
    #[must_use]
    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            ..Default::default()
        }
    }

    /// Create flags for creating a new file (write + create)
    #[inline]
    #[must_use]
    pub fn create() -> Self {
        Self {
            write: true,
            create: true,
            ..Default::default()
        }
    }

    /// Create flags for creating a new file exclusively (write + create_new)
    #[inline]
    #[must_use]
    pub fn create_new() -> Self {
        Self {
            write: true,
            create_new: true,
            ..Default::default()
        }
    }

    /// Create flags for appending (write + append)
    #[inline]
    #[must_use]
    pub fn append_only() -> Self {
        Self {
            write: true,
            append: true,
            ..Default::default()
        }
    }

    /// Check if any write operation is possible
    #[inline]
    #[must_use]
    pub const fn is_writable(&self) -> bool {
        self.write || self.append
    }

    /// Check if this will create a file
    #[inline]
    #[must_use]
    pub const fn will_create(&self) -> bool {
        self.create || self.create_new
    }

    /// Convert from POSIX-style flags (O_RDONLY, O_WRONLY, O_RDWR, etc.)
    pub fn from_posix(flags: u32) -> Self {
        // Extract access mode from lower 2 bits
        let access_mode = flags & 0x0003;
        let read = access_mode == 0x0001 || access_mode == 0x0003;
        let write = access_mode == 0x0002 || access_mode == 0x0003;
        let append = flags & 0x0400 != 0;
        let truncate = flags & 0x0200 != 0;
        let create = flags & 0x0040 != 0;
        let create_new = flags & 0x0080 != 0;

        Self {
            read,
            write,
            append,
            truncate,
            create,
            create_new,
        }
    }

    /// Convert to POSIX-style flags
    pub fn to_posix(&self) -> u32 {
        let mut flags = 0u32;

        // Set access mode
        flags |= match (self.read, self.write) {
            (true, true) => 0x0003,   // O_RDWR
            (true, false) => 0x0001,  // O_RDONLY
            (false, true) => 0x0002,  // O_WRONLY
            (false, false) => 0x0001, // Default to O_RDONLY
        };

        if self.append {
            flags |= 0x0400; // O_APPEND
        }
        if self.truncate {
            flags |= 0x0200; // O_TRUNC
        }
        if self.create {
            flags |= 0x0040; // O_CREAT
        }
        if self.create_new {
            flags |= 0x0080; // O_EXCL
        }

        flags
    }

    /// Validate flag combinations
    #[must_use = "validation result must be checked"]
    pub fn validate(&self) -> Result<(), VfsError> {
        // Cannot have both create_new and create without create_new implying create
        if self.create_new && !self.write {
            return Err(VfsError::InvalidArgument(
                "create_new requires write flag".into(),
            ));
        }
        if self.truncate && !self.write {
            return Err(VfsError::InvalidArgument(
                "truncate requires write flag".into(),
            ));
        }
        if self.append && self.truncate {
            return Err(VfsError::InvalidArgument(
                "cannot use both append and truncate".into(),
            ));
        }
        Ok(())
    }
}

/// File open mode (for creation) with serde support
///
/// Specifies permissions for newly created files. Defaults to read-write (0o644).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenMode {
    #[serde(skip_serializing_if = "is_default", default)]
    pub permissions: Permissions,
}

impl OpenMode {
    /// Create mode with specified permissions
    #[inline]
    #[must_use]
    pub const fn new(mode: u32) -> Self {
        Self {
            permissions: Permissions::new(mode),
        }
    }

    /// Create mode with read-only permissions
    #[inline]
    #[must_use]
    pub const fn readonly() -> Self {
        Self {
            permissions: Permissions::readonly(),
        }
    }

    /// Create mode with read-write permissions
    #[inline]
    #[must_use]
    pub const fn readwrite() -> Self {
        Self {
            permissions: Permissions::readwrite(),
        }
    }

    /// Create mode with executable permissions
    #[inline]
    #[must_use]
    pub const fn executable() -> Self {
        Self {
            permissions: Permissions::executable(),
        }
    }
}

impl Default for OpenMode {
    fn default() -> Self {
        Self {
            permissions: Permissions::readwrite(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_flags() {
        let flags = OpenFlags::read_only();
        assert!(flags.read);
        assert!(!flags.write);
        assert!(!flags.is_writable());

        let flags = OpenFlags::write_only();
        assert!(!flags.read);
        assert!(flags.write);
        assert!(flags.is_writable());

        let flags = OpenFlags::read_write();
        assert!(flags.read);
        assert!(flags.write);

        // Test create helpers
        let flags = OpenFlags::create();
        assert!(flags.write);
        assert!(flags.create);
        assert!(flags.will_create());

        let flags = OpenFlags::append_only();
        assert!(flags.append);
        assert!(flags.is_writable());
    }

    #[test]
    fn test_open_flags_posix() {
        let flags = OpenFlags::from_posix(0x0001); // O_RDONLY
        assert!(flags.read);
        assert!(!flags.write);

        let flags = OpenFlags::from_posix(0x0002); // O_WRONLY
        assert!(!flags.read);
        assert!(flags.write);

        let flags = OpenFlags::from_posix(0x0003); // O_RDWR
        assert!(flags.read);
        assert!(flags.write);

        let flags = OpenFlags::from_posix(0x0042); // O_WRONLY | O_CREAT
        assert!(flags.write);
        assert!(flags.create);

        // Test round-trip
        let original = OpenFlags::read_write();
        let posix = original.to_posix();
        let restored = OpenFlags::from_posix(posix);
        // Note: restored may not be identical due to posix ambiguities
        assert_eq!(original.read, restored.read);
        assert_eq!(original.write, restored.write);
    }

    #[test]
    fn test_open_flags_serialization() {
        let flags = OpenFlags::read_only();
        let json = serde_json::to_string(&flags).unwrap();
        // Should only serialize true values
        assert!(json.contains("\"read\":true"));
        assert!(!json.contains("write"));

        let deserialized: OpenFlags = serde_json::from_str(&json).unwrap();
        assert_eq!(flags, deserialized);
    }

    #[test]
    fn test_open_flags_validation() {
        let flags = OpenFlags {
            write: true,
            create_new: true,
            ..Default::default()
        };
        assert!(flags.validate().is_ok());

        // Invalid: create_new without write
        let flags = OpenFlags {
            read: true,
            create_new: true,
            ..Default::default()
        };
        assert!(flags.validate().is_err());

        // Invalid: truncate without write
        let flags = OpenFlags {
            read: true,
            truncate: true,
            ..Default::default()
        };
        assert!(flags.validate().is_err());

        // Invalid: append with truncate
        let flags = OpenFlags {
            write: true,
            append: true,
            truncate: true,
            ..Default::default()
        };
        assert!(flags.validate().is_err());
    }

    #[test]
    fn test_open_mode() {
        let mode = OpenMode::default();
        assert_eq!(mode.permissions.mode, 0o644);

        let mode = OpenMode::readonly();
        assert!(mode.permissions.is_readonly());

        let mode = OpenMode::executable();
        assert!(mode.permissions.is_executable());
    }
}
