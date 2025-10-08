/*!
 * Completion Queue
 * io_uring-style completion queue for zero-copy IPC
 */

use super::ZeroCopyError;
use crate::core::types::Size;
use std::collections::VecDeque;

/// Completion queue for zero-copy operations
pub struct CompletionQueue {
    entries: VecDeque<CompletionEntry>,
    capacity: Size,
}

impl CompletionQueue {
    /// Create a new completion queue
    pub fn new(capacity: Size) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push an entry to the queue
    pub fn push(&mut self, entry: CompletionEntry) -> Result<(), ZeroCopyError> {
        if self.entries.len() >= self.capacity {
            // Drop oldest completion to make space
            self.entries.pop_front();
        }

        self.entries.push_back(entry);
        Ok(())
    }

    /// Pop an entry from the queue
    pub fn pop(&mut self) -> Option<CompletionEntry> {
        self.entries.pop_front()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get number of pending completions
    pub fn pending(&self) -> Size {
        self.entries.len()
    }
}

/// Completion entry for a zero-copy operation
#[derive(Debug, Clone)]
pub struct CompletionEntry {
    /// Sequence number matching the submission
    pub seq: u64,
    /// Completion status
    pub status: CompletionStatus,
    /// Result (e.g., bytes transferred)
    pub result: usize,
}

impl CompletionEntry {
    /// Create a new completion entry
    pub fn new(seq: u64, status: CompletionStatus, result: usize) -> Self {
        Self {
            seq,
            status,
            result,
        }
    }

    /// Create a success completion
    pub fn success(seq: u64, result: usize) -> Self {
        Self::new(seq, CompletionStatus::Success, result)
    }

    /// Create an error completion
    pub fn error(seq: u64, error: String) -> Self {
        Self::new(seq, CompletionStatus::Error(error), 0)
    }
}

/// Completion status
#[derive(Debug, Clone)]
pub enum CompletionStatus {
    /// Operation completed successfully
    Success,
    /// Operation failed with error
    Error(String),
    /// Operation was cancelled
    Cancelled,
}
