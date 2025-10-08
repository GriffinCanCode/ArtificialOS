/*!
 * Clipboard Manager
 * High-performance clipboard with ring buffer history and subscriptions
 */

use super::types::*;
use crate::core::types::Pid;
use ahash::RandomState;
use dashmap::DashMap;
use log::{debug, info, trace};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Maximum clipboard history size per process
const MAX_HISTORY_SIZE: usize = 100;

/// Maximum clipboard entry size (10 MB)
const MAX_ENTRY_SIZE: usize = 10 * 1024 * 1024;

/// Per-process clipboard state
#[derive(Debug)]
struct ProcessClipboard {
    /// Ring buffer of clipboard entries
    history: VecDeque<ClipboardEntry>,
    /// Current clipboard entry (most recent)
    current: Option<ClipboardEntry>,
}

impl ProcessClipboard {
    fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            current: None,
        }
    }

    fn push(&mut self, entry: ClipboardEntry) {
        // Store current as history
        if let Some(current) = self.current.take() {
            self.history.push_front(current);
            if self.history.len() > MAX_HISTORY_SIZE {
                self.history.pop_back();
            }
        }
        self.current = Some(entry);
    }

    fn current(&self) -> Option<&ClipboardEntry> {
        self.current.as_ref()
    }

    fn history(&self) -> Vec<ClipboardEntry> {
        self.history.iter().cloned().collect()
    }

    fn clear(&mut self) {
        self.history.clear();
        self.current = None;
    }

    fn total_size(&self) -> usize {
        let current_size = self.current.as_ref().map_or(0, |e| e.size());
        let history_size: usize = self.history.iter().map(|e| e.size()).sum();
        current_size + history_size
    }
}

/// Clipboard manager with per-process isolation and global clipboard
#[derive(Clone)]
pub struct ClipboardManager {
    /// Per-process clipboards
    clipboards: Arc<DashMap<Pid, ProcessClipboard, RandomState>>,
    /// Global clipboard shared across processes
    global: Arc<parking_lot::RwLock<ProcessClipboard>>,
    /// Next entry ID
    next_id: Arc<AtomicU64>,
    /// Active subscriptions
    subscriptions: Arc<DashMap<Pid, ClipboardSubscription, RandomState>>,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    #[must_use]
    pub fn new() -> Self {
        info!("Clipboard manager initialized");
        Self {
            clipboards: Arc::new(DashMap::with_hasher(RandomState::new())),
            global: Arc::new(parking_lot::RwLock::new(ProcessClipboard::new())),
            next_id: Arc::new(AtomicU64::new(1)),
            subscriptions: Arc::new(DashMap::with_hasher(RandomState::new())),
        }
    }

    /// Copy data to process clipboard
    pub fn copy(&self, pid: Pid, data: ClipboardData) -> ClipboardResult<EntryId> {
        // Validate size
        let size = data.size();
        if size > MAX_ENTRY_SIZE {
            return Err(ClipboardError::TooLarge {
                size,
                max: MAX_ENTRY_SIZE,
            });
        }

        if data.is_empty() {
            return Err(ClipboardError::InvalidData("Empty data".to_string()));
        }

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let entry = ClipboardEntry::new(id, data, pid);

        // Insert into process clipboard
        self.clipboards
            .entry(pid)
            .or_insert_with(ProcessClipboard::new)
            .push(entry.clone());

        debug!("PID {} copied to clipboard: entry {}", pid, id);
        self.notify_subscribers(&entry);

        Ok(id)
    }

    /// Copy data to global clipboard
    pub fn copy_global(&self, pid: Pid, data: ClipboardData) -> ClipboardResult<EntryId> {
        let size = data.size();
        if size > MAX_ENTRY_SIZE {
            return Err(ClipboardError::TooLarge {
                size,
                max: MAX_ENTRY_SIZE,
            });
        }

        if data.is_empty() {
            return Err(ClipboardError::InvalidData("Empty data".to_string()));
        }

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let entry = ClipboardEntry::new(id, data, pid);

        self.global.write().push(entry.clone());
        debug!("PID {} copied to global clipboard: entry {}", pid, id);
        self.notify_subscribers(&entry);

        Ok(id)
    }

    /// Paste from process clipboard
    pub fn paste(&self, pid: Pid) -> ClipboardResult<ClipboardEntry> {
        self.clipboards
            .get(&pid)
            .and_then(|cb| cb.current().cloned())
            .ok_or(ClipboardError::Empty)
    }

    /// Paste from global clipboard
    pub fn paste_global(&self) -> ClipboardResult<ClipboardEntry> {
        self.global
            .read()
            .current()
            .cloned()
            .ok_or(ClipboardError::Empty)
    }

    /// Get clipboard history for process
    pub fn history(&self, pid: Pid, limit: Option<usize>) -> Vec<ClipboardEntry> {
        let history = self
            .clipboards
            .get(&pid)
            .map(|cb| cb.history())
            .unwrap_or_default();

        match limit {
            Some(n) => history.into_iter().take(n).collect(),
            None => history,
        }
    }

