/*!
 * Pipe Manager
 * Central manager for Unix-style pipes
 */

use super::super::traits::PipeChannel;
use super::super::types::{IpcResult, PipeId};
use super::pipe::Pipe;
use super::types::{
    PipeError, PipeStats, DEFAULT_PIPE_CAPACITY, GLOBAL_PIPE_MEMORY_LIMIT, MAX_PIPES_PER_PROCESS,
    MAX_PIPE_CAPACITY,
};
use crate::core::types::{Pid, Size};
use crate::memory::MemoryManager;
use log::{info, warn};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Global pipe memory tracking
static GLOBAL_PIPE_MEMORY: AtomicUsize = AtomicUsize::new(0);

/// Pipe manager
pub struct PipeManager {
    pipes: Arc<RwLock<HashMap<PipeId, Pipe>>>,
    next_id: Arc<RwLock<PipeId>>,
    // Track pipe count per process
    process_pipes: Arc<RwLock<HashMap<Pid, Size>>>,
    memory_manager: MemoryManager,
}

impl PipeManager {
    pub fn new(memory_manager: MemoryManager) -> Self {
        info!(
            "Pipe manager initialized (capacity: {}, limit: {} MB)",
            DEFAULT_PIPE_CAPACITY,
            GLOBAL_PIPE_MEMORY_LIMIT / (1024 * 1024)
        );
        Self {
            pipes: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            process_pipes: Arc::new(RwLock::new(HashMap::new())),
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
        let process_pipes = self.process_pipes.read();
        let reader_count = process_pipes.get(&reader_pid).unwrap_or(&0);
        let writer_count = process_pipes.get(&writer_pid).unwrap_or(&0);

        if *reader_count >= MAX_PIPES_PER_PROCESS || *writer_count >= MAX_PIPES_PER_PROCESS {
            return Err(PipeError::ProcessLimitExceeded(
                (*reader_count).max(*writer_count),
                MAX_PIPES_PER_PROCESS,
            ));
        }
        drop(process_pipes);

        // Check global memory limit
        let current_global = GLOBAL_PIPE_MEMORY.load(Ordering::Acquire);
        if current_global + capacity > GLOBAL_PIPE_MEMORY_LIMIT {
            return Err(PipeError::GlobalMemoryExceeded(
                current_global,
                GLOBAL_PIPE_MEMORY_LIMIT,
            ));
        }

        // Allocate memory through MemoryManager (unified memory accounting)
        // Use writer_pid as the owner for accounting purposes
        let address = self
            .memory_manager
            .allocate(capacity, writer_pid)
            .map_err(|e| PipeError::AllocationFailed(e.to_string()))?;

        let mut pipes = self.pipes.write();
        let mut next_id = self.next_id.write();

        let pipe_id = *next_id;
        *next_id += 1;

        let pipe = Pipe::new(
            pipe_id,
            reader_pid,
            writer_pid,
            capacity,
            address,
            self.memory_manager.clone(),
        );
        pipes.insert(pipe_id, pipe);
        drop(pipes);
        drop(next_id);

        // Update process pipe counts
        let mut process_pipes = self.process_pipes.write();
        *process_pipes.entry(reader_pid).or_insert(0) += 1;
        *process_pipes.entry(writer_pid).or_insert(0) += 1;
        drop(process_pipes);

        // Update global memory
        GLOBAL_PIPE_MEMORY.fetch_add(capacity, Ordering::Release);

        info!(
            "Created pipe {} (reader: {}, writer: {}, capacity: {} bytes, address: 0x{:x})",
            pipe_id, reader_pid, writer_pid, capacity, address
        );

        Ok(pipe_id)
    }

    pub fn write(&self, pipe_id: PipeId, pid: Pid, data: &[u8]) -> Result<Size, PipeError> {
        let mut pipes = self.pipes.write();
        let pipe = pipes
            .get_mut(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        if pipe.writer_pid != pid {
            return Err(PipeError::PermissionDenied("Not the write end".to_string()));
        }

        let written = pipe.write(data)?;

        info!(
            "Pipe {} write: {} bytes ({} buffered)",
            pipe_id,
            written,
            pipe.buffered()
        );

        Ok(written)
    }

    pub fn read(&self, pipe_id: PipeId, pid: Pid, size: Size) -> Result<Vec<u8>, PipeError> {
        let mut pipes = self.pipes.write();
        let pipe = pipes
            .get_mut(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        if pipe.reader_pid != pid {
            return Err(PipeError::PermissionDenied("Not the read end".to_string()));
        }

        let data = pipe.read(size)?;

        info!(
            "Pipe {} read: {} bytes ({} remaining)",
            pipe_id,
            data.len(),
            pipe.buffered()
        );

        Ok(data)
    }

    pub fn close(&self, pipe_id: PipeId, pid: Pid) -> Result<(), PipeError> {
        let mut pipes = self.pipes.write();
        let pipe = pipes
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
        let mut pipes = self.pipes.write();
        let pipe = pipes.remove(&pipe_id).ok_or(PipeError::NotFound(pipe_id))?;

        let capacity = pipe.capacity;
        let address = pipe.address;
        let reader_pid = pipe.reader_pid;
        let writer_pid = pipe.writer_pid;

        drop(pipes);

        // Deallocate memory through MemoryManager (unified memory accounting)
        if let Err(e) = self.memory_manager.deallocate(address) {
            warn!(
                "Failed to deallocate memory for pipe {} at address 0x{:x}: {}",
                pipe_id, address, e
            );
        }

        // Update process pipe counts
        let mut process_pipes = self.process_pipes.write();
        if let Some(count) = process_pipes.get_mut(&reader_pid) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                process_pipes.remove(&reader_pid);
            }
        }
        if let Some(count) = process_pipes.get_mut(&writer_pid) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                process_pipes.remove(&writer_pid);
            }
        }
        drop(process_pipes);

        // Reclaim global memory
        GLOBAL_PIPE_MEMORY.fetch_sub(capacity, Ordering::Release);

        info!(
            "Destroyed pipe {} (reclaimed {} bytes at 0x{:x}, {} bytes global memory)",
            pipe_id,
            capacity,
            address,
            GLOBAL_PIPE_MEMORY.load(Ordering::Relaxed)
        );

        Ok(())
    }

    pub fn stats(&self, pipe_id: PipeId) -> Result<PipeStats, PipeError> {
        let pipes = self.pipes.read();
        let pipe = pipes.get(&pipe_id).ok_or(PipeError::NotFound(pipe_id))?;

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
        let pipes = self.pipes.read();
        let pipe_ids: Vec<u32> = pipes
            .values()
            .filter(|p| p.reader_pid == pid || p.writer_pid == pid)
            .map(|p| p.id)
            .collect();
        drop(pipes);

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
        GLOBAL_PIPE_MEMORY.load(Ordering::Relaxed)
    }
}

impl Clone for PipeManager {
    fn clone(&self) -> Self {
        Self {
            pipes: Arc::clone(&self.pipes),
            next_id: Arc::clone(&self.next_id),
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
