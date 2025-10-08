/*!
 * FIFO Queue
 * First-in-first-out message queue implementation
 */

use super::super::types::{IpcError, IpcResult, QueueId};
use super::types::QueueMessage;
use crate::core::sync::WaitQueue;
use crate::core::types::Pid;
use std::collections::VecDeque;
use std::sync::Arc;

/// FIFO queue implementation
///
/// # Performance
///
/// Uses centralized WaitQueue (futex on Linux) for optimal blocking operations
pub(super) struct FifoQueue {
    pub id: QueueId,
    pub owner: Pid,
    pub capacity: usize,
    pub messages: VecDeque<QueueMessage>,
    pub wait_queue: Arc<WaitQueue<QueueId>>,
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

        self.messages.push_back(message);
        // Wake one waiter using centralized futex-based wake (Linux) or condvar (other platforms)
        self.wait_queue.wake_one(self.id);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<QueueMessage> {
        self.messages.pop_front()
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
