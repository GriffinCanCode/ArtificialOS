/*!
 * Pipe Module
 * Unix-style pipes for streaming data between processes
 */

use log::{info, warn};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use thiserror::Error;

// Pipe limits to prevent resource exhaustion
const DEFAULT_PIPE_CAPACITY: usize = 65536; // 64KB (Linux default)
const MAX_PIPE_CAPACITY: usize = 1024 * 1024; // 1MB max
const MAX_PIPES_PER_PROCESS: usize = 100;
const GLOBAL_PIPE_MEMORY_LIMIT: usize = 50 * 1024 * 1024; // 50MB total

// Global pipe memory tracking
static GLOBAL_PIPE_MEMORY: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Error)]
pub enum PipeError {
    #[error("Pipe not found: {0}")]
    NotFound(u32),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Pipe closed")]
    Closed,

    #[error("Would block: {0}")]
    WouldBlock(String),

    #[error("Capacity exceeded: requested {requested}, capacity {capacity}")]
    CapacityExceeded { requested: usize, capacity: usize },

    #[error("Process pipe limit exceeded: {0}/{1}")]
    ProcessLimitExceeded(usize, usize),

    #[error("Global pipe memory limit exceeded: {0}/{1} bytes")]
    GlobalMemoryExceeded(usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipeStats {
    pub id: u32,
    pub reader_pid: u32,
    pub writer_pid: u32,
    pub capacity: usize,
    pub buffered: usize,
    pub closed: bool,
}

#[derive(Debug)]
struct Pipe {
    id: u32,
    reader_pid: u32,
    writer_pid: u32,
    buffer: VecDeque<u8>,
    capacity: usize,
    closed: bool,
}

impl Pipe {
    fn new(id: u32, reader_pid: u32, writer_pid: u32, capacity: usize) -> Self {
        Self {
            id,
            reader_pid,
            writer_pid,
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            closed: false,
        }
    }

    fn available_space(&self) -> usize {
        self.capacity.saturating_sub(self.buffer.len())
    }

    fn buffered(&self) -> usize {
        self.buffer.len()
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, PipeError> {
        if self.closed {
            return Err(PipeError::Closed);
        }

        let available = self.available_space();
        if available == 0 {
            return Err(PipeError::WouldBlock("Pipe buffer full".to_string()));
        }

        let to_write = data.len().min(available);
        self.buffer.extend(&data[..to_write]);

        Ok(to_write)
    }

    fn read(&mut self, size: usize) -> Result<Vec<u8>, PipeError> {
        if self.buffer.is_empty() {
            if self.closed {
                return Ok(Vec::new()); // EOF
            }
            return Err(PipeError::WouldBlock("No data available".to_string()));
        }

        let to_read = size.min(self.buffer.len());
        let data: Vec<u8> = self.buffer.drain(..to_read).collect();

        Ok(data)
    }
}

pub struct PipeManager {
    pipes: Arc<RwLock<HashMap<u32, Pipe>>>,
    next_id: Arc<RwLock<u32>>,
    // Track pipe count per process
    process_pipes: Arc<RwLock<HashMap<u32, usize>>>,
}

impl PipeManager {
    pub fn new() -> Self {
        info!(
            "Pipe manager initialized (capacity: {}, limit: {} MB)",
            DEFAULT_PIPE_CAPACITY,
            GLOBAL_PIPE_MEMORY_LIMIT / (1024 * 1024)
        );
        Self {
            pipes: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            process_pipes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create(
        &self,
        reader_pid: u32,
        writer_pid: u32,
        capacity: Option<usize>,
    ) -> Result<u32, PipeError> {
        let capacity = capacity.unwrap_or(DEFAULT_PIPE_CAPACITY).min(MAX_PIPE_CAPACITY);

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

        let mut pipes = self.pipes.write();
        let mut next_id = self.next_id.write();

        let pipe_id = *next_id;
        *next_id += 1;

        let pipe = Pipe::new(pipe_id, reader_pid, writer_pid, capacity);
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
            "Created pipe {} (reader: {}, writer: {}, capacity: {} bytes)",
            pipe_id, reader_pid, writer_pid, capacity
        );

        Ok(pipe_id)
    }

    pub fn write(&self, pipe_id: u32, pid: u32, data: &[u8]) -> Result<usize, PipeError> {
        let mut pipes = self.pipes.write();
        let pipe = pipes
            .get_mut(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        if pipe.writer_pid != pid {
            return Err(PipeError::PermissionDenied(
                "Not the write end".to_string(),
            ));
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

    pub fn read(&self, pipe_id: u32, pid: u32, size: usize) -> Result<Vec<u8>, PipeError> {
        let mut pipes = self.pipes.write();
        let pipe = pipes
            .get_mut(&pipe_id)
            .ok_or(PipeError::NotFound(pipe_id))?;

        if pipe.reader_pid != pid {
            return Err(PipeError::PermissionDenied(
                "Not the read end".to_string(),
            ));
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

    pub fn close(&self, pipe_id: u32, pid: u32) -> Result<(), PipeError> {
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

    pub fn destroy(&self, pipe_id: u32) -> Result<(), PipeError> {
        let mut pipes = self.pipes.write();
        let pipe = pipes.remove(&pipe_id).ok_or(PipeError::NotFound(pipe_id))?;

        // Update process pipe counts
        let mut process_pipes = self.process_pipes.write();
        if let Some(count) = process_pipes.get_mut(&pipe.reader_pid) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                process_pipes.remove(&pipe.reader_pid);
            }
        }
        if let Some(count) = process_pipes.get_mut(&pipe.writer_pid) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                process_pipes.remove(&pipe.writer_pid);
            }
        }
        drop(process_pipes);

        // Reclaim global memory
        GLOBAL_PIPE_MEMORY.fetch_sub(pipe.capacity, Ordering::Release);

        info!(
            "Destroyed pipe {} (reclaimed {} bytes, {} bytes global memory)",
            pipe_id,
            pipe.capacity,
            GLOBAL_PIPE_MEMORY.load(Ordering::Relaxed)
        );

        Ok(())
    }

    pub fn stats(&self, pipe_id: u32) -> Result<PipeStats, PipeError> {
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

    pub fn cleanup_process(&self, pid: u32) -> usize {
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

    pub fn get_global_memory_usage(&self) -> usize {
        GLOBAL_PIPE_MEMORY.load(Ordering::Relaxed)
    }
}

impl Clone for PipeManager {
    fn clone(&self) -> Self {
        Self {
            pipes: Arc::clone(&self.pipes),
            next_id: Arc::clone(&self.next_id),
            process_pipes: Arc::clone(&self.process_pipes),
        }
    }
}

impl Default for PipeManager {
    fn default() -> Self {
        Self::new()
    }
}
