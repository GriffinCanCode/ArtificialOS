/*!
 * Syscall Completion Queue
 * io_uring-style completion queue for syscall results with lock-free ring buffer
 */

use super::IoUringError;
use crate::ipc::lockfree_ring::LockFreeRing;
use crate::syscalls::types::SyscallResult;

/// Completion queue for syscall operations (lock-free)
///
/// # Performance
/// - Lock-free ring buffer for zero-contention completions
/// - Optimized for high-frequency result delivery
pub struct SyscallCompletionQueue {
    ring: LockFreeRing<SyscallCompletionEntry>,
}

impl SyscallCompletionQueue {
    /// Create a new completion queue (lock-free)
    pub fn new(capacity: usize) -> Self {
        Self {
            ring: LockFreeRing::new(capacity),
        }
    }

    /// Push an entry to the queue (lock-free)
    ///
    /// # Performance
    /// Hot path - lock-free atomic operation
    pub fn push(&self, entry: SyscallCompletionEntry) -> Result<(), IoUringError> {
        // If full, try to make space by popping oldest (ring buffer behavior)
        if self.ring.is_full() {
            let _ = self.ring.pop();
        }

        self.ring
            .push(entry)
            .map_err(|_| IoUringError::CompletionQueueFull)
    }

    /// Pop an entry from the queue (lock-free)
    ///
    /// # Performance
    /// Hot path - lock-free atomic operation
    pub fn pop(&self) -> Option<SyscallCompletionEntry> {
        self.ring.pop()
    }

    /// Find and remove a specific completion by sequence number
    ///
    /// # Note
    /// This operation requires scanning the queue and is O(n). For best performance,
    /// prefer popping in order rather than searching for specific completions.
    pub fn find_and_remove(&self, seq: u64) -> Option<SyscallCompletionEntry> {
        // Collect all entries temporarily
        let mut temp = Vec::new();
        let mut found = None;

        while let Some(entry) = self.ring.pop() {
            if entry.seq == seq && found.is_none() {
                found = Some(entry);
            } else {
                temp.push(entry);
            }
        }

        // Put back non-matching entries
        for entry in temp {
            let _ = self.ring.push(entry);
        }

        found
    }

    /// Check if queue is empty (lock-free)
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    /// Get number of pending completions (approximate, lock-free)
    pub fn pending(&self) -> usize {
        self.ring.len()
    }
}

/// Completion entry for a syscall operation
#[derive(Debug, Clone)]
pub struct SyscallCompletionEntry {
    /// Sequence number matching the submission
    pub seq: u64,
    /// Completion status
    pub status: SyscallCompletionStatus,
    /// Syscall result
    pub result: SyscallResult,
    /// User data from submission
    pub user_data: u64,
}

impl SyscallCompletionEntry {
    /// Create a new completion entry
    pub fn new(
        seq: u64,
        status: SyscallCompletionStatus,
        result: SyscallResult,
        user_data: u64,
    ) -> Self {
        Self {
            seq,
            status,
            result,
            user_data,
        }
    }

    /// Create a success completion
    pub fn success(seq: u64, result: SyscallResult, user_data: u64) -> Self {
        Self::new(seq, SyscallCompletionStatus::Success, result, user_data)
    }

    /// Create an error completion
    pub fn error(seq: u64, error: String, user_data: u64) -> Self {
        let result = SyscallResult::Error {
            message: error.clone(),
        };
        Self::new(
            seq,
            SyscallCompletionStatus::Error(error),
            result,
            user_data,
        )
    }

    /// Create a cancelled completion
    pub fn cancelled(seq: u64, user_data: u64) -> Self {
        let result = SyscallResult::Error {
            message: "Operation cancelled".to_string(),
        };
        Self::new(seq, SyscallCompletionStatus::Cancelled, result, user_data)
    }
}

/// Completion status for syscall operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyscallCompletionStatus {
    /// Operation completed successfully
    Success,
    /// Operation failed with error
    Error(String),
    /// Operation was cancelled
    Cancelled,
}

impl SyscallCompletionStatus {
    /// Check if the status indicates success
    pub fn is_success(&self) -> bool {
        matches!(self, SyscallCompletionStatus::Success)
    }

    /// Check if the status indicates an error
    pub fn is_error(&self) -> bool {
        matches!(self, SyscallCompletionStatus::Error(_))
    }

    /// Check if the status indicates cancellation
    pub fn is_cancelled(&self) -> bool {
        matches!(self, SyscallCompletionStatus::Cancelled)
    }

    /// Get error message if this is an error status
    pub fn error_message(&self) -> Option<&str> {
        match self {
            SyscallCompletionStatus::Error(msg) => Some(msg),
            _ => None,
        }
    }
}
