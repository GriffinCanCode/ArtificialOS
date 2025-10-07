/*!
 * Submission Queue
 * io_uring-style submission queue for zero-copy IPC
 */

use super::ZeroCopyError;
use crate::core::types::{Pid, Size, Address};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

/// Submission queue for zero-copy operations
pub struct SubmissionQueue {
    entries: VecDeque<SubmissionEntry>,
    capacity: Size,
    seq_counter: AtomicU64,
}

impl SubmissionQueue {
    /// Create a new submission queue
    pub fn new(capacity: Size) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
            seq_counter: AtomicU64::new(0),
        }
    }

    /// Push an entry to the queue
    pub fn push(&mut self, entry: SubmissionEntry) -> Result<u64, ZeroCopyError> {
        if self.entries.len() >= self.capacity {
            return Err(ZeroCopyError::SubmissionQueueFull);
        }

        let seq = self.seq_counter.fetch_add(1, Ordering::SeqCst);
        let mut entry = entry;
        entry.seq = seq;

        self.entries.push_back(entry);
        Ok(seq)
    }

    /// Pop an entry from the queue
    pub fn pop(&mut self) -> Option<SubmissionEntry> {
        self.entries.pop_front()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get available space
    pub fn available(&self) -> Size {
        self.capacity - self.entries.len()
    }
}

/// Submission entry for a zero-copy operation
#[derive(Debug, Clone)]
pub struct SubmissionEntry {
    /// Sequence number
    pub seq: u64,
    /// Operation type
    pub op: OperationType,
    /// Target process ID
    pub target_pid: Pid,
    /// Buffer address (in shared memory)
    pub buffer_addr: Address,
    /// Operation size
    pub size: Size,
}

impl SubmissionEntry {
    /// Create a new transfer operation
    pub fn new_transfer(target_pid: Pid, buffer_addr: Address, size: Size) -> Self {
        Self {
            seq: 0, // Will be set when pushed
            op: OperationType::Transfer,
            target_pid,
            buffer_addr,
            size,
        }
    }

    /// Create a new read operation
    pub fn new_read(target_pid: Pid, buffer_addr: Address, size: Size) -> Self {
        Self {
            seq: 0,
            op: OperationType::Read,
            target_pid,
            buffer_addr,
            size,
        }
    }

    /// Create a new write operation
    pub fn new_write(target_pid: Pid, buffer_addr: Address, size: Size) -> Self {
        Self {
            seq: 0,
            op: OperationType::Write,
            target_pid,
            buffer_addr,
            size,
        }
    }
}

/// Operation type for zero-copy IPC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// Transfer data between processes
    Transfer,
    /// Read from shared buffer
    Read,
    /// Write to shared buffer
    Write,
}

