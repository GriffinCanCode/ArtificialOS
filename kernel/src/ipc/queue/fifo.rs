/*!
 * FIFO Queue
 * First-in-first-out message queue implementation
 */

use super::super::types::{IpcError, IpcResult, QueueId};
use super::types::QueueMessage;
use crate::core::types::Pid;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Notify;

/// FIFO queue implementation
pub(super) struct FifoQueue {
    pub id: QueueId,
    pub owner: Pid,
    pub capacity: usize,
    pub messages: VecDeque<QueueMessage>,
    pub notify: Arc<Notify>,
    pub closed: bool,
}

impl FifoQueue {
    pub fn new(id: QueueId, owner: Pid, capacity: usize) -> Self {
        use super::types::MAX_QUEUE_CAPACITY;
        Self {
            id,
            owner,
            capacity: capacity.min(MAX_QUEUE_CAPACITY),
            messages: VecDeque::new(),
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

        self.messages.push_back(message);
        self.notify.notify_one();
        Ok(())
    }

    pub fn pop(&mut self) -> Option<QueueMessage> {
        self.messages.pop_front()
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
