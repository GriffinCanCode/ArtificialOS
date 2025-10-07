/*!
 * Buffer Pool
 * Pre-allocated buffer pool for zero-copy operations
 */

use crate::core::types::{Pid, Size, Address};
use crate::memory::MemoryManager;
use parking_lot::Mutex;
use std::collections::VecDeque;
use tracing::debug;

/// Pre-allocated buffer sizes
const SMALL_BUFFER_SIZE: Size = 4096;      // 4KB
const MEDIUM_BUFFER_SIZE: Size = 65536;    // 64KB
const LARGE_BUFFER_SIZE: Size = 1048576;   // 1MB

/// Buffer pool for zero-copy operations
pub struct BufferPool {
    pid: Pid,
    memory_manager: MemoryManager,
    small_buffers: Mutex<VecDeque<Buffer>>,
    medium_buffers: Mutex<VecDeque<Buffer>>,
    large_buffers: Mutex<VecDeque<Buffer>>,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(pid: Pid, memory_manager: MemoryManager) -> Self {
        Self {
            pid,
            memory_manager,
            small_buffers: Mutex::new(VecDeque::new()),
            medium_buffers: Mutex::new(VecDeque::new()),
            large_buffers: Mutex::new(VecDeque::new()),
        }
    }

    /// Acquire a buffer from the pool
    pub fn acquire(&self, size: Size) -> Result<Buffer, String> {
        // Determine buffer size category
        let (pool, buffer_size) = if size <= SMALL_BUFFER_SIZE {
            (&self.small_buffers, SMALL_BUFFER_SIZE)
        } else if size <= MEDIUM_BUFFER_SIZE {
            (&self.medium_buffers, MEDIUM_BUFFER_SIZE)
        } else {
            (&self.large_buffers, LARGE_BUFFER_SIZE)
        };

        // Try to get from pool
        {
            let mut pool = pool.lock();
            if let Some(buffer) = pool.pop_front() {
                debug!(
                    pid = self.pid,
                    size = size,
                    buffer_size = buffer_size,
                    "Acquired buffer from pool"
                );
                return Ok(buffer);
            }
        }

        // Allocate new buffer
        let address = self
            .memory_manager
            .allocate(buffer_size, self.pid)
            .map_err(|e| format!("Failed to allocate buffer: {}", e))?;

        debug!(
            pid = self.pid,
            size = size,
            buffer_size = buffer_size,
            address = format!("0x{:x}", address),
            "Allocated new buffer"
        );

        Ok(Buffer {
            address,
            size: buffer_size,
        })
    }

    /// Release a buffer back to the pool
    pub fn release(&self, buffer: Buffer) {
        let buffer_size = buffer.size;

        let pool = if buffer_size == SMALL_BUFFER_SIZE {
            &self.small_buffers
        } else if buffer_size == MEDIUM_BUFFER_SIZE {
            &self.medium_buffers
        } else {
            &self.large_buffers
        };

        let mut pool = pool.lock();
        pool.push_back(buffer);

        debug!(
            pid = self.pid,
            buffer_size = buffer_size,
            "Released buffer to pool"
        );
    }

    /// Get pool statistics
    pub fn stats(&self) -> BufferPoolStats {
        BufferPoolStats {
            small_available: self.small_buffers.lock().len(),
            medium_available: self.medium_buffers.lock().len(),
            large_available: self.large_buffers.lock().len(),
        }
    }
}

/// A pre-allocated buffer
#[derive(Debug, Clone)]
pub struct Buffer {
    pub address: Address,
    pub size: Size,
}

/// Buffer pool statistics
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    pub small_available: usize,
    pub medium_available: usize,
    pub large_available: usize,
}

