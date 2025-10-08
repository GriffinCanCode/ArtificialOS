/*!
 * Search Syscalls
 * File and content search operations
 */

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Search operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "syscall")]
#[non_exhaustive]
pub enum SearchSyscall {
    /// Search for files by name (fuzzy matching)
    SearchFiles {
        /// Starting directory
        path: PathBuf,
        /// Search query
        query: String,
        /// Maximum results
        #[serde(default = "default_limit")]
        limit: usize,
        /// Recursive search
        #[serde(default = "default_true")]
        recursive: bool,
        /// Case sensitive
        #[serde(default)]
        case_sensitive: bool,
        /// Match threshold (0.0-1.0)
        #[serde(default = "default_threshold")]
        threshold: f64,
    },

    /// Search file contents (grep-like)
    SearchContent {
        /// Starting directory
        path: PathBuf,
        /// Search query
        query: String,
        /// Maximum results
        #[serde(default = "default_limit")]
        limit: usize,
        /// Recursive search
        #[serde(default = "default_true")]
        recursive: bool,
        /// Case sensitive
        #[serde(default)]
        case_sensitive: bool,
        /// Include file path in results
        #[serde(default = "default_true")]
        include_path: bool,
    },
}

fn default_limit() -> usize {
    100
}

fn default_true() -> bool {
    true
}

fn default_threshold() -> f64 {
    0.3
}

/// Search result entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    /// File path
    pub path: PathBuf,
    /// Match score (0.0 = perfect, 1.0 = worst)
    pub score: f64,
    /// Matched content (for content search)
    pub content: Option<String>,
    /// Line number (for content search)
    pub line_number: Option<usize>,
}

