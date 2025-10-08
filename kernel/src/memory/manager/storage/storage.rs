/*!
 * Memory Storage Operations
 * Read/write operations for simulated physical memory
 */

use super::super::core::{MemoryError, MemoryResult};
use super::super::MemoryManager;
use crate::core::types::{Address, Size};
use log::info;

impl MemoryManager {
    /// Write bytes to a memory address
    /// This simulates writing to physical memory for shared memory segments
    pub fn write_bytes(&self, address: Address, data: &[u8]) -> MemoryResult<()> {
        let mut base_addr = None;
        let mut block_size = 0;
        for entry in self.blocks.iter() {
            let addr = *entry.key();
            let block = entry.value();
            if block.allocated && address >= addr && address < addr + block.size {
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
            let offset = address - base_addr;

            let mut entry = self.memory_storage.entry(base_addr).or_insert_with(|| {
                use crate::core::memory::CowMemory;
                CowMemory::new(vec![0u8; block_size])
            });

            entry.write(|buffer| {
                if buffer.len() < block_size {
                    buffer.resize(block_size, 0u8);
                }
                let end = offset + data.len();
                buffer[offset..end].copy_from_slice(data);
            });

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
        let mut base_addr = None;
        for entry in self.blocks.iter() {
            let addr = *entry.key();
            let block = entry.value();
            if block.allocated && address >= addr && address < addr + block.size {
                if address + size <= addr + block.size {
                    base_addr = Some(addr);
                    break;
                } else {
                    return Err(MemoryError::InvalidAddress(address));
                }
            }
        }

        if let Some(base_addr) = base_addr {
            let offset = address - base_addr;

            let data = if let Some(cow_mem) = self.memory_storage.get(&base_addr) {
                cow_mem.read(|buffer| {
                    let end = offset + size;
                    let mut result = vec![0u8; size];
                    result.copy_from_slice(&buffer[offset..end]);
                    result
                })
            } else {
                vec![0u8; size]
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
