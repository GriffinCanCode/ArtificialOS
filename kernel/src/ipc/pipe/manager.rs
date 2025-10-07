/*!
 * Pipe Manager
 * Central manager for Unix-style pipes
 */

use super::super::traits::PipeChannel;
use super::super::types::{IpcResult, PipeId};
use super::pipe::Pipe;
use super::types::{
    PipeError, PipeStats, DEFAULT_PIPE_CAPACITY, MAX_PIPES_PER_PROCESS,
    MAX_PIPE_CAPACITY,
};
use crate::core::types::{Pid, Size};
use crate::memory::MemoryManager;
use dashmap::DashMap;
use log::{info, warn};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// Pipe manager
pub struct PipeManager {
    pipes: Arc<DashMap<PipeId, Pipe>>,
    next_id: AtomicU32,
    // Track pipe count per process
    process_pipes: Arc<DashMap<Pid, Size>>,
    memory_manager: MemoryManager,
}

impl PipeManager {
    pub fn new(memory_manager: MemoryManager) -> Self {
        info!(
            "Pipe manager initialized (capacity: {})",
            DEFAULT_PIPE_CAPACITY
        );
        Self {
            // Use 64 shards for pipes - high I/O contention
            pipes: Arc::new(DashMap::with_shard_amount(64)),
            next_id: AtomicU32::new(1),
            // Use 32 shards for process pipe tracking
            process_pipes: Arc::new(DashMap::with_shard_amount(32)),
            memory_manager,
        }
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
        let reader_count = self.process_pipes.get(&reader_pid).map(|r| *r.value()).unwrap_or(0);
        let writer_count = self.process_pipes.get(&writer_pid).map(|r| *r.value()).unwrap_or(0);

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
            .map_err(|e| PipeError::AllocationFailed(e.to_string()))?;

        let pipe_id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let pipe = Pipe::new(
            pipe_id,
            reader_pid,
            writer_pid,
            capacity,
            address,
            self.memory_manager.clone(),
        );
        self.pipes.insert(pipe_id, pipe);

        // Update process pipe counts using alter() for atomic increment
        self.process_pipes.alter(&reader_pid, |_, count| count + 1);
        self.process_pipes.alter(&writer_pid, |_, count| count + 1);

        let (_, used, _) = self.memory_manager.info();
        info!(
            "Created pipe {} (reader: {}, writer: {}, capacity: {} bytes, address: 0x{:x}, {} bytes used memory)",
            pipe_id, reader_pid, writer_pid, capacity, address, used
        );

        Ok(pipe_id)
    }

    pub fn write(&self, pipe_id: PipeId, pid: Pid, data: &[u8]) -> Result<Size, PipeError> {
        let mut pipe = self.pipes
            .get_mut(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        if pipe.writer_pid != pid {
            return Err(PipeError::PermissionDenied("Not the write end".to_string()));
        }

        let written = pipe.write(data)?;
        let buffered = pipe.buffered();

        info!(
            "Pipe {} write: {} bytes ({} buffered)",
            pipe_id,
            written,
            buffered
        );

        Ok(written)
    }

    pub fn read(&self, pipe_id: PipeId, pid: Pid, size: Size) -> Result<Vec<u8>, PipeError> {
        let mut pipe = self.pipes
            .get_mut(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        if pipe.reader_pid != pid {
            return Err(PipeError::PermissionDenied("Not the read end".to_string()));
        }

        let data = pipe.read(size)?;
        let buffered = pipe.buffered();

        info!(
            "Pipe {} read: {} bytes ({} remaining)",
            pipe_id,
            data.len(),
            buffered
        );

        Ok(data)
    }

    pub fn close(&self, pipe_id: PipeId, pid: Pid) -> Result<(), PipeError> {
        let mut pipe = self.pipes
            .get_mut(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        if pipe.reader_pid != pid && pipe.writer_pid != pid {
            return Err(PipeError::PermissionDenied(
                "Not a pipe endpoint".to_string(),
            ));
        }

        pipe.closed = true;

        info!("Closed pipe {} by PID {}", pipe_id, pid);

        Ok(())
    }

    pub fn destroy(&self, pipe_id: PipeId) -> Result<(), PipeError> {
        let (_, pipe) = self.pipes.remove(&pipe_id).ok_or(PipeError::NotFound(pipe_id))?;

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

        // Update process pipe counts using alter() for atomic decrement
        self.process_pipes.alter(&reader_pid, |_, count| {
            let new_count = count.saturating_sub(1);
            if new_count == 0 {
                // Signal for removal by returning 0
                0
            } else {
                new_count
            }
        });
        self.process_pipes.alter(&writer_pid, |_, count| {
            let new_count = count.saturating_sub(1);
            if new_count == 0 {
                0
            } else {
                new_count
            }
        });

        // Remove zero-count entries
        if let Some(entry) = self.process_pipes.get(&reader_pid) {
            if *entry.value() == 0 {
                drop(entry);
                self.process_pipes.remove(&reader_pid);
            }
        }
        if let Some(entry) = self.process_pipes.get(&writer_pid) {
            if *entry.value() == 0 {
                drop(entry);
                self.process_pipes.remove(&writer_pid);
            }
        }

        let (_, used, _) = self.memory_manager.info();
        info!(
            "Destroyed pipe {} (reclaimed {} bytes at 0x{:x}, {} bytes used memory)",
            pipe_id,
            capacity,
            address,
            used
        );

        Ok(())
    }

    pub fn stats(&self, pipe_id: PipeId) -> Result<PipeStats, PipeError> {
        let pipe = self.pipes.get(&pipe_id).ok_or(PipeError::NotFound(pipe_id))?;

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
        let pipe_ids: Vec<u32> = self.pipes
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
            next_id: AtomicU32::new(self.next_id.load(Ordering::SeqCst)),
            process_pipes: Arc::clone(&self.process_pipes),
            memory_manager: self.memory_manager.clone(),
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
