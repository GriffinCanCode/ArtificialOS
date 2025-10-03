/**
 * Memory Management
 * Handles memory allocation and deallocation
 */

use log::info;
use std::collections::HashMap;

pub struct MemoryBlock {
    pub address: usize,
    pub size: usize,
    pub allocated: bool,
    pub owner_pid: Option<u32>,
}

pub struct MemoryManager {
    blocks: HashMap<usize, MemoryBlock>,
    next_address: usize,
    total_memory: usize,
    used_memory: usize,
}

impl MemoryManager {
    pub fn new() -> Self {
        let total = 1024 * 1024 * 1024; // 1GB simulated memory
        info!("Memory manager initialized with {} bytes", total);
        Self {
            blocks: HashMap::new(),
            next_address: 0,
            total_memory: total,
            used_memory: 0,
        }
    }

    pub fn allocate(&mut self, size: usize, pid: u32) -> Option<usize> {
        if self.used_memory + size > self.total_memory {
            return None; // Out of memory
        }

        let address = self.next_address;
        self.next_address += size;
        self.used_memory += size;

        let block = MemoryBlock {
            address,
            size,
            allocated: true,
            owner_pid: Some(pid),
        };

        self.blocks.insert(address, block);
        info!("Allocated {} bytes at 0x{:x} for PID {}", size, address, pid);
        Some(address)
    }

    pub fn deallocate(&mut self, address: usize) -> bool {
        if let Some(block) = self.blocks.get_mut(&address) {
            if block.allocated {
                block.allocated = false;
                self.used_memory -= block.size;
                info!("Deallocated {} bytes at 0x{:x}", block.size, address);
                return true;
            }
        }
        false
    }

    pub fn get_memory_info(&self) -> (usize, usize, usize) {
        (self.total_memory, self.used_memory, self.total_memory - self.used_memory)
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

