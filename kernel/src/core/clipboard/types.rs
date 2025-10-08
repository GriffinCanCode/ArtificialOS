/*!
 * Clipboard Types
 * Type-safe clipboard data structures with modern serde patterns
 */

use crate::core::types::{Pid, Timestamp};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Clipboard entry ID
pub type EntryId = u64;

/// Clipboard format types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardFormat {
    /// Plain text
    Text,
    /// Rich text/HTML
    Html,
    /// Raw binary data
    Bytes,
    /// File paths
    Files,
    /// Image data
    Image { mime_type: String },
    /// Custom MIME type
    Custom { mime_type: String },
}

impl ClipboardFormat {
    /// Get MIME type string
    #[must_use]
    pub fn mime_type(&self) -> &str {
        match self {
            Self::Text => "text/plain",
            Self::Html => "text/html",
            Self::Bytes => "application/octet-stream",
            Self::Files => "text/uri-list",
            Self::Image { mime_type } | Self::Custom { mime_type } => mime_type,
        }
    }
}

/// Clipboard data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ClipboardData {
    /// Plain text content
    Text(String),
    /// HTML content
    Html(String),
    /// Raw binary data
    Bytes(Vec<u8>),
    /// File paths
    Files(Vec<PathBuf>),
    /// Image data with format
    Image { data: Vec<u8>, mime_type: String },
}

impl ClipboardData {
    /// Get the format of this data
    #[must_use]
    pub fn format(&self) -> ClipboardFormat {
        match self {
            Self::Text(_) => ClipboardFormat::Text,
            Self::Html(_) => ClipboardFormat::Html,
            Self::Bytes(_) => ClipboardFormat::Bytes,
            Self::Files(_) => ClipboardFormat::Files,
            Self::Image { mime_type, .. } => ClipboardFormat::Image {
                mime_type: mime_type.clone(),
            },
        }
    }

    /// Get size in bytes
    #[must_use]
    pub fn size(&self) -> usize {
        match self {
            Self::Text(s) | Self::Html(s) => s.len(),
            Self::Bytes(b) | Self::Image { data: b, .. } => b.len(),
            Self::Files(files) => files.iter().map(|f| f.as_os_str().len()).sum(),
        }
    }

    /// Check if data is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Text(s) | Self::Html(s) => s.is_empty(),
            Self::Bytes(b) | Self::Image { data: b, .. } => b.is_empty(),
            Self::Files(files) => files.is_empty(),
        }
    }
}

/// Clipboard entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    /// Unique entry ID
    pub id: EntryId,
    /// Entry data
    pub data: ClipboardData,
    /// Source process ID
    pub source_pid: Pid,
    /// Creation timestamp (microseconds since epoch)
    pub timestamp: Timestamp,
    /// Optional label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl ClipboardEntry {
    /// Create a new clipboard entry
    #[must_use]
    pub fn new(id: EntryId, data: ClipboardData, source_pid: Pid) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as Timestamp)
            .unwrap_or(0);

        Self {
            id,
            data,
            source_pid,
            timestamp,
            label: None,
        }
    }

    /// Create with label
    #[must_use]
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// Get entry size
    #[must_use]
    pub fn size(&self) -> usize {
        self.data.size()
    }

    /// Get format
    #[must_use]
    pub fn format(&self) -> ClipboardFormat {
        self.data.format()
    }
}

/// Clipboard subscription for watching changes
#[derive(Debug, Clone)]
pub struct ClipboardSubscription {
    pub pid: Pid,
    pub formats: Vec<ClipboardFormat>,
}

/// Clipboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardStats {
    /// Total entries across all clipboards
    pub total_entries: usize,
    /// Total size in bytes
    pub total_size: usize,
    /// Number of processes with clipboards
    pub process_count: usize,
    /// Global clipboard entries
    pub global_entries: usize,
    /// Active subscriptions
    pub subscriptions: usize,
}

/// Clipboard errors
#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "error_type", content = "details", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ClipboardError {
    #[error("Entry not found: {0}")]
    NotFound(EntryId),

    #[error("Empty clipboard")]
    Empty,

    #[error("Invalid format: expected {expected}, got {actual}")]
    FormatMismatch { expected: String, actual: String },

    #[error("Data too large: {size} bytes (max: {max})")]
    TooLarge { size: usize, max: usize },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type for clipboard operations
pub type ClipboardResult<T> = Result<T, ClipboardError>;

