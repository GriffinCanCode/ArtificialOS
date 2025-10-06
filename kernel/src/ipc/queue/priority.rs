/*!
 * Priority Queue
 * Priority-based message queue implementation (higher priority first)
 */

use super::super::types::{IpcError, IpcResult, QueueId};
use super::types::{PriorityMessage, QueueMessage};
use crate::core::types::Pid;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::Notify;

/// Priority queue implementation
pub(super) struct PriorityQueue {
    pub id: QueueId,
    pub owner: Pid,
    pub capacity: usize,
    pub messages: BinaryHeap<PriorityMessage>,
    pub notify: Arc<Notify>,
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
            notify: Arc::new(Notify::new()),
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
        self.notify.notify_one();
        Ok(())
    }

    pub fn pop(&mut self) -> Option<QueueMessage> {
        self.messages.pop().map(|pm| pm.message)
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn close(&mut self) {
        self.closed = true;
        self.notify.notify_waiters();
    }
}
