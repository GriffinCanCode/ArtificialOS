/*!
 * Memory Types
 * Common types for memory management
 */

use crate::core::serde::{is_default, is_none, is_zero_usize};
use crate::core::types::{Address, Pid, Size};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Memory operation result
pub type MemoryResult<T> = Result<T, MemoryError>;

/// Memory errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MemoryError {
    #[error("Out of memory: requested {requested} bytes, available {available} bytes ({used} used / {total} total)")]
    OutOfMemory {
        requested: Size,
        available: Size,
        #[serde(skip_serializing_if = "is_zero_usize")]
        used: Size,
        total: Size,
    },

    #[error("Process memory limit exceeded: requested {requested} bytes, limit {limit} bytes, current {current} bytes")]
    ProcessLimitExceeded {
        requested: Size,
        limit: Size,
        #[serde(skip_serializing_if = "is_zero_usize")]
        current: Size,
    },

    #[error("Invalid memory address: 0x{0:x}")]
    InvalidAddress(Address),

    #[error("Memory corruption detected at 0x{0:x}")]
    CorruptionDetected(Address),

    #[error("Alignment error: address 0x{address:x}, required alignment {alignment}")]
    AlignmentError {
        address: Address,
        alignment: Size,
    },

    #[error("Memory protection violation: {0}")]
    ProtectionViolation(String),
}

/// Memory block metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MemoryBlock {
    pub address: Address,
    pub size: Size,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub allocated: bool,
    #[serde(skip_serializing_if = "is_none")]
    pub owner_pid: Option<Pid>,
}

/// Helper for default true value
fn default_true() -> bool {
    true
}

/// Helper to check if bool is true
fn is_true(value: &bool) -> bool {
    *value
}

impl Default for MemoryBlock {
    fn default() -> Self {
        Self {
            address: 0,
            size: 0,
            allocated: false,
            owner_pid: None,
        }
    }
}

impl MemoryBlock {
    /// Create a new allocated memory block
    pub fn new(address: Address, size: Size, owner_pid: Pid) -> Self {
        Self {
            address,
            size,
            allocated: true,
            owner_pid: Some(owner_pid),
        }
    }

    /// Create an unowned memory block
    pub fn unowned(address: Address, size: Size) -> Self {
        Self {
            address,
            size,
            allocated: true,
            owner_pid: None,
        }
    }

    /// Mark block as deallocated
    pub fn free(&mut self) {
        self.allocated = false;
    }

    /// Check if block is allocated
    pub fn is_allocated(&self) -> bool {
        self.allocated
    }

    /// Get owner PID if any
    pub fn owner(&self) -> Option<Pid> {
        self.owner_pid
    }

    /// Check if block is owned by a specific process
    pub fn is_owned_by(&self, pid: Pid) -> bool {
        self.owner_pid == Some(pid)
    }
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MemoryStats {
    pub total_memory: Size,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub used_memory: Size,
    pub available_memory: Size,
    #[serde(default, skip_serializing_if = "is_default")]
    pub usage_percentage: f64,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub allocated_blocks: usize,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub fragmented_blocks: usize,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_memory: 0,
            used_memory: 0,
            available_memory: 0,
            usage_percentage: 0.0,
            allocated_blocks: 0,
            fragmented_blocks: 0,
        }
    }
}

impl MemoryStats {
    /// Create new memory stats
    pub fn new(total_memory: Size, used_memory: Size) -> Self {
        let available_memory = total_memory.saturating_sub(used_memory);
        let usage_percentage = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_memory,
            used_memory,
            available_memory,
            usage_percentage,
            allocated_blocks: 0,
            fragmented_blocks: 0,
        }
    }

    /// Get memory pressure level
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

    /// Check if memory is low (>=60% usage)
    pub fn is_low_memory(&self) -> bool {
        self.usage_percentage >= 60.0
    }

    /// Check if memory is critical (>=95% usage)
    pub fn is_critical(&self) -> bool {
        self.usage_percentage >= 95.0
    }

    /// Get fragmentation ratio
    pub fn fragmentation_ratio(&self) -> f64 {
        let total_blocks = self.allocated_blocks + self.fragmented_blocks;
        if total_blocks == 0 {
            0.0
        } else {
            self.fragmented_blocks as f64 / total_blocks as f64
        }
    }
}

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AllocationRequest {
    pub size: Size,
    #[serde(default = "default_alignment", skip_serializing_if = "is_default_alignment")]
    pub alignment: Size,
    pub pid: Pid,
}

/// Default alignment (pointer size)
fn default_alignment() -> Size {
    std::mem::align_of::<usize>()
}

/// Check if alignment is default
fn is_default_alignment(value: &Size) -> bool {
    *value == std::mem::align_of::<usize>()
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
#[serde(rename_all = "snake_case")]
pub struct ProcessMemoryStats {
    pub pid: Pid,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub allocated_bytes: Size,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub peak_bytes: Size,
    #[serde(default, skip_serializing_if = "is_zero_usize")]
    pub allocation_count: usize,
}

impl Default for ProcessMemoryStats {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ProcessMemoryStats {
    /// Create new process memory stats
    pub fn new(pid: Pid) -> Self {
        Self {
            pid,
            allocated_bytes: 0,
            peak_bytes: 0,
            allocation_count: 0,
        }
    }

    /// Calculate utilization ratio
    pub fn utilization_ratio(&self) -> f64 {
        if self.peak_bytes == 0 {
            0.0
        } else {
            self.allocated_bytes as f64 / self.peak_bytes as f64
        }
    }

    /// Get average allocation size
    pub fn avg_allocation_size(&self) -> Size {
        if self.allocation_count == 0 {
            0
        } else {
            self.allocated_bytes / self.allocation_count
        }
    }
}
