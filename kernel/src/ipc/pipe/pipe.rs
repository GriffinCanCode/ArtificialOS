/*!
 * Pipe Implementation
 * Core pipe data structure
 */

use super::types::PipeError;
use super::super::types::PipeId;
use crate::core::types::{Address, Pid, Size};
use crate::memory::MemoryManager;
use std::collections::VecDeque;

pub(super) struct Pipe {
    pub id: PipeId,
    pub reader_pid: Pid,
    pub writer_pid: Pid,
    /// Memory address allocated through MemoryManager
    pub address: Address,
    /// Internal buffer for data flow (backed by allocated memory)
    pub buffer: VecDeque<u8>,
    pub capacity: Size,
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
            .field("buffer_len", &self.buffer.len())
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
        Self {
            id,
            reader_pid,
            writer_pid,
            address,
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            memory_manager,
            closed: false,
        }
    }

    pub fn available_space(&self) -> Size {
        self.capacity.saturating_sub(self.buffer.len())
    }

    pub fn buffered(&self) -> Size {
        self.buffer.len()
    }

    pub fn write(&mut self, data: &[u8]) -> Result<Size, PipeError> {
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

    pub fn read(&mut self, size: Size) -> Result<Vec<u8>, PipeError> {
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
