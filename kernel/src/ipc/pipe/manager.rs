/*!
 * Pipe Manager
 * Central manager for Unix-style pipes
 */

use super::super::traits::PipeChannel;
use super::super::types::{IpcResult, PipeId};
use super::pipe::Pipe;
use super::types::{
    PipeError, PipeStats, DEFAULT_PIPE_CAPACITY, MAX_PIPES_PER_PROCESS, MAX_PIPE_CAPACITY,
};
use crate::core::sync::WaitQueue;
use crate::core::types::{Pid, Size};
use crate::core::{ShardManager, WorkloadProfile};
use crate::memory::MemoryManager;
use crate::monitoring::Collector;
use ahash::RandomState;
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use log::{info, warn};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Pipe manager
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic ID counter
/// - Lock-free queue for ID recycling (hot path optimization)
/// - next_id wrapped in Arc to ensure ID uniqueness across clones (prevents collision bug)
/// - Futex-based wait/notify for efficient blocking I/O (zero CPU spinning on Linux)
#[repr(C, align(64))]
pub struct PipeManager {
    pipes: Arc<DashMap<PipeId, Pipe, RandomState>>,
    next_id: Arc<AtomicU32>,
    // Track pipe count per process
    process_pipes: Arc<DashMap<Pid, Size, RandomState>>,
    memory_manager: MemoryManager,
    // Lock-free queue for ID recycling (prevents ID exhaustion)
    free_ids: Arc<SegQueue<PipeId>>,
    // Wait queue for blocking I/O (futex on Linux, condvar elsewhere)
    wait_queue: Arc<WaitQueue<PipeId>>,
    // Observability collector
    collector: Option<Arc<Collector>>,
}

