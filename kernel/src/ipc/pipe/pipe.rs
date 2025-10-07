/*!
 * Pipe Implementation
 * Core pipe data structure with ringbuf-based circular buffer
 */

use super::super::types::PipeId;
use super::types::PipeError;
use crate::core::types::{Address, Pid, Size};
use crate::memory::MemoryManager;
use ringbuf::{traits::*, HeapRb};
use std::sync::{Arc, Mutex};

pub(super) struct Pipe {
    pub id: PipeId,
    pub reader_pid: Pid,
    pub writer_pid: Pid,
    /// Memory address allocated through MemoryManager (for accounting)
    pub address: Address,
    /// Ring buffer for efficient SPSC pipe operations
    buffer: Arc<Mutex<HeapRb<u8>>>,
    pub capacity: Size,
    pub memory_manager: MemoryManager,
    pub closed: bool,
}

impl std::fmt::Debug for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buffer = self.buffer.lock().unwrap();
        f.debug_struct("Pipe")
            .field("id", &self.id)
            .field("reader_pid", &self.reader_pid)
            .field("writer_pid", &self.writer_pid)
            .field("address", &format_args!("0x{:x}", self.address))
            .field("buffered_bytes", &buffer.occupied_len())
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
        // Create a ring buffer with the specified capacity
        let buffer = HeapRb::<u8>::new(capacity);

        Self {
            id,
            reader_pid,
            writer_pid,
            address,
            buffer: Arc::new(Mutex::new(buffer)),
            capacity,
            memory_manager,
            closed: false,
        }
    }

    pub fn available_space(&self) -> Size {
        let buffer = self.buffer.lock().unwrap();
        buffer.vacant_len()
    }

    pub fn buffered(&self) -> Size {
        let buffer = self.buffer.lock().unwrap();
        buffer.occupied_len()
    }

    pub fn write(&mut self, data: &[u8]) -> Result<Size, PipeError> {
        if self.closed {
            return Err(PipeError::Closed);
        }

        let mut buffer = self.buffer.lock().unwrap();
        let available = buffer.vacant_len();

        if available == 0 {
            return Err(PipeError::WouldBlock("Pipe buffer full".to_string()));
        }

        let to_write = data.len().min(available);

        // Use ringbuf's push_slice for writing
        let written = buffer.push_slice(&data[..to_write]);

        Ok(written)
    }

    pub fn read(&mut self, size: Size) -> Result<Vec<u8>, PipeError> {
        let mut buffer = self.buffer.lock().unwrap();

        if buffer.is_empty() {
            if self.closed {
                return Ok(Vec::new()); // EOF
            }
            return Err(PipeError::WouldBlock("No data available".to_string()));
        }

        let to_read = size.min(buffer.occupied_len());
        let mut data = vec![0u8; to_read];

        // Use ringbuf's pop_slice for reading
        let read = buffer.pop_slice(&mut data);
        data.truncate(read);

        Ok(data)
    }
}
