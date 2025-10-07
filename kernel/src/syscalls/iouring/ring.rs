/*!
 * Syscall Completion Ring
 * Core ring structure for io_uring-style syscall completion with lock-free queues
 */

 use super::submission::{SyscallSubmissionQueue, SyscallSubmissionEntry};
 use super::completion::{SyscallCompletionQueue, SyscallCompletionEntry, SyscallCompletionStatus};
 use super::IoUringError;
 use crate::core::types::Pid;
 use std::sync::Arc;
 use std::sync::atomic::{AtomicU64, Ordering};
 use std::time::{Duration, Instant};

 /// Completion ring with lock-free submission and completion queues
 ///
 /// # Performance
 /// - Lock-free ring buffers for zero-contention syscall batching
 /// - Optimized for high-frequency async syscall patterns
 /// - Cache-line aligned stats for accurate monitoring
 pub struct SyscallCompletionRing {
     pid: Pid,
     submission_queue: Arc<SyscallSubmissionQueue>,
     completion_queue: Arc<SyscallCompletionQueue>,
     stats: Arc<RingStats>,
 }

 impl SyscallCompletionRing {
     /// Create a new completion ring (lock-free)
     pub fn new(pid: Pid, sq_size: usize, cq_size: usize) -> Self {
         Self {
             pid,
             submission_queue: Arc::new(SyscallSubmissionQueue::new(sq_size)),
             completion_queue: Arc::new(SyscallCompletionQueue::new(cq_size)),
             stats: Arc::new(RingStats::default()),
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
     /// Hot path - zero-contention atomic operation
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
     }

     /// Wait for a completion with timeout (blocking)
     pub fn wait_completion_timeout(
         &self,
         seq: u64,
         timeout: Duration,
     ) -> Result<SyscallCompletionEntry, IoUringError> {
         let start = Instant::now();

         loop {
             // Lock-free check for completion
             if let Some(entry) = self.completion_queue.find_and_remove(seq) {
                 return Ok(entry);
             }

             if start.elapsed() >= timeout {
                 return Err(IoUringError::Timeout);
             }

             // Yield to avoid busy waiting
             std::thread::sleep(Duration::from_micros(100));
         }
     }

     /// Wait for a completion (blocking, no timeout)
     pub fn wait_completion(&self, seq: u64) -> Result<SyscallCompletionEntry, IoUringError> {
         // Use a very long timeout instead of infinite
         self.wait_completion_timeout(seq, Duration::from_secs(300))
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