impl PipeManager {
    pub fn new(memory_manager: MemoryManager) -> Self {
        info!(
            "Pipe manager initialized with lock-free ID recycling and futex-based wait/notify (capacity: {})",
            DEFAULT_PIPE_CAPACITY
        );
        Self {
            // CPU-topology-aware shard counts for optimal concurrent performance
            pipes: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                ShardManager::shards(WorkloadProfile::MediumContention), // pipes: moderate I/O contention
            ).into()),
            next_id: Arc::new(AtomicU32::new(1)),
            process_pipes: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                ShardManager::shards(WorkloadProfile::LowContention), // per-process tracking: light access
            ).into()),
            memory_manager,
            free_ids: Arc::new(SegQueue::new().into()),
            // Use long_wait config for pipe I/O (typically 1-30s waits, futex optimal)
            wait_queue: Arc::new(WaitQueue::long_wait().into()),
            collector: None,
        }
    }

    /// Add observability collector
    pub fn with_collector(mut self, collector: Arc<Collector>) -> Self {
        self.collector = Some(collector);
        self
    }

    /// Set collector after construction
    pub fn set_collector(&mut self, collector: Arc<Collector>) {
        self.collector = Some(collector);
    }

    /// Get the wait queue for external blocking operations
    ///
    /// # Usage
    ///
    /// Used by `TimeoutPipeOps` and other wrappers to implement blocking I/O with timeouts.
    /// The PipeManager will automatically wake waiters when data/space becomes available.
    pub fn wait_queue(&self) -> Arc<WaitQueue<PipeId>> {
        Arc::clone(&self.wait_queue)
    }

    pub fn create(
        &self,
        reader_pid: Pid,
        writer_pid: Pid,
        capacity: Option<Size>,
    ) -> Result<PipeId, PipeError> {
        let capacity = capacity
            .unwrap_or(DEFAULT_PIPE_CAPACITY)
            .min(MAX_PIPE_CAPACITY);

        // Check per-process limits
        let reader_count = self
            .process_pipes
            .get(&reader_pid)
            .map(|r| *r.value())
            .unwrap_or(0);
        let writer_count = self
            .process_pipes
            .get(&writer_pid)
            .map(|r| *r.value())
            .unwrap_or(0);

        if reader_count >= MAX_PIPES_PER_PROCESS || writer_count >= MAX_PIPES_PER_PROCESS {
            return Err(PipeError::ProcessLimitExceeded(
                reader_count.max(writer_count),
                MAX_PIPES_PER_PROCESS,
            ));
        }

        // Allocate memory through MemoryManager (unified memory accounting)
        // Use writer_pid as the owner for accounting purposes
        // MemoryManager will handle global memory limits
        let address = self
            .memory_manager
            .allocate(capacity, writer_pid)
            .map_err(|e| PipeError::AllocationFailed(e.to_string().into()))?;

        // Try to recycle an ID from the lock-free free list, otherwise allocate new
        let pipe_id = if let Some(recycled_id) = self.free_ids.pop() {
            info!(
                "Recycled pipe ID {} for PIDs {}/{} (lock-free)",
                recycled_id, reader_pid, writer_pid
            );
            recycled_id
        } else {
            self.next_id.fetch_add(1, Ordering::SeqCst)
        };

        let pipe = Pipe::new(
            pipe_id,
            reader_pid,
            writer_pid,
            capacity,
            address,
            self.memory_manager.clone(),
        );
        self.pipes.insert(pipe_id, pipe);

        // Update process pipe counts using entry() for atomic increment
        *self.process_pipes.entry(reader_pid).or_insert(0) += 1;
        *self.process_pipes.entry(writer_pid).or_insert(0) += 1;

        let (_, used, _) = self.memory_manager.info();
        info!(
            "Created pipe {} (reader: {}, writer: {}, capacity: {} bytes, address: 0x{:x}, {} bytes used memory)",
            pipe_id, reader_pid, writer_pid, capacity, address, used
        );

        // Emit pipe created event
        if let Some(ref collector) = self.collector {
            use crate::monitoring::{Category, Event, Payload, Severity};
            collector.emit(
                Event::new(
                    Severity::Debug,
                    Category::Ipc,
                    Payload::MessageSent {
                        queue_id: pipe_id as u64,
                        size: capacity,
                    },
                )
                .with_pid(writer_pid),
            );
        }

        Ok(pipe_id)
    }

    pub fn write(&self, pipe_id: PipeId, pid: Pid, data: &[u8]) -> Result<Size, PipeError> {
        let mut pipe = self
            .pipes
            .get_mut(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        if pipe.writer_pid != pid {
            return Err(PipeError::PermissionDenied("Not the write end".to_string().into()));
        }

        let written = pipe.write(data)?;
        let buffered = pipe.buffered();

        info!(
            "Pipe {} write: {} bytes ({} buffered)",
            pipe_id, written, buffered
        );

        // Emit pipe write event
        if let Some(ref collector) = self.collector {
            use crate::monitoring::{Category, Event, Payload, Severity};
            collector.emit(
                Event::new(
                    Severity::Debug,
                    Category::Ipc,
                    Payload::MessageSent {
                        queue_id: pipe_id as u64,
                        size: written,
                    },
                )
                .with_pid(pid),
            );
        }

        // CRITICAL FIX: Wake readers waiting for data
        // Uses futex on Linux (zero CPU spinning), condvar elsewhere
        drop(pipe); // Release lock before wake to reduce contention
        self.wait_queue.wake_one(pipe_id);

        Ok(written)
    }

    pub fn read(&self, pipe_id: PipeId, pid: Pid, size: Size) -> Result<Vec<u8>, PipeError> {
        let mut pipe = self
            .pipes
            .get_mut(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        if pipe.reader_pid != pid {
            return Err(PipeError::PermissionDenied("Not the read end".to_string().into()));
        }

        let data = pipe.read(size)?;
        let buffered = pipe.buffered();

        info!(
            "Pipe {} read: {} bytes ({} remaining)",
            pipe_id,
            data.len(),
            buffered
        );

        // Emit pipe read event
        if let Some(ref collector) = self.collector {
            use crate::monitoring::{Category, Event, Payload, Severity};
            collector.emit(
                Event::new(
                    Severity::Debug,
                    Category::Ipc,
                    Payload::MessageReceived {
                        queue_id: pipe_id as u64,
                        size: data.len(),
                        wait_time_us: 0, // Pipe reads are synchronous
                    },
                )
                .with_pid(pid),
            );
        }

        // CRITICAL FIX: Wake writers waiting for space
        // Uses futex on Linux (zero CPU spinning), condvar elsewhere
        drop(pipe); // Release lock before wake to reduce contention
        self.wait_queue.wake_one(pipe_id);
        Ok(data)
    }

    pub fn close(&self, pipe_id: PipeId, pid: Pid) -> Result<(), PipeError> {
        let mut pipe = self
            .pipes
            .get_mut(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        if pipe.reader_pid != pid && pipe.writer_pid != pid {
            return Err(PipeError::PermissionDenied(
                "Not a pipe endpoint".to_string(),
            ));
        }

        pipe.closed = true;

        info!("Closed pipe {} by PID {}", pipe_id, pid);

        // Wake all waiters on close (they should check closed flag and return EOF/error)
        drop(pipe); // Release lock before wake
        self.wait_queue.wake_all(pipe_id);

        Ok(())
    }

    pub fn destroy(&self, pipe_id: PipeId) -> Result<(), PipeError> {
        let (_, pipe) = self
            .pipes
            .remove(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        let capacity = pipe.capacity;
        let address = pipe.address;
        let reader_pid = pipe.reader_pid;
        let writer_pid = pipe.writer_pid;

        // Deallocate memory through MemoryManager (unified memory accounting)
        if let Err(e) = self.memory_manager.deallocate(address) {
            warn!(
                "Failed to deallocate memory for pipe {} at address 0x{:x}: {}",
                pipe_id, address, e
            );
        }

        // Add pipe ID to lock-free free list for recycling
        self.free_ids.push(pipe_id);
        info!(
            "Added pipe ID {} to lock-free free list for recycling",
            pipe_id
        );

        // Update process pipe counts using entry() for atomic decrement
        if let Some(mut entry) = self.process_pipes.get_mut(&reader_pid) {
            *entry = entry.saturating_sub(1);
            if *entry == 0 {
                drop(entry);
                self.process_pipes.remove(&reader_pid);
            }
        }
        if let Some(mut entry) = self.process_pipes.get_mut(&writer_pid) {
            *entry = entry.saturating_sub(1);
            if *entry == 0 {
                drop(entry);
                self.process_pipes.remove(&writer_pid);
            }
        }

        let (_, used, _) = self.memory_manager.info();
        info!(
            "Destroyed pipe {} (reclaimed {} bytes at 0x{:x}, {} bytes used memory)",
            pipe_id, capacity, address, used
        );

        Ok(())
    }

    pub fn stats(&self, pipe_id: PipeId) -> Result<PipeStats, PipeError> {
        let pipe = self
            .pipes
            .get(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        Ok(PipeStats {
            id: pipe.id,
            reader_pid: pipe.reader_pid,
            writer_pid: pipe.writer_pid,
            capacity: pipe.capacity,
            buffered: pipe.buffered(),
            closed: pipe.closed,
        })
    }

    pub fn cleanup_process(&self, pid: Pid) -> Size {
        let pipe_ids: Vec<u32> = self
            .pipes
            .iter()
            .filter(|entry| entry.reader_pid == pid || entry.writer_pid == pid)
            .map(|entry| entry.id)
            .collect();

        let count = pipe_ids.len();

        for pipe_id in pipe_ids {
            if let Err(e) = self.destroy(pipe_id) {
                warn!("Failed to destroy pipe {} during cleanup: {}", pipe_id, e);
            }
        }

        if count > 0 {
            info!("Cleaned up {} pipes for PID {}", count, pid);
        }

        count
    }

    pub fn get_global_memory_usage(&self) -> Size {
        let (_, used, _) = self.memory_manager.info();
        used
    }
}

impl Clone for PipeManager {
    fn clone(&self) -> Self {
        Self {
            pipes: Arc::clone(&self.pipes),
            next_id: Arc::clone(&self.next_id), // Share ID counter to prevent collision
            process_pipes: Arc::clone(&self.process_pipes),
            memory_manager: self.memory_manager.clone(),
            free_ids: Arc::clone(&self.free_ids),
            wait_queue: Arc::clone(&self.wait_queue), // Share wait queue across clones
            collector: self.collector.as_ref().map(Arc::clone),
        }
    }
}

// Note: Default trait removed - PipeManager now requires MemoryManager dependency

// Implement PipeChannel trait
impl PipeChannel for PipeManager {
    fn create(
        &self,
        reader_pid: Pid,
        writer_pid: Pid,
        capacity: Option<Size>,
    ) -> IpcResult<PipeId> {
        self.create(reader_pid, writer_pid, capacity)
            .map_err(|e| e.into())
    }

    fn write(&self, pipe_id: PipeId, pid: Pid, data: &[u8]) -> IpcResult<Size> {
        self.write(pipe_id, pid, data).map_err(|e| e.into())
    }

    fn read(&self, pipe_id: PipeId, pid: Pid, size: Size) -> IpcResult<Vec<u8>> {
        self.read(pipe_id, pid, size).map_err(|e| e.into())
    }

    fn close(&self, pipe_id: PipeId, pid: Pid) -> IpcResult<()> {
        self.close(pipe_id, pid).map_err(|e| e.into())
    }

    fn destroy(&self, pipe_id: PipeId) -> IpcResult<()> {
        self.destroy(pipe_id).map_err(|e| e.into())
    }

    fn stats(&self, pipe_id: PipeId) -> IpcResult<PipeStats> {
        self.stats(pipe_id).map_err(|e| e.into())
    }
}