    /// Get global clipboard history
    pub fn history_global(&self, limit: Option<usize>) -> Vec<ClipboardEntry> {
        let history = self.global.read().history();
        match limit {
            Some(n) => history.into_iter().take(n).collect(),
            None => history,
        }
    }

    /// Get specific entry by ID
    pub fn get_entry(&self, pid: Pid, entry_id: EntryId) -> ClipboardResult<ClipboardEntry> {
        // Check current
        if let Some(cb) = self.clipboards.get(&pid) {
            if let Some(current) = cb.current() {
                if current.id == entry_id {
                    return Ok(current.clone());
                }
            }
            // Check history
            for entry in cb.history().iter() {
                if entry.id == entry_id {
                    return Ok(entry.clone());
                }
            }
        }

        Err(ClipboardError::NotFound(entry_id))
    }

    /// Clear process clipboard
    pub fn clear(&self, pid: Pid) {
        if let Some(mut cb) = self.clipboards.get_mut(&pid) {
            cb.clear();
            debug!("PID {} cleared clipboard", pid);
        }
    }

    /// Clear global clipboard
    pub fn clear_global(&self) {
        self.global.write().clear();
        debug!("Global clipboard cleared");
    }

    /// Subscribe to clipboard changes
    pub fn subscribe(&self, pid: Pid, formats: Vec<ClipboardFormat>) {
        let subscription = ClipboardSubscription { pid, formats };
        self.subscriptions.insert(pid, subscription);
        debug!("PID {} subscribed to clipboard changes", pid);
    }

    /// Unsubscribe from clipboard changes
    pub fn unsubscribe(&self, pid: Pid) {
        self.subscriptions.remove(&pid);
        debug!("PID {} unsubscribed from clipboard changes", pid);
    }

    /// Notify subscribers of clipboard change
    fn notify_subscribers(&self, entry: &ClipboardEntry) {
        let format = entry.format();
        for sub in self.subscriptions.iter() {
            if sub.formats.is_empty() || sub.formats.contains(&format) {
                trace!(
                    "Notifying PID {} of clipboard change: entry {}",
                    sub.pid,
                    entry.id
                );
                // TODO: Implement notification mechanism (signals or IPC message)
            }
        }
    }

    /// Get clipboard statistics
    #[must_use]
    pub fn stats(&self) -> ClipboardStats {
        let process_count = self.clipboards.len();
        let mut total_entries = 0;
        let mut total_size = 0;

        for cb in self.clipboards.iter() {
            if cb.current().is_some() {
                total_entries += 1;
            }
            total_entries += cb.history().len();
            total_size += cb.total_size();
        }

        let global = self.global.read();
        let global_entries = if global.current().is_some() { 1 } else { 0 } + global.history().len();
        total_entries += global_entries;
        total_size += global.total_size();

        ClipboardStats {
            total_entries,
            total_size,
            process_count,
            global_entries,
            subscriptions: self.subscriptions.len(),
        }
    }

    /// Cleanup clipboard for terminated process
    pub fn cleanup(&self, pid: Pid) {
        self.clipboards.remove(&pid);
        self.subscriptions.remove(&pid);
        debug!("Cleaned up clipboard for PID {}", pid);
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_paste() {
        let manager = ClipboardManager::new();
        let pid = 100;
        let data = ClipboardData::Text("Hello, World!".to_string());

        let entry_id = manager.copy(pid, data.clone()).unwrap();
        assert!(entry_id > 0);

        let pasted = manager.paste(pid).unwrap();
        assert_eq!(pasted.id, entry_id);
        assert_eq!(pasted.source_pid, pid);
    }

    #[test]
    fn test_history() {
        let manager = ClipboardManager::new();
        let pid = 100;

        manager
            .copy(pid, ClipboardData::Text("First".to_string()))
            .unwrap();
        manager
            .copy(pid, ClipboardData::Text("Second".to_string()))
            .unwrap();
        manager
            .copy(pid, ClipboardData::Text("Third".to_string()))
            .unwrap();

        let history = manager.history(pid, None);
        assert_eq!(history.len(), 2); // First two entries
    }

    #[test]
    fn test_global_clipboard() {
        let manager = ClipboardManager::new();
        let data = ClipboardData::Text("Global".to_string());

        manager.copy_global(100, data).unwrap();
        let pasted = manager.paste_global().unwrap();
        assert_eq!(pasted.source_pid, 100);
    }

    #[test]
    fn test_size_limit() {
        let manager = ClipboardManager::new();
        let large_data = ClipboardData::Bytes(vec![0u8; MAX_ENTRY_SIZE + 1]);

        let result = manager.copy(100, large_data);
        assert!(matches!(result, Err(ClipboardError::TooLarge { .. })));
    }
}

