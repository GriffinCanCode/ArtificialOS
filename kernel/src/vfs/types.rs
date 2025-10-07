/*!
 * VFS Types
 * Shared types for filesystem operations with modern serde patterns
 */

use crate::core::serde::{is_default, is_false, is_zero_u64, serde_as, system_time_micros};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::time::SystemTime;
use thiserror::Error;

/// VFS operation result
///
/// # Must Use
/// VFS operations can fail and must be handled to prevent data loss
#[must_use = "VFS operations can fail and must be handled"]
pub type VfsResult<T> = Result<T, VfsError>;

/// VFS errors with structured, type-safe error handling
///
/// All error variants include context strings that should be non-empty.
/// Serialization uses tagged enum pattern for type safety.
#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "error", content = "details")]
pub enum VfsError {
    #[error("Not found: {0}")]
    NotFound(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Already exists: {0}")]
    AlreadyExists(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Permission denied: {0}")]
    PermissionDenied(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Not a directory: {0}")]
    NotADirectory(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Is a directory: {0}")]
    IsADirectory(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Invalid path: {0}")]
    InvalidPath(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("I/O error: {0}")]
    IoError(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Not supported: {0}")]
    NotSupported(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("Out of space")]
    OutOfSpace,

    #[error("Invalid argument: {0}")]
    InvalidArgument(#[serde(deserialize_with = "deserialize_nonempty_string")] String),

    #[error("File too large")]
    FileTooLarge,

    #[error("Read-only filesystem")]
    ReadOnly,

    #[error("Cross-device link")]
    CrossDevice,
}

/// Deserialize and validate non-empty string for error messages
fn deserialize_nonempty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Err(serde::de::Error::custom("error message must not be empty"));
    }
    Ok(s)
}

/// File type enumeration with complete serde support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    File,
    Directory,
    Symlink,
    #[serde(rename = "block_device")]
    BlockDevice,
    #[serde(rename = "char_device")]
    CharDevice,
    Fifo,
    Socket,
    Unknown,
}

impl Default for FileType {
    fn default() -> Self {
        Self::Unknown
    }
}

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
        )));
    }
    Ok(mode)
}

impl Default for Permissions {
    fn default() -> Self {
        Self::readwrite()
    }
}

/// File metadata with optimized serialization
///
/// Timestamps are serialized as microseconds since UNIX epoch for precision and efficiency.
/// Size and permissions are skipped when they are default values to reduce payload size.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Metadata {
    pub file_type: FileType,
    #[serde(skip_serializing_if = "is_zero_u64", default)]
    pub size: u64,
    #[serde(skip_serializing_if = "is_default", default)]
    pub permissions: Permissions,
    #[serde(with = "system_time_micros")]
    pub modified: SystemTime,
    #[serde(with = "system_time_micros")]
    pub accessed: SystemTime,
    #[serde(with = "system_time_micros")]
    pub created: SystemTime,
}

impl Metadata {
    /// Check if this is a directory
    ///
    /// # Performance
    /// Hot path - very frequently called in path resolution
    #[inline(always)]
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self.file_type, FileType::Directory)
    }

    /// Check if this is a regular file
    ///
    /// # Performance
    /// Hot path - very frequently called in file operations
    #[inline(always)]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self.file_type, FileType::File)
    }

    /// Check if this is a symbolic link
    ///
    /// # Performance
    /// Hot path - frequently called during path resolution
    #[inline(always)]
    #[must_use]
    pub const fn is_symlink(&self) -> bool {
        matches!(self.file_type, FileType::Symlink)
    }

    /// Check if this is a special file (device, fifo, socket)
    #[inline]
    #[must_use]
    pub const fn is_special(&self) -> bool {
        matches!(
            self.file_type,
            FileType::BlockDevice | FileType::CharDevice | FileType::Fifo | FileType::Socket
        )
    }
}

/// Directory entry with type-safe construction and validation
///
/// Entry names must be non-empty and cannot contain null bytes or path separators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "deserialize_valid_filename")]
    pub name: String,
    #[serde(skip_serializing_if = "is_default", default)]
    pub file_type: FileType,
}

impl Entry {
    /// Create a new directory entry with validation
    #[must_use = "validation result must be checked"]
    pub fn new(name: String, file_type: FileType) -> Result<Self, VfsError> {
        if name.is_empty() {
            return Err(VfsError::InvalidPath("entry name cannot be empty".into()));
        }
        if name.contains('\0') {
            return Err(VfsError::InvalidPath(
                "entry name cannot contain null bytes".into(),
            ));
        }
        if name.contains('/') || name.contains('\\') {
            return Err(VfsError::InvalidPath(
                "entry name cannot contain path separators".into(),
            ));
        }
        Ok(Self { name, file_type })
    }

    /// Create a new entry without validation (internal use)
    pub(crate) fn new_unchecked(name: String, file_type: FileType) -> Self {
        Self { name, file_type }
    }

    /// Create a file entry
    #[inline]
    #[must_use = "validation result must be checked"]
    pub fn file(name: String) -> Result<Self, VfsError> {
        Self::new(name, FileType::File)
    }

