/*!
 * Zero-Copy IPC
 *
 * io_uring-inspired zero-copy IPC mechanisms that eliminate unnecessary data copies
 * between processes by using shared memory and submission/completion queues.
 */

mod ring;
mod submission;
mod completion;
mod buffer_pool;

pub use ring::ZeroCopyRing;
pub use submission::{SubmissionQueue, SubmissionEntry};
pub use completion::{CompletionQueue, CompletionEntry};
pub use buffer_pool::BufferPool;

use crate::core::types::{Pid, Size, Address};
use crate::memory::MemoryManager;
use dashmap::DashMap;
use ahash::RandomState;
use std::sync::Arc;
use tracing::{info, debug};
use thiserror::Error;

/// Zero-copy IPC manager
#[derive(Clone)]
pub struct ZeroCopyIpc {
    /// Active rings per process
    rings: Arc<DashMap<Pid, Arc<ZeroCopyRing>, RandomState>>,
    /// Shared buffer pools
    buffer_pools: Arc<DashMap<Pid, Arc<BufferPool>, RandomState>>,
    /// Memory manager for allocation
    memory_manager: MemoryManager,
}

impl ZeroCopyIpc {
    /// Create a new zero-copy IPC manager
    pub fn new(memory_manager: MemoryManager) -> Self {
        info!("Initializing zero-copy IPC with io_uring-inspired design");
        Self {
            rings: Arc::new(DashMap::with_hasher(RandomState::new())),
            buffer_pools: Arc::new(DashMap::with_hasher(RandomState::new())),
            memory_manager,
        }
    }

    /// Create a zero-copy ring for a process
    pub fn create_ring(
        &self,
        pid: Pid,
        sq_size: Size,
        cq_size: Size,
    ) -> Result<Arc<ZeroCopyRing>, ZeroCopyError> {
        debug!(
            pid = pid,
            sq_size = sq_size,
            cq_size = cq_size,
            "Creating zero-copy ring"
        );

        // Allocate shared memory for the ring
        let ring_size = sq_size + cq_size;
        let address = self
            .memory_manager
            .allocate(ring_size, pid)
            .map_err(|e| ZeroCopyError::AllocationFailed(format!("{}", e)))?;

        // Create the ring
        let ring = Arc::new(ZeroCopyRing::new(pid, address, sq_size, cq_size));

        // Register the ring
        self.rings.insert(pid, ring.clone());

        // Create buffer pool for this process
        let buffer_pool = Arc::new(BufferPool::new(pid, self.memory_manager.clone()));
        self.buffer_pools.insert(pid, buffer_pool);

        info!(
            pid = pid,
            address = format!("0x{:x}", address),
            "Zero-copy ring created"
        );

        Ok(ring)
    }

    /// Get a process's zero-copy ring
    pub fn get_ring(&self, pid: Pid) -> Option<Arc<ZeroCopyRing>> {
        self.rings.get(&pid).map(|r| r.clone())
    }

    /// Get a process's buffer pool
    pub fn get_buffer_pool(&self, pid: Pid) -> Option<Arc<BufferPool>> {
        self.buffer_pools.get(&pid).map(|p| p.clone())
    }

    /// Submit an IPC operation for zero-copy transfer
    pub fn submit_operation(
        &self,
        pid: Pid,
        target_pid: Pid,
        buffer_addr: Address,
        size: Size,
    ) -> Result<u64, ZeroCopyError> {
        let ring = self
            .get_ring(pid)
            .ok_or(ZeroCopyError::RingNotFound(pid))?;

        // Create submission entry
        let entry = SubmissionEntry::new_transfer(target_pid, buffer_addr, size);

        // Submit to ring
        let seq = ring.submit(entry)?;

        debug!(
            pid = pid,
            target_pid = target_pid,
            seq = seq,
            size = size,
            "Zero-copy operation submitted"
        );

        Ok(seq)
    }

    /// Complete an IPC operation and get result
    pub fn complete_operation(
        &self,
        pid: Pid,
        seq: u64,
    ) -> Result<CompletionEntry, ZeroCopyError> {
        let ring = self
            .get_ring(pid)
            .ok_or(ZeroCopyError::RingNotFound(pid))?;

        // Wait for completion
        let completion = ring.wait_completion(seq)?;

        debug!(
            pid = pid,
            seq = seq,
            status = ?completion.status,
            "Zero-copy operation completed"
        );

        Ok(completion)
    }

    /// Destroy a zero-copy ring
    pub fn destroy_ring(&self, pid: Pid) -> Result<(), ZeroCopyError> {
        // Remove ring
        if let Some((_, ring)) = self.rings.remove(&pid) {
            // Deallocate memory
            self.memory_manager
                .deallocate(ring.address())
                .map_err(|e| ZeroCopyError::DeallocationFailed(format!("{}", e)))?;
        }

        // Remove buffer pool
        self.buffer_pools.remove(&pid);

        info!(pid = pid, "Zero-copy ring destroyed");
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> ZeroCopyStats {
        let total_rings = self.rings.len();
        let total_buffer_pools = self.buffer_pools.len();

        let mut total_submissions = 0;
        let mut total_completions = 0;

        for ring in self.rings.iter() {
            let stats = ring.stats();
            total_submissions += stats.submissions;
            total_completions += stats.completions;
        }

        ZeroCopyStats {
            active_rings: total_rings,
            active_buffer_pools: total_buffer_pools,
            total_submissions,
            total_completions,
        }
    }
}

/// Zero-copy IPC error
#[derive(Error, Debug)]
pub enum ZeroCopyError {
    #[error("Ring not found for PID {0}")]
    RingNotFound(Pid),

    #[error("Submission queue full")]
    SubmissionQueueFull,

    #[error("Completion queue empty")]
    CompletionQueueEmpty,

    #[error("Allocation failed: {0}")]
    AllocationFailed(String),

    #[error("Deallocation failed: {0}")]
    DeallocationFailed(String),

    #[error("Operation timeout")]
    Timeout,

    #[error("Invalid operation")]
    InvalidOperation,
}

/// Zero-copy IPC statistics
#[derive(Debug, Clone)]
pub struct ZeroCopyStats {
    pub active_rings: usize,
    pub active_buffer_pools: usize,
    pub total_submissions: u64,
    pub total_completions: u64,
}

