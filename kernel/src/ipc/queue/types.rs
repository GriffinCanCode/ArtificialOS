/*!
 * Queue Types
 * Common types and constants for message queues
 */

use super::super::types::{QueueId, QueueType};
use crate::core::serde::{is_false, is_zero_usize, system_time_micros};
use crate::core::types::{Pid, Size};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::time::SystemTime;

// Queue limits
pub const MAX_QUEUE_CAPACITY: usize = 10_000;
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB
pub const MAX_QUEUES_PER_PROCESS: usize = 100;
pub const GLOBAL_QUEUE_MEMORY_LIMIT: usize = 100 * 1024 * 1024; // 100MB

/// Queue message with metadata (data stored in MemoryManager)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QueueMessage {
    pub id: u64,
    pub from: Pid,
    /// Memory address where data is stored (via MemoryManager)
    #[serde(skip)]
    pub data_address: usize,
    /// Length of data stored at data_address
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub data_length: usize,
    #[serde(skip_serializing_if = "crate::core::serde::is_default")]
    pub priority: u8,
    #[serde(with = "system_time_micros")]
    pub timestamp: SystemTime,
}

impl QueueMessage {
    pub fn new(id: u64, from: Pid, data_address: usize, data_length: usize, priority: u8) -> Self {
        Self {
            id,
            from,
            data_address,
            data_length,
            priority,
            timestamp: SystemTime::now(),
        }
    }

    pub fn size(&self) -> usize {
        // Metadata size only - actual data is in MemoryManager
        std::mem::size_of::<Self>()
    }

    pub fn data_size(&self) -> usize {
        self.data_length
    }

    /// Read the actual data from MemoryManager
    pub fn read_data(
        &self,
        memory_manager: &crate::memory::MemoryManager,
    ) -> Result<Vec<u8>, String> {
        if self.data_length == 0 {
            return Ok(Vec::new());
        }
        memory_manager
            .read_bytes(self.data_address, self.data_length)
            .map_err(|e| format!("Failed to read message data: {}", e))
    }

    /// Serialize metadata using bincode for internal queue operations
    ///
    /// This provides much better performance than JSON for queue metadata.
    /// Note: data_address is skipped in JSON serialization but included in bincode
    /// since it's needed for internal operations.
    pub fn to_bincode_bytes(&self) -> Result<Vec<u8>, String> {
        crate::core::bincode::to_vec(self)
            .map_err(|e| format!("Failed to serialize queue message with bincode: {}", e))
    }

    /// Deserialize metadata from bincode format
    pub fn from_bincode_bytes(bytes: &[u8]) -> Result<Self, String> {
        crate::core::bincode::from_slice(bytes)
            .map_err(|e| format!("Failed to deserialize queue message with bincode: {}", e))
    }
}

// Implement BincodeSerializable trait for QueueMessage
impl crate::core::traits::BincodeSerializable for QueueMessage {}

// Priority wrapper for heap ordering
#[derive(Debug)]
pub(super) struct PriorityMessage {
    pub message: QueueMessage,
}

impl PartialEq for PriorityMessage {
    fn eq(&self, other: &Self) -> bool {
        self.message.priority == other.message.priority
    }
}

impl Eq for PriorityMessage {}

impl PartialOrd for PriorityMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first (max heap)
        self.message
            .priority
            .cmp(&other.message.priority)
            .then_with(|| other.message.id.cmp(&self.message.id))
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QueueStats {
    pub id: QueueId,
    pub queue_type: QueueType,
    pub owner_pid: Pid,
    pub capacity: Size,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub length: Size,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub subscriber_count: Size,
    #[serde(skip_serializing_if = "is_false")]
    pub closed: bool,
}

// Implement BincodeSerializable for QueueStats
impl crate::core::traits::BincodeSerializable for QueueStats {}
