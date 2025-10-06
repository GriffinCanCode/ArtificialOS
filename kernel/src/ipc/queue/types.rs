/*!
 * Queue Types
 * Common types and constants for message queues
 */

use super::super::types::{IpcError, IpcResult, QueueId, QueueType};
use crate::core::types::{Pid, Size};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::time::SystemTime;

// Queue limits
pub const MAX_QUEUE_CAPACITY: usize = 10_000;
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB
pub const MAX_QUEUES_PER_PROCESS: usize = 100;
pub const GLOBAL_QUEUE_MEMORY_LIMIT: usize = 100 * 1024 * 1024; // 100MB

/// Queue message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMessage {
    pub id: u64,
    pub from: Pid,
    pub data: Vec<u8>,
    pub priority: u8,
    pub timestamp: SystemTime,
    /// Memory address for data allocation (tracked through MemoryManager)
    #[serde(skip)]
    pub data_address: Option<usize>,
}

impl QueueMessage {
    pub fn new(id: u64, from: Pid, data: Vec<u8>, priority: u8) -> Self {
        Self {
            id,
            from,
            data,
            priority,
            timestamp: SystemTime::now(),
            data_address: None,
        }
    }

    pub fn with_address(id: u64, from: Pid, data: Vec<u8>, priority: u8, address: usize) -> Self {
        Self {
            id,
            from,
            data,
            priority,
            timestamp: SystemTime::now(),
            data_address: Some(address),
        }
    }

    pub fn size(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.len()
    }
}

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
pub struct QueueStats {
    pub id: QueueId,
    pub queue_type: QueueType,
    pub owner_pid: Pid,
    pub capacity: Size,
    pub length: Size,
    pub subscriber_count: Size,
    pub closed: bool,
}
