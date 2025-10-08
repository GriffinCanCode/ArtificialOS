/*!
 * Memory Storage Operations
 * Read/write operations for simulated physical memory
 */

use super::super::types::{MemoryError, MemoryResult};
use super::MemoryManager;
use crate::core::types::{Address, Size};
use crate::memory::{simd_memcpy, simd_memset};
use log::info;

impl MemoryManager {
    /// Write bytes to a memory address
    /// This simulates writing to physical memory for shared memory segments
    pub fn write_bytes(&self, address: Address, data: &[u8]) -> MemoryResult<()> {
        // Find the block containing this address
        let mut base_addr = None;
        let mut block_size = 0;
        for entry in self.blocks.iter() {
            let addr = *entry.key();
            let block = entry.value();
            if block.allocated && address >= addr && address < addr + block.size {
                // Check if write fits within block bounds
                if address + data.len() <= addr + block.size {
                    base_addr = Some(addr);
                    block_size = block.size;
                    break;
                } else {
                    return Err(MemoryError::InvalidAddress(address));
                }
            }
        }

        if let Some(base_addr) = base_addr {
            // Calculate offset within the block
            let offset = address - base_addr;

            // Get or create storage for this block
            let mut entry = self
                .memory_storage
                .entry(base_addr)
                .or_insert_with(|| vec![0u8; block_size]);

            // Ensure block_data is large enough
            if entry.len() < block_size {
                entry.resize(block_size, 0u8);
            }

            // Write data at the offset using SIMD-accelerated copy
            let end = offset + data.len();
            simd_memcpy(&mut entry[offset..end], data);

            info!(
                "Wrote {} bytes to address 0x{:x} (offset {} in block at 0x{:x})",
                data.len(),
                address,
                offset,
                base_addr
            );
            Ok(())
        } else {
            Err(MemoryError::InvalidAddress(address))
        }
    }

    /// Read bytes from a memory address
    /// This simulates reading from physical memory for shared memory segments
    pub fn read_bytes(&self, address: Address, size: Size) -> MemoryResult<Vec<u8>> {
        // Find the block containing this address
        let mut base_addr = None;
        for entry in self.blocks.iter() {
            let addr = *entry.key();
            let block = entry.value();
            if block.allocated && address >= addr && address < addr + block.size {
                // Check if read fits within block bounds
                if address + size <= addr + block.size {
                    base_addr = Some(addr);
                    break;
                } else {
                    return Err(MemoryError::InvalidAddress(address));
                }
            }
        }

        if let Some(base_addr) = base_addr {
            // Calculate offset within the block
            let offset = address - base_addr;

            // Get storage for this block
            let data = if let Some(block_data) = self.memory_storage.get(&base_addr) {
                // Read data from the stored bytes using SIMD-accelerated copy
                let end = offset + size;
                let mut result = vec![0u8; size];
                simd_memcpy(&mut result, &block_data[offset..end]);
                result
            } else {
                // Block has no data written yet, return zeros using SIMD-accelerated fill
                let mut result = vec![0u8; size];
                simd_memset(&mut result, 0);
                result
            };

            info!(
                "Read {} bytes from address 0x{:x} (offset {} in block at 0x{:x})",
                size, address, offset, base_addr
            );

            Ok(data)
        } else {
            Err(MemoryError::InvalidAddress(address))
        }
    }
}
