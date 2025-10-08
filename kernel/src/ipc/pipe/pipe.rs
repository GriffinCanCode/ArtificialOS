/*!
 * Pipe Implementation
 * Core pipe data structure with ringbuf-based circular buffer
 */

use super::super::core::types::PipeId;
use super::super::utils::lockfree_ring::LockFreeByteRing;
use super::types::PipeError;
use crate::core::types::{Address, Pid, Size};
use crate::memory::MemoryManager;

pub(super) struct Pipe {
    pub id: PipeId,
    pub reader_pid: Pid,
    pub writer_pid: Pid,
    /// Memory address allocated through MemoryManager (for accounting)
    pub address: Address,
    /// Lock-free ring buffer for zero-contention SPSC pipe operations
    buffer: LockFreeByteRing,
    pub capacity: Size,
    #[allow(dead_code)]
    pub memory_manager: MemoryManager,
    pub closed: bool,
}

impl std::fmt::Debug for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipe")
            .field("id", &self.id)
            .field("reader_pid", &self.reader_pid)
            .field("writer_pid", &self.writer_pid)
            .field("address", &format_args!("0x{:x}", self.address))
            .field("buffered_bytes", &self.buffer.buffered())
            .field("capacity", &self.capacity)
            .field("closed", &self.closed)
            .finish()
    }
}

impl Pipe {
    pub fn new(
        id: PipeId,
        reader_pid: Pid,
        writer_pid: Pid,
        capacity: Size,
        address: Address,
        memory_manager: MemoryManager,
    ) -> Self {
        // Create a lock-free ring buffer with the specified capacity
        let buffer = LockFreeByteRing::new(capacity);

        Self {
            id,
            reader_pid,
            writer_pid,
            address,
            buffer,
            capacity,
            memory_manager,
            closed: false,
        }
    }

    #[allow(dead_code)]
    pub fn available_space(&self) -> Size {
        self.buffer.available_space()
    }

    pub fn buffered(&self) -> Size {
        self.buffer.buffered()
    }

    pub fn write(&mut self, data: &[u8]) -> Result<Size, PipeError> {
        if self.closed {
            return Err(PipeError::Closed);
        }

        let available = self.buffer.available_space();

        if available == 0 {
            return Err(PipeError::WouldBlock("Pipe buffer full".to_string().into()));
        }

        // Lock-free write - zero contention in SPSC pattern
        let written = self.buffer.write(data);

        Ok(written)
    }

    pub fn read(&mut self, size: Size) -> Result<Vec<u8>, PipeError> {
        if self.buffer.is_empty() {
            if self.closed {
                return Ok(Vec::new()); // EOF
            }
            return Err(PipeError::WouldBlock(
                "No data available".to_string().into(),
            ));
        }

        // Lock-free read - zero contention in SPSC pattern
        let data = self.buffer.read(size);

        Ok(data)
    }
}
