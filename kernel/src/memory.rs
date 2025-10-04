/**
 * Memory Management
 * Handles memory allocation and deallocation with graceful OOM handling
 */

use log::{info, warn, error};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct MemoryBlock {
    pub address: usize,
    pub size: usize,
    pub allocated: bool,
    pub owner_pid: Option<u32>,
}

/// Memory allocation result
#[derive(Debug)]
pub enum MemoryError {
    OutOfMemory {
        requested: usize,
        available: usize,
        used: usize,
        total: usize,
    },
    ExceedsProcessLimit {
        requested: usize,
        process_limit: usize,
        process_current: usize,
    },
    InvalidAddress,
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::OutOfMemory { requested, available, used, total } => {
                write!(
                    f,
                    "Out of memory: requested {} bytes, available {} bytes ({} used / {} total)",
                    requested, available, used, total
                )
            }
            MemoryError::ExceedsProcessLimit { requested, process_limit, process_current } => {
                write!(
                    f,
                    "Process memory limit exceeded: requested {} bytes, limit {} bytes, current {} bytes",
                    requested, process_limit, process_current
                )
            }
            MemoryError::InvalidAddress => write!(f, "Invalid memory address"),
        }
    }
}

impl std::error::Error for MemoryError {}

pub struct MemoryManager {
    blocks: Arc<RwLock<HashMap<usize, MemoryBlock>>>,
    next_address: Arc<RwLock<usize>>,
    total_memory: usize,
    used_memory: Arc<RwLock<usize>>,
    // Memory pressure thresholds (percentage)
    warning_threshold: f64,  // 80%
    critical_threshold: f64, // 95%
}

impl MemoryManager {
    pub fn new() -> Self {
        let total = 1024 * 1024 * 1024; // 1GB simulated memory
        info!("Memory manager initialized with {} bytes", total);
        Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
            next_address: Arc::new(RwLock::new(0)),
            total_memory: total,
            used_memory: Arc::new(RwLock::new(0)),
            warning_threshold: 0.80,
            critical_threshold: 0.95,
        }
    }

    /// Check memory pressure level
    fn check_memory_pressure(&self, used: usize) -> Option<&str> {
        let usage_ratio = used as f64 / self.total_memory as f64;
        
        if usage_ratio >= self.critical_threshold {
            Some("CRITICAL")
        } else if usage_ratio >= self.warning_threshold {
            Some("WARNING")
        } else {
            None
        }
    }

    /// Allocate memory with graceful OOM handling
    pub fn allocate(&self, size: usize, pid: u32) -> Result<usize, MemoryError> {
        let mut used = self.used_memory.write();
        
        // Check if allocation would exceed total memory
        if *used + size > self.total_memory {
            let available = self.total_memory - *used;
            error!(
                "OOM: PID {} requested {} bytes, only {} bytes available ({} used / {} total)",
                pid, size, available, *used, self.total_memory
            );
            
            return Err(MemoryError::OutOfMemory {
                requested: size,
                available,
                used: *used,
                total: self.total_memory,
            });
        }

        // Allocate memory
        let mut next_addr = self.next_address.write();
        let address = *next_addr;
        *next_addr += size;
        *used += size;

        let block = MemoryBlock {
            address,
            size,
            allocated: true,
            owner_pid: Some(pid),
        };

        self.blocks.write().insert(address, block);
        
        // Log allocation with memory pressure warnings
        if let Some(level) = self.check_memory_pressure(*used) {
            warn!(
                "Memory pressure {}: Allocated {} bytes at 0x{:x} for PID {} ({:.1}% used: {} / {})",
                level, size, address, pid,
                (*used as f64 / self.total_memory as f64) * 100.0,
                *used, self.total_memory
            );
        } else {
            info!("Allocated {} bytes at 0x{:x} for PID {}", size, address, pid);
        }

        Ok(address)
    }

    /// Deallocate memory
    pub fn deallocate(&self, address: usize) -> Result<(), MemoryError> {
        let mut blocks = self.blocks.write();
        
        if let Some(block) = blocks.get_mut(&address) {
            if block.allocated {
                let size = block.size;
                block.allocated = false;
                
                let mut used = self.used_memory.write();
                *used = used.saturating_sub(size);
                
                info!("Deallocated {} bytes at 0x{:x} ({} bytes now available)", 
                      size, address, self.total_memory - *used);
                
                return Ok(());
            }
        }
        
        warn!("Attempted to deallocate invalid or already freed address: 0x{:x}", address);
        Err(MemoryError::InvalidAddress)
    }

    /// Free all memory allocated to a specific process (called on process termination)
    pub fn free_process_memory(&self, pid: u32) -> usize {
        let mut blocks = self.blocks.write();
        let mut freed_bytes = 0;
        
        for (_, block) in blocks.iter_mut() {
            if block.allocated && block.owner_pid == Some(pid) {
                block.allocated = false;
                freed_bytes += block.size;
            }
        }
        
        if freed_bytes > 0 {
            let mut used = self.used_memory.write();
            *used = used.saturating_sub(freed_bytes);
            
            info!(
                "Cleaned up {} bytes from terminated PID {} ({} bytes now available)",
                freed_bytes, pid, self.total_memory - *used
            );
        }
        
        freed_bytes
    }

    /// Get memory statistics for a specific process
    pub fn get_process_memory(&self, pid: u32) -> usize {
        let blocks = self.blocks.read();
        blocks.values()
            .filter(|b| b.allocated && b.owner_pid == Some(pid))
            .map(|b| b.size)
            .sum()
    }

    /// Get overall memory info: (total, used, available)
    pub fn get_memory_info(&self) -> (usize, usize, usize) {
        let used = *self.used_memory.read();
        (self.total_memory, used, self.total_memory - used)
    }

    /// Get detailed memory statistics
    pub fn get_detailed_stats(&self) -> MemoryStats {
        let blocks = self.blocks.read();
        let used = *self.used_memory.read();
        
        let allocated_blocks = blocks.values().filter(|b| b.allocated).count();
        let fragmented_blocks = blocks.values().filter(|b| !b.allocated).count();
        
        MemoryStats {
            total_memory: self.total_memory,
            used_memory: used,
            available_memory: self.total_memory - used,
            usage_percentage: (used as f64 / self.total_memory as f64) * 100.0,
            allocated_blocks,
            fragmented_blocks,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_memory: usize,
    pub used_memory: usize,
    pub available_memory: usize,
    pub usage_percentage: f64,
    pub allocated_blocks: usize,
    pub fragmented_blocks: usize,
}

impl Clone for MemoryManager {
    fn clone(&self) -> Self {
        Self {
            blocks: Arc::clone(&self.blocks),
            next_address: Arc::clone(&self.next_address),
            total_memory: self.total_memory,
            used_memory: Arc::clone(&self.used_memory),
            warning_threshold: self.warning_threshold,
            critical_threshold: self.critical_threshold,
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

