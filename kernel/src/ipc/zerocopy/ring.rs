/*!
 * Zero-Copy Ring Buffer
 * Core ring structure inspired by io_uring
 */

use super::submission::{SubmissionQueue, SubmissionEntry};
use super::completion::{CompletionQueue, CompletionEntry, CompletionStatus};
use super::ZeroCopyError;
use crate::core::types::{Pid, Size, Address};
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Zero-copy ring with submission and completion queues
pub struct ZeroCopyRing {
    pid: Pid,
    address: Address,
    submission_queue: Arc<RwLock<SubmissionQueue>>,
    completion_queue: Arc<RwLock<CompletionQueue>>,
    stats: Arc<RingStats>,
}

impl ZeroCopyRing {
    /// Create a new zero-copy ring
    pub fn new(pid: Pid, address: Address, sq_size: Size, cq_size: Size) -> Self {
        Self {
            pid,
            address,
            submission_queue: Arc::new(RwLock::new(SubmissionQueue::new(sq_size))),
            completion_queue: Arc::new(RwLock::new(CompletionQueue::new(cq_size))),
            stats: Arc::new(RingStats::default()),
        }
    }

    /// Submit an entry to the submission queue
    pub fn submit(&self, entry: SubmissionEntry) -> Result<u64, ZeroCopyError> {
        let mut sq = self.submission_queue.write();
        let seq = sq.push(entry)?;
        self.stats.submissions.fetch_add(1, Ordering::Relaxed);
        Ok(seq)
    }

    /// Complete an operation and add to completion queue
    pub fn complete(&self, seq: u64, status: CompletionStatus, result: usize) {
        let mut cq = self.completion_queue.write();
        let entry = CompletionEntry::new(seq, status, result);
        let _ = cq.push(entry);
        self.stats.completions.fetch_add(1, Ordering::Relaxed);
    }

    /// Wait for a completion (blocking)
    pub fn wait_completion(&self, seq: u64) -> Result<CompletionEntry, ZeroCopyError> {
        // In a real implementation, this would use futex or similar
        // For now, we'll simulate with polling
        loop {
            let mut cq = self.completion_queue.write();
            if let Some(entry) = cq.pop() {
                if entry.seq == seq {
                    return Ok(entry);
                }
                // Put it back if it's not the one we're waiting for
                let _ = cq.push(entry);
            }

            drop(cq);
            std::thread::yield_now();
        }
    }

    /// Try to get a completion (non-blocking)
    pub fn try_complete(&self) -> Option<CompletionEntry> {
        let mut cq = self.completion_queue.write();
        cq.pop()
    }

    /// Get the base address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get PID
    pub fn pid(&self) -> Pid {
        self.pid
    }

    /// Get statistics
    pub fn stats(&self) -> RingStatistics {
        RingStatistics {
            submissions: self.stats.submissions.load(Ordering::Relaxed),
            completions: self.stats.completions.load(Ordering::Relaxed),
        }
    }
}

/// Ring statistics
///
/// # Performance
/// - Cache-line aligned to prevent false sharing between submission and completion counters
#[repr(C, align(64))]
#[derive(Default)]
struct RingStats {
    submissions: AtomicU64,
    completions: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct RingStatistics {
    pub submissions: u64,
    pub completions: u64,
}

