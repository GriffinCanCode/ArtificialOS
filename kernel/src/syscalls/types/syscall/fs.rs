/*!
 * Filesystem Syscalls
 * File and directory operations
 */

use crate::core::types::Fd;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Filesystem operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "syscall")]
#[non_exhaustive]
pub enum FsSyscall {
    /// Read file contents
    ReadFile {
        /// Path to file
        path: PathBuf,
    },

    /// Write data to file
    WriteFile {
        /// Path to file
        path: PathBuf,
        /// Data to write
        data: Vec<u8>,
    },

    /// Create empty file
    CreateFile {
        /// Path to file
        path: PathBuf,
    },

    /// Delete file
    DeleteFile {
        /// Path to file
        path: PathBuf,
    },

    /// List directory contents
    ListDirectory {
        /// Path to directory
        path: PathBuf,
    },

    /// Check if file exists
    FileExists {
        /// Path to check
        path: PathBuf,
    },

    /// Get file metadata
    FileStat {
        /// Path to file
        path: PathBuf,
    },

    /// Move/rename file
    MoveFile {
        /// Source path
        source: PathBuf,
        /// Destination path
        destination: PathBuf,
    },

    /// Copy file
    CopyFile {
        /// Source path
        source: PathBuf,
        /// Destination path
        destination: PathBuf,
    },

    /// Create directory
    CreateDirectory {
        /// Path to directory
        path: PathBuf,
    },

    /// Remove directory
    RemoveDirectory {
        /// Path to directory
        path: PathBuf,
    },

    /// Get current working directory
    GetWorkingDirectory,

    /// Set current working directory
    SetWorkingDirectory {
        /// Path to directory
        path: PathBuf,
    },

    /// Truncate file to size
    TruncateFile {
        /// Path to file
        path: PathBuf,
        /// New size in bytes
        size: u64,
    },

    /// Open file and return FD
    Open {
        /// Path to file
        path: PathBuf,
        /// Open flags (O_RDONLY, O_WRONLY, O_RDWR, O_CREAT, O_APPEND, etc.)
        flags: u32,
        /// File permissions (0644, etc.)
        #[serde(default)]
        mode: u32,
    },

    /// Close file descriptor
    Close {
        /// File descriptor
        fd: Fd,
    },

    /// Duplicate file descriptor
    Dup {
        /// File descriptor to duplicate
        fd: Fd,
    },

    /// Duplicate FD to specific number
    Dup2 {
        /// Source file descriptor
        oldfd: Fd,
        /// Target file descriptor
        newfd: Fd,
    },

    /// Seek within file
    Lseek {
        /// File descriptor
        fd: Fd,
        /// Offset to seek to
        offset: i64,
        /// Seek mode (SEEK_SET, SEEK_CUR, SEEK_END)
        whence: u32,
    },

    /// File control operations
    Fcntl {
        /// File descriptor
        fd: Fd,
        /// Control command
        cmd: u32,
        /// Command argument
        #[serde(default)]
        arg: u32,
    },
}
