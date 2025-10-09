/*!
 * File Watch Syscall Implementation
 * Subscribe to file system events
 */

use crate::core::serialization::json;
use crate::core::types::Pid;
use crate::monitoring::span_operation;
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::types::watch::{FileWatchEvent, WatchHandle};
use crate::syscalls::types::SyscallResult;
use crate::vfs::{FileEvent, Observable};

use log::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use dashmap::DashMap;
use std::collections::HashMap;

/// Active watch subscriptions
/// Maps watch_id -> (pid, pattern, task_handle)
pub struct WatchManager {
    subscriptions: Arc<DashMap<String, (Pid, String, tokio::task::JoinHandle<()>)>>,
}

impl WatchManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(DashMap::new()),
        }
    }

    /// Cancel all watches for a process
    pub fn cleanup_process(&self, pid: Pid) {
        self.subscriptions.retain(|_, (watch_pid, _, handle)| {
            if *watch_pid == pid {
                handle.abort();
                false // Remove this subscription
            } else {
                true // Keep this subscription
            }
        });
    }
}

impl Default for WatchManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SyscallExecutorWithIpc {
    /// Watch files matching pattern
    pub(in crate::syscalls) async fn watch_files(
        &self,
        pid: Pid,
        pattern: String,
    ) -> SyscallResult {
        let span = span_operation("watch_files");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("pattern", &pattern);

        // Check if VFS supports observability
        let vfs = match &self.optional().vfs {
            Some(vfs) => vfs.clone(),
            None => {
                warn!("VFS not available for file watching");
                return SyscallResult::error("VFS not configured");
            }
        };

        // Try to cast to Observable
        // TODO: Make VFS Observable by default
        info!("File watching requested for pattern: {}", pattern);

        // Generate watch ID
        let watch_id = Uuid::new_v4().to_string();

        // For now, return success with watch handle
        // Full implementation requires:
        // 1. Making MountManager Observable
        // 2. Background task to stream events via IPC
        // 3. Pattern matching against paths

        let handle = WatchHandle {
            watch_id: watch_id.clone(),
            pattern: pattern.clone(),
        };

        match json::to_vec(&handle) {
            Ok(data) => {
                info!("Watch created: {} for pattern {}", watch_id, pattern);
                span.record_result(true);
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                warn!("Failed to serialize watch handle: {}", e);
                span.record_error("Serialization failed");
                SyscallResult::error("Failed to create watch")
            }
        }
    }

    /// Stop watching files
    pub(in crate::syscalls) async fn unwatch_files(
        &self,
        pid: Pid,
        watch_id: String,
    ) -> SyscallResult {
        let span = span_operation("unwatch_files");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("watch_id", &watch_id);

        info!("Unwatch requested for: {}", watch_id);

        // TODO: Remove subscription from watch manager

        span.record_result(true);
        SyscallResult::success()
    }
}

/// Convert VFS FileEvent to syscall FileWatchEvent
fn convert_event(event: FileEvent) -> FileWatchEvent {
    match event {
        FileEvent::Created { path } => FileWatchEvent::Created { path },
        FileEvent::Modified { path } => FileWatchEvent::Modified { path },
        FileEvent::Deleted { path } => FileWatchEvent::Deleted { path },
        FileEvent::Renamed { from, to } => FileWatchEvent::Renamed { from, to },
    }
}

