/*!
 * Syscall Completion Ring
 * Core ring structure for io_uring-style syscall completion with lock-free queues
 */

use super::completion::{SyscallCompletionEntry, SyscallCompletionQueue, SyscallCompletionStatus};
use super::submission::{SyscallSubmissionEntry, SyscallSubmissionQueue};
use super::IoUringError;
use crate::core::sync::WaitQueue;
use crate::core::types::Pid;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Default timeout for syscall completion operations (30 seconds)
const DEFAULT_COMPLETION_TIMEOUT: Duration = Duration::from_secs(30);

/// Completion ring with lock-free submission and completion queues
///
/// # Performance
/// - Lock-free ring buffers for zero-contention syscall batching
/// - Optimized for high-frequency async syscall patterns
/// - Cache-line aligned stats for accurate monitoring
/// - Futex-based waiting (Linux) for minimal overhead
pub struct SyscallCompletionRing {
    pid: Pid,
    submission_queue: Arc<SyscallSubmissionQueue>,
    completion_queue: Arc<SyscallCompletionQueue>,
    stats: Arc<RingStats>,
    /// Efficient wait queue for completion notifications
    wait_queue: WaitQueue<u64>,
}

impl SyscallCompletionRing {
    /// Create a new completion ring (lock-free)
    pub fn new(pid: Pid, sq_size: usize, cq_size: usize) -> Self {
        Self {
            pid,
            submission_queue: Arc::new(SyscallSubmissionQueue::new(sq_size)),
            completion_queue: Arc::new(SyscallCompletionQueue::new(cq_size)),
            stats: Arc::new(RingStats::default()),
            // Use low_latency config for syscall completions
            wait_queue: WaitQueue::low_latency(),
        }
    }

    /// Submit an entry to the submission queue (lock-free)
    ///
    /// # Performance
    /// Hot path - zero-contention atomic operation
    pub fn submit(&self, entry: SyscallSubmissionEntry) -> Result<u64, IoUringError> {
        let seq = self.submission_queue.push(entry)?;
        self.stats.submissions.fetch_add(1, Ordering::Relaxed);
        Ok(seq)
    }

    /// Pop a submission entry for execution (lock-free)
    ///
    /// # Performance
    /// Hot path - zero-contention atomic operation
    pub fn pop_submission(&self) -> Option<SyscallSubmissionEntry> {
        self.submission_queue.pop()
    }

    /// Pop multiple submission entries for batch processing (lock-free)
    ///
    /// # Performance
    /// Hot path - optimized for syscall batching
    pub fn pop_submissions(&self, max: usize) -> Vec<SyscallSubmissionEntry> {
        self.submission_queue.pop_batch(max)
    }

    /// Complete an operation and add to completion queue (lock-free)
    ///
    /// # Performance
    /// Hot path - zero-contention atomic operation with efficient wake
    pub fn complete(
        &self,
        seq: u64,
        status: SyscallCompletionStatus,
        result: crate::syscalls::types::SyscallResult,
        user_data: u64,
    ) {
        let entry = SyscallCompletionEntry::new(seq, status, result, user_data);
        let _ = self.completion_queue.push(entry);
        self.stats.completions.fetch_add(1, Ordering::Relaxed);

        // Wake any waiters (futex on Linux, no polling!)
        self.wait_queue.wake_one(seq);
    }

    /// Wait for a completion with timeout (blocking)
    ///
    /// # Performance
    ///
    /// Uses adaptive spinwait followed by futex (Linux) for optimal latency
    pub fn wait_completion_timeout(
        &self,
        seq: u64,
        timeout: Duration,
    ) -> Result<SyscallCompletionEntry, IoUringError> {
        let start = Instant::now();

        loop {
            // Lock-free fast path: check if completion is already available
            if let Some(entry) = self.completion_queue.find_and_remove(seq) {
                return Ok(entry);
            }

            // Check timeout
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Err(IoUringError::Timeout);
            }

            // Calculate remaining timeout
            let remaining = timeout.saturating_sub(elapsed);

            // Efficient wait (adaptive spin + futex = no busy-loop)
            self.wait_queue
                .wait(seq, Some(remaining))
                .map_err(|_| IoUringError::Timeout)?;

            // Loop to check completion queue again after wake
        }
    }

    /// Wait for a completion (blocking, with default timeout)
    ///
    /// Uses a default timeout of 30 seconds to prevent hung operations from blocking indefinitely.
    /// For custom timeouts, use `wait_completion_timeout` directly.
    pub fn wait_completion(&self, seq: u64) -> Result<SyscallCompletionEntry, IoUringError> {
        self.wait_completion_timeout(seq, DEFAULT_COMPLETION_TIMEOUT)
    }

    /// Try to get a completion (non-blocking, lock-free)
    pub fn try_complete(&self) -> Option<SyscallCompletionEntry> {
        self.completion_queue.pop()
    }

    /// Try to get a specific completion (non-blocking, lock-free)
    pub fn try_complete_seq(&self, seq: u64) -> Option<SyscallCompletionEntry> {
        self.completion_queue.find_and_remove(seq)
    }

    /// Get PID
    pub fn pid(&self) -> Pid {
        self.pid
    }

    /// Get submission queue pending count (approximate, lock-free)
    pub fn sq_pending(&self) -> usize {
        self.submission_queue.pending()
    }

    /// Get completion queue pending count (approximate, lock-free)
    pub fn cq_pending(&self) -> usize {
        self.completion_queue.pending()
    }

    /// Check if submission queue is empty (lock-free)
    pub fn sq_is_empty(&self) -> bool {
        self.submission_queue.is_empty()
    }

    /// Check if completion queue is empty (lock-free)
    pub fn cq_is_empty(&self) -> bool {
        self.completion_queue.is_empty()
    }

    /// Get statistics (lock-free)
    pub fn stats(&self) -> RingStatistics {
        RingStatistics {
            submissions: self.stats.submissions.load(Ordering::Relaxed),
            completions: self.stats.completions.load(Ordering::Relaxed),
        }
    }
}

/// Ring statistics
///
/// Cache-line aligned to prevent false sharing
#[repr(C, align(64))]
#[derive(Default)]
struct RingStats {
    submissions: AtomicU64,
    completions: AtomicU64,
}

/// Public statistics snapshot
#[derive(Debug, Clone)]
pub struct RingStatistics {
    pub submissions: u64,
    pub completions: u64,
}
