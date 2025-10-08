/*!
 * io_uring-style Async Syscall Completion
 *
 * Provides io_uring-inspired submission/completion queues for syscalls,
 * optimized for I/O-heavy operations like file and network I/O.
 *
 * This augments the existing AsyncTaskManager with a more efficient
 * batched submission and completion model for operations that benefit
 * from it, without replacing existing patterns.
 */

mod completion;
mod executor;
pub mod handlers;
mod ring;
mod submission;

pub use completion::{SyscallCompletionEntry, SyscallCompletionQueue, SyscallCompletionStatus};
pub use executor::IoUringExecutor;
pub use ring::SyscallCompletionRing;
pub use submission::{SyscallOpType, SyscallSubmissionEntry, SyscallSubmissionQueue};

use crate::core::types::Pid;
use ahash::RandomState;
use dashmap::DashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info};

/// Default queue sizes for submission and completion
pub use crate::core::limits::{DEFAULT_CQ_SIZE, DEFAULT_SQ_SIZE};

/// io_uring-style manager for async syscall completion
///
/// This provides efficient batched syscall submission and completion
/// for I/O-heavy operations. It works alongside the existing AsyncTaskManager
/// and is used for operations that benefit from the io_uring model.
#[derive(Clone)]
pub struct IoUringManager {
    /// Completion rings per process
    rings: Arc<DashMap<Pid, Arc<SyscallCompletionRing>, RandomState>>,
    /// Shared executor for async operations
    executor: Arc<IoUringExecutor>,
}

impl IoUringManager {
    /// Create a new io_uring-style manager
    pub fn new(executor: Arc<IoUringExecutor>) -> Self {
        info!("Initializing io_uring-style syscall completion manager");
        Self {
            rings: Arc::new(DashMap::with_hasher(RandomState::new().into())),
            executor,
        }
    }

    /// Create a completion ring for a process
    pub fn create_ring(
        &self,
        pid: Pid,
        sq_size: Option<usize>,
        cq_size: Option<usize>,
    ) -> Result<Arc<SyscallCompletionRing>, IoUringError> {
        let sq_size = sq_size.unwrap_or(DEFAULT_SQ_SIZE);
        let cq_size = cq_size.unwrap_or(DEFAULT_CQ_SIZE);

        debug!(
            pid = pid,
            sq_size = sq_size,
            cq_size = cq_size,
            "Creating io_uring-style completion ring"
        );

        let ring = Arc::new(SyscallCompletionRing::new(pid, sq_size, cq_size));
        self.rings.insert(pid, ring.clone());

        info!(pid = pid, "io_uring-style completion ring created");

        Ok(ring)
    }

    /// Get a process's completion ring
    pub fn get_ring(&self, pid: Pid) -> Option<Arc<SyscallCompletionRing>> {
        self.rings.get(&pid).map(|r| r.clone())
    }

    /// Get or create a ring for a process
    pub fn get_or_create_ring(&self, pid: Pid) -> Result<Arc<SyscallCompletionRing>, IoUringError> {
        if let Some(ring) = self.get_ring(pid) {
            Ok(ring)
        } else {
            self.create_ring(pid, None, None)
        }
    }

    /// Submit a syscall operation
    pub fn submit(&self, pid: Pid, entry: SyscallSubmissionEntry) -> Result<u64, IoUringError> {
        let ring = self.get_or_create_ring(pid)?;
        let seq = ring.submit(entry)?;

        // Spawn async execution
        let ring_clone = ring.clone();
        let executor = self.executor.clone();
        tokio::spawn(async move {
            executor.execute_async(ring_clone).await;
        });

        Ok(seq)
    }

    /// Submit multiple syscalls in a batch
    pub fn submit_batch(
        &self,
        pid: Pid,
        entries: Vec<SyscallSubmissionEntry>,
    ) -> Result<Vec<u64>, IoUringError> {
        let ring = self.get_or_create_ring(pid)?;
        let mut seqs = Vec::with_capacity(entries.len());

        for entry in entries {
            let seq = ring.submit(entry)?;
            seqs.push(seq);
        }

        // Spawn async batch execution
        let ring_clone = ring.clone();
        let executor = self.executor.clone();
        tokio::spawn(async move {
            executor.execute_batch_async(ring_clone).await;
        });

        Ok(seqs)
    }

    /// Try to get completions (non-blocking)
    pub fn reap_completions(
        &self,
        pid: Pid,
        max: Option<usize>,
    ) -> Result<Vec<SyscallCompletionEntry>, IoUringError> {
        let ring = self.get_ring(pid).ok_or(IoUringError::RingNotFound(pid))?;

        let max = max.unwrap_or(usize::MAX);
        let mut completions = Vec::new();

        for _ in 0..max {
            if let Some(entry) = ring.try_complete() {
                completions.push(entry);
            } else {
                break;
            }
        }

        Ok(completions)
    }

    /// Wait for a specific completion (blocking)
    pub fn wait_completion(
        &self,
        pid: Pid,
        seq: u64,
    ) -> Result<SyscallCompletionEntry, IoUringError> {
        let ring = self.get_ring(pid).ok_or(IoUringError::RingNotFound(pid))?;

        ring.wait_completion(seq)
    }

    /// Destroy a completion ring
    pub fn destroy_ring(&self, pid: Pid) -> Result<(), IoUringError> {
        self.rings.remove(&pid);
        info!(pid = pid, "io_uring-style completion ring destroyed");
        Ok(())
    }

    /// Cleanup all rings for a terminated process
    pub fn cleanup_process_rings(&self, pid: Pid) -> usize {
        if self.rings.remove(&pid).is_some() {
            info!("Cleaned io_uring ring for terminated PID {}", pid);
            1
        } else {
            0
        }
    }

    /// Check if process has any rings
    pub fn has_process_rings(&self, pid: Pid) -> bool {
        self.rings.contains_key(&pid)
    }

    /// Get statistics
    pub fn stats(&self) -> IoUringStats {
        let total_rings = self.rings.len();
        let mut total_submissions = 0;
        let mut total_completions = 0;

        for ring in self.rings.iter() {
            let stats = ring.stats();
            total_submissions += stats.submissions;
            total_completions += stats.completions;
        }

        IoUringStats {
            active_rings: total_rings,
            total_submissions,
            total_completions,
            pending: total_submissions.saturating_sub(total_completions),
        }
    }
}

/// io_uring error types
#[derive(Error, Debug)]
pub enum IoUringError {
    #[error("Ring not found for PID {0}")]
    RingNotFound(Pid),

    #[error("Submission queue full")]
    SubmissionQueueFull,

    #[error("Completion queue full")]
    CompletionQueueFull,

    #[error("Completion queue empty")]
    CompletionQueueEmpty,

    #[error("Operation timeout")]
    Timeout,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),
}

/// Statistics for io_uring operations
#[derive(Debug, Clone)]
pub struct IoUringStats {
    pub active_rings: usize,
    pub total_submissions: u64,
    pub total_completions: u64,
    pub pending: u64,
}
