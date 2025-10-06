/*!
 * Memory Types
 * Common types for memory management
 */

use crate::core::types::{Pid, Address, Size};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Memory operation result
pub type MemoryResult<T> = Result<T, MemoryError>;

/// Memory errors
#[derive(Error, Debug, Clone)]
pub enum MemoryError {
    #[error("Out of memory: requested {requested} bytes, available {available} bytes ({used} used / {total} total)")]
    OutOfMemory {
        requested: usize,
        available: usize,
        used: usize,
        total: usize,
    },

    #[error("Process memory limit exceeded: requested {requested} bytes, limit {limit} bytes, current {current} bytes")]
    ProcessLimitExceeded {
        requested: usize,
        limit: usize,
        current: usize,
    },

    #[error("Invalid memory address: 0x{0:x}")]
    InvalidAddress(usize),

    #[error("Memory corruption detected at 0x{0:x}")]
    CorruptionDetected(usize),

    #[error("Alignment error: address 0x{address:x}, required alignment {alignment}")]
    AlignmentError { address: usize, alignment: usize },

    #[error("Memory protection violation: {0}")]
    ProtectionViolation(String),
}

/// Memory block metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    pub address: Address,
    pub size: Size,
    pub allocated: bool,
    pub owner_pid: Option<Pid>,
}

impl MemoryBlock {
    pub fn new(address: Address, size: Size, owner_pid: Pid) -> Self {
        Self {
            address,
            size,
            allocated: true,
            owner_pid: Some(owner_pid),
        }
    }

    pub fn free(&mut self) {
        self.allocated = false;
    }

    pub fn is_allocated(&self) -> bool {
        self.allocated
    }
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_memory: usize,
    pub used_memory: usize,
    pub available_memory: usize,
    pub usage_percentage: f64,
    pub allocated_blocks: usize,
    pub fragmented_blocks: usize,
}

impl MemoryStats {
    pub fn memory_pressure(&self) -> MemoryPressure {
        if self.usage_percentage >= 95.0 {
            MemoryPressure::Critical
        } else if self.usage_percentage >= 80.0 {
            MemoryPressure::High
        } else if self.usage_percentage >= 60.0 {
            MemoryPressure::Medium
        } else {
            MemoryPressure::Low
        }
    }
}

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryPressure {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for MemoryPressure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MemoryPressure::Low => write!(f, "LOW"),
            MemoryPressure::Medium => write!(f, "MEDIUM"),
            MemoryPressure::High => write!(f, "HIGH"),
            MemoryPressure::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Memory allocation request
#[derive(Debug, Clone)]
pub struct AllocationRequest {
    pub size: Size,
    pub alignment: Size,
    pub pid: Pid,
}

impl AllocationRequest {
    pub fn new(size: Size, pid: Pid) -> Self {
        Self {
            size,
            alignment: std::mem::align_of::<usize>(),
            pid,
        }
    }

    pub fn with_alignment(mut self, alignment: usize) -> Self {
        self.alignment = alignment;
        self
    }
}

/// Process memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMemoryStats {
    pub pid: Pid,
    pub allocated_bytes: Size,
    pub peak_bytes: Size,
    pub allocation_count: usize,
}
