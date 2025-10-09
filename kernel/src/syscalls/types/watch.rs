/*!
 * File Watch Types
 * Support for file system event subscriptions
 */

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File system event notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FileWatchEvent {
    /// File or directory was created
    Created { path: PathBuf },

    /// File contents were modified
    Modified { path: PathBuf },

    /// File or directory was deleted
    Deleted { path: PathBuf },

    /// File or directory was renamed
    Renamed { from: PathBuf, to: PathBuf },
}

/// Watch subscription handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchHandle {
    pub watch_id: String,
    pub pattern: String,
}

