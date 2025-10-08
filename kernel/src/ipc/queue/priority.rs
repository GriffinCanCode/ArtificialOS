/*!
 * Priority Queue
 * Priority-based message queue implementation (higher priority first)
 */

use super::super::types::{IpcError, IpcResult, QueueId};
use super::types::{PriorityMessage, QueueMessage};
use crate::core::sync::WaitQueue;
use crate::core::types::Pid;
use std::collections::BinaryHeap;
use std::sync::Arc;

/// Priority queue implementation
///
/// # Performance
///
/// Uses centralized WaitQueue (futex on Linux) for optimal blocking operations
pub(super) struct PriorityQueue {
    pub id: QueueId,
    pub owner: Pid,
    pub capacity: usize,
    pub messages: BinaryHeap<PriorityMessage>,
    pub wait_queue: Arc<WaitQueue<QueueId>>,
    pub closed: bool,
}

impl PriorityQueue {
    pub fn new(id: QueueId, owner: Pid, capacity: usize) -> Self {
        use super::types::MAX_QUEUE_CAPACITY;
        Self {
            id,
            owner,
            capacity: capacity.min(MAX_QUEUE_CAPACITY),
            messages: BinaryHeap::new(),
            // Use long_wait config for IPC operations (futex on Linux, zero CPU spinning)
            wait_queue: Arc::new(WaitQueue::long_wait()),
            closed: false,
        }
    }

    pub fn push(&mut self, message: QueueMessage) -> IpcResult<()> {
        if self.closed {
            return Err(IpcError::Closed("Queue closed".into()));
        }

        if self.messages.len() >= self.capacity {
            return Err(IpcError::LimitExceeded(format!(
                "Queue full: {}/{}",
                self.messages.len(),
                self.capacity
            )));
        }

        self.messages.push(PriorityMessage { message });
        // Wake one waiter using centralized futex-based wake (Linux) or condvar (other platforms)
        self.wait_queue.wake_one(self.id);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<QueueMessage> {
        self.messages.pop().map(|pm| pm.message)
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn close(&mut self) {
        self.closed = true;
        // Wake all waiters on close
        self.wait_queue.wake_all(self.id);
    }
}