    /// Create a directory entry
    #[inline]
    #[must_use = "validation result must be checked"]
    pub fn directory(name: String) -> Result<Self, VfsError> {
        Self::new(name, FileType::Directory)
    }

    /// Check if this is a directory entry
    #[inline]
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self.file_type, FileType::Directory)
    }

    /// Check if this is a file entry
    #[inline]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self.file_type, FileType::File)
    }

    /// Validate entry name
    #[must_use = "validation result must be checked"]
    pub fn validate_name(name: &str) -> Result<(), VfsError> {
        if name.is_empty() {
            return Err(VfsError::InvalidPath("entry name cannot be empty".into()));
        }
        if name.contains('\0') {
            return Err(VfsError::InvalidPath(
                "entry name cannot contain null bytes".into(),
            ));
        }
        if name.contains('/') || name.contains('\\') {
            return Err(VfsError::InvalidPath(
                "entry name cannot contain path separators".into(),
            ));
        }
        Ok(())
    }
}

/// Deserialize and validate filename
fn deserialize_valid_filename<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let name = String::deserialize(deserializer)?;
    if name.is_empty() {
        return Err(serde::de::Error::custom("entry name cannot be empty"));
    }
    if name.contains('\0') {
        return Err(serde::de::Error::custom(
            "entry name cannot contain null bytes",
        ));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(serde::de::Error::custom(
            "entry name cannot contain path separators",
        ));
    }
    Ok(name)
}

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

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileType::File => write!(f, "file"),
            FileType::Directory => write!(f, "directory"),
            FileType::Symlink => write!(f, "symlink"),
            FileType::BlockDevice => write!(f, "block device"),
            FileType::CharDevice => write!(f, "char device"),
            FileType::Fifo => write!(f, "fifo"),
            FileType::Socket => write!(f, "socket"),
            FileType::Unknown => write!(f, "unknown"),
        }
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
    fn test_file_type_display() {
        assert_eq!(FileType::File.to_string(), "file");
        assert_eq!(FileType::Directory.to_string(), "directory");
        assert_eq!(FileType::Symlink.to_string(), "symlink");
    }

    #[test]
    fn test_file_type_serialization() {
        let ft = FileType::BlockDevice;
        let json = serde_json::to_string(&ft).unwrap();
        assert_eq!(json, "\"block_device\"");

        let deserialized: FileType = serde_json::from_str(&json).unwrap();
        assert_eq!(ft, deserialized);
    }

    #[test]
    fn test_metadata_helpers() {
        let metadata = Metadata {
            file_type: FileType::File,
            size: 100,
            permissions: Permissions::readwrite(),
            modified: SystemTime::now(),
            accessed: SystemTime::now(),
            created: SystemTime::now(),
        };

        assert!(metadata.is_file());
        assert!(!metadata.is_dir());
        assert!(!metadata.is_symlink());
        assert!(!metadata.is_special());

        let dir_metadata = Metadata {
            file_type: FileType::Directory,
            size: 0,
            permissions: Permissions::executable(),
            modified: SystemTime::now(),
            accessed: SystemTime::now(),
            created: SystemTime::now(),
        };

        assert!(dir_metadata.is_dir());
        assert!(!dir_metadata.is_file());
    }

    #[test]
    fn test_entry_helpers() {
        let entry = Entry::file("test.txt".to_string()).unwrap();
        assert_eq!(entry.name, "test.txt");
        assert!(entry.is_file());
        assert!(!entry.is_dir());

        let entry = Entry::directory("folder".to_string()).unwrap();
        assert_eq!(entry.name, "folder");
        assert!(entry.is_dir());
        assert!(!entry.is_file());
    }

    #[test]
    fn test_entry_validation() {
        // Valid names
        assert!(Entry::file("test.txt".to_string()).is_ok());
        assert!(Entry::file("my-file_2.txt".to_string()).is_ok());

        // Invalid names
        assert!(Entry::file("".to_string()).is_err());
        assert!(Entry::file("test/file.txt".to_string()).is_err());
        assert!(Entry::file("test\\file.txt".to_string()).is_err());
        assert!(Entry::file("test\0file.txt".to_string()).is_err());

        // Validate function
        assert!(Entry::validate_name("valid.txt").is_ok());
        assert!(Entry::validate_name("").is_err());
        assert!(Entry::validate_name("invalid/path").is_err());
    }

    #[test]
    fn test_entry_serialization() {
        let entry = Entry::file("test.txt".to_string()).unwrap();
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: Entry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);

        // Test invalid entry during deserialization
        let invalid_json = r#"{"name":"test/file.txt","file_type":"file"}"#;
        let result: Result<Entry, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());

        let empty_name_json = r#"{"name":"","file_type":"file"}"#;
        let result: Result<Entry, _> = serde_json::from_str(empty_name_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_vfs_error_validation() {
        // Valid error with non-empty message
        let error = VfsError::NotFound("file.txt".to_string());
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: VfsError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);

        // Invalid error with empty message should fail deserialization
        let invalid_json = r#"{"error":"not_found","details":""}"#;
        let result: Result<VfsError, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
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
