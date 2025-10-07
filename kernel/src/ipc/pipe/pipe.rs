/*!
 * Pipe Implementation
 * Core pipe data structure with unified memory storage
 */

use super::super::types::PipeId;
use super::types::PipeError;
use crate::core::types::{Address, Pid, Size};
use crate::memory::MemoryManager;

pub(super) struct Pipe {
    pub id: PipeId,
    pub reader_pid: Pid,
    pub writer_pid: Pid,
    /// Memory address allocated through MemoryManager (base of circular buffer)
    pub address: Address,
    /// Circular buffer read offset (where to read next)
    read_offset: Size,
    /// Circular buffer write offset (where to write next)
    write_offset: Size,
    /// Number of bytes currently buffered
    buffered_bytes: Size,
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
            .field("buffered_bytes", &self.buffered_bytes)
            .field("read_offset", &self.read_offset)
            .field("write_offset", &self.write_offset)
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
            read_offset: 0,
            write_offset: 0,
            buffered_bytes: 0,
            capacity,
            memory_manager,
            closed: false,
        }
    }

    pub fn available_space(&self) -> Size {
        self.capacity.saturating_sub(self.buffered_bytes)
    }

    pub fn buffered(&self) -> Size {
        self.buffered_bytes
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

        // Write to circular buffer via MemoryManager
        // Handle wrapping if necessary
        let bytes_to_end = self.capacity - self.write_offset;

        if to_write <= bytes_to_end {
            // Single contiguous write
            let write_addr = self.address + self.write_offset;
            self.memory_manager
                .write_bytes(write_addr, &data[..to_write])
                .map_err(|e| PipeError::AllocationFailed(e.to_string()))?;
        } else {
            // Write wraps around - split into two writes
            // First: write to end of buffer
            let write_addr = self.address + self.write_offset;
            self.memory_manager
                .write_bytes(write_addr, &data[..bytes_to_end])
                .map_err(|e| PipeError::AllocationFailed(e.to_string()))?;

            // Second: write remainder from beginning
            let remaining = to_write - bytes_to_end;
            self.memory_manager
                .write_bytes(self.address, &data[bytes_to_end..to_write])
                .map_err(|e| PipeError::AllocationFailed(e.to_string()))?;
        }

        // Update write offset (circular)
        self.write_offset = (self.write_offset + to_write) % self.capacity;
        self.buffered_bytes += to_write;

        Ok(to_write)
    }

    pub fn read(&mut self, size: Size) -> Result<Vec<u8>, PipeError> {
        if self.buffered_bytes == 0 {
            if self.closed {
                return Ok(Vec::new()); // EOF
            }
            return Err(PipeError::WouldBlock("No data available".to_string()));
        }

        let to_read = size.min(self.buffered_bytes);
        let mut data = Vec::with_capacity(to_read);

        // Read from circular buffer via MemoryManager
        // Handle wrapping if necessary
        let bytes_to_end = self.capacity - self.read_offset;

        if to_read <= bytes_to_end {
            // Single contiguous read
            let read_addr = self.address + self.read_offset;
            let bytes = self
                .memory_manager
                .read_bytes(read_addr, to_read)
                .map_err(|e| PipeError::AllocationFailed(e.to_string()))?;
            data.extend_from_slice(&bytes);
        } else {
            // Read wraps around - split into two reads
            // First: read to end of buffer
            let read_addr = self.address + self.read_offset;
            let bytes = self
                .memory_manager
                .read_bytes(read_addr, bytes_to_end)
                .map_err(|e| PipeError::AllocationFailed(e.to_string()))?;
            data.extend_from_slice(&bytes);

            // Second: read remainder from beginning
            let remaining = to_read - bytes_to_end;
            let bytes = self
                .memory_manager
                .read_bytes(self.address, remaining)
                .map_err(|e| PipeError::AllocationFailed(e.to_string()))?;
            data.extend_from_slice(&bytes);
        }

        // Update read offset (circular)
        self.read_offset = (self.read_offset + to_read) % self.capacity;
        self.buffered_bytes -= to_read;

        Ok(data)
    }
}
