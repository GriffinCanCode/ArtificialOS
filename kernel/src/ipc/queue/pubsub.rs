/*!
 * PubSub Queue
 * Publish-subscribe queue implementation (broadcast to all subscribers)
 */

use super::super::types::{IpcError, IpcResult, QueueId};
use super::types::QueueMessage;
use crate::core::types::Pid;
use log::debug;
use ahash::HashMap;
use flume;

/// PubSub queue implementation
pub(super) struct PubSubQueue {
    pub id: QueueId,
    pub owner: Pid,
    pub capacity: usize,
    pub subscribers: HashMap<Pid, flume::Sender<QueueMessage>>,
    pub closed: bool,
}

impl PubSubQueue {
    pub fn new(id: QueueId, owner: Pid, capacity: usize) -> Self {
        use super::types::MAX_QUEUE_CAPACITY;
        Self {
            id,
            owner,
            capacity: capacity.min(MAX_QUEUE_CAPACITY),
            subscribers: HashMap::default(),
            closed: false,
        }
    }

    pub fn subscribe(&mut self, pid: Pid) -> flume::Receiver<QueueMessage> {
        let (tx, rx) = flume::unbounded();
        self.subscribers.insert(pid, tx);
        debug!("PID {} subscribed to queue {}", pid, self.id);
        rx
    }

    pub fn unsubscribe(&mut self, pid: Pid) {
        self.subscribers.remove(&pid);
        debug!("PID {} unsubscribed from queue {}", pid, self.id);
    }

    pub fn publish(&mut self, message: QueueMessage) -> IpcResult<usize> {
        if self.closed {
            return Err(IpcError::Closed("Queue closed".into()));
        }

        let mut sent = 0;
        let mut to_remove = Vec::new();

        for (pid, tx) in &self.subscribers {
            match tx.send(message.clone()) {
                Ok(_) => sent += 1,
                Err(_) => {
                    debug!("Subscriber {} disconnected from queue {}", pid, self.id);
                    to_remove.push(*pid);
                }
            }
        }

        // Clean up disconnected subscribers
        for pid in to_remove {
            self.subscribers.remove(&pid);
        }

        Ok(sent)
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    pub fn close(&mut self) {
        self.closed = true;
        self.subscribers.clear();
    }
}
