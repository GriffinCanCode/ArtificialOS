/*!
 * Async Message Queues
 * High-performance async message passing with multiple queue types
 */

use super::types::{IpcError, IpcResult, QueueId, QueueType};
use crate::core::types::{Pid, Priority, Size};
use log::{debug, info, warn};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, Notify};

// Queue limits
const MAX_QUEUE_CAPACITY: usize = 10_000;
const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB
const MAX_QUEUES_PER_PROCESS: usize = 100;
const GLOBAL_QUEUE_MEMORY_LIMIT: usize = 100 * 1024 * 1024; // 100MB

/// Queue message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMessage {
    pub id: u64,
    pub from: Pid,
    pub data: Vec<u8>,
    pub priority: u8,
    pub timestamp: SystemTime,
}

impl QueueMessage {
    pub fn new(id: u64, from: Pid, data: Vec<u8>, priority: u8) -> Self {
        Self {
            id,
            from,
            data,
            priority,
            timestamp: SystemTime::now(),
        }
    }

    pub fn size(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.len()
    }
}

// Priority wrapper for heap ordering
#[derive(Debug)]
struct PriorityMessage {
    message: QueueMessage,
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

/// FIFO queue implementation
struct FifoQueue {
    id: QueueId,
    owner: Pid,
    capacity: usize,
    messages: VecDeque<QueueMessage>,
    notify: Arc<Notify>,
    closed: bool,
}

impl FifoQueue {
    fn new(id: QueueId, owner: Pid, capacity: usize) -> Self {
        Self {
            id,
            owner,
            capacity: capacity.min(MAX_QUEUE_CAPACITY),
            messages: VecDeque::new(),
            notify: Arc::new(Notify::new()),
            closed: false,
        }
    }

    fn push(&mut self, message: QueueMessage) -> IpcResult<()> {
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

    fn pop(&mut self) -> Option<QueueMessage> {
        self.messages.pop_front()
    }

    fn len(&self) -> usize {
        self.messages.len()
    }

    fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    fn close(&mut self) {
        self.closed = true;
        self.notify.notify_waiters();
    }
}

/// Priority queue implementation
struct PriorityQueue {
    id: QueueId,
    owner: Pid,
    capacity: usize,
    messages: BinaryHeap<PriorityMessage>,
    notify: Arc<Notify>,
    closed: bool,
}

impl PriorityQueue {
    fn new(id: QueueId, owner: Pid, capacity: usize) -> Self {
        Self {
            id,
            owner,
            capacity: capacity.min(MAX_QUEUE_CAPACITY),
            messages: BinaryHeap::new(),
            notify: Arc::new(Notify::new()),
            closed: false,
        }
    }

    fn push(&mut self, message: QueueMessage) -> IpcResult<()> {
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

    fn pop(&mut self) -> Option<QueueMessage> {
        self.messages.pop().map(|pm| pm.message)
    }

    fn len(&self) -> usize {
        self.messages.len()
    }

    fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    fn close(&mut self) {
        self.closed = true;
        self.notify.notify_waiters();
    }
}

/// PubSub queue implementation
struct PubSubQueue {
    id: QueueId,
    owner: Pid,
    capacity: usize,
    subscribers: HashMap<Pid, mpsc::UnboundedSender<QueueMessage>>,
    closed: bool,
}

impl PubSubQueue {
    fn new(id: QueueId, owner: Pid, capacity: usize) -> Self {
        Self {
            id,
            owner,
            capacity: capacity.min(MAX_QUEUE_CAPACITY),
            subscribers: HashMap::new(),
            closed: false,
        }
    }

    fn subscribe(&mut self, pid: Pid) -> mpsc::UnboundedReceiver<QueueMessage> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscribers.insert(pid, tx);
        debug!("PID {} subscribed to queue {}", pid, self.id);
        rx
    }

    fn unsubscribe(&mut self, pid: Pid) {
        self.subscribers.remove(&pid);
        debug!("PID {} unsubscribed from queue {}", pid, self.id);
    }

    fn publish(&mut self, message: QueueMessage) -> IpcResult<usize> {
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

    fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    fn close(&mut self) {
        self.closed = true;
        self.subscribers.clear();
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

/// Unified queue wrapper
enum Queue {
    Fifo(FifoQueue),
    Priority(PriorityQueue),
    PubSub(PubSubQueue),
}

impl Queue {
    fn owner(&self) -> Pid {
        match self {
            Queue::Fifo(q) => q.owner,
            Queue::Priority(q) => q.owner,
            Queue::PubSub(q) => q.owner,
        }
    }

    fn len(&self) -> usize {
        match self {
            Queue::Fifo(q) => q.len(),
            Queue::Priority(q) => q.len(),
            Queue::PubSub(q) => q.subscriber_count(),
        }
    }

    fn close(&mut self) {
        match self {
            Queue::Fifo(q) => q.close(),
            Queue::Priority(q) => q.close(),
            Queue::PubSub(q) => q.close(),
        }
    }
}

/// Async queue manager
pub struct QueueManager {
    queues: Arc<RwLock<HashMap<QueueId, Queue>>>,
    next_id: Arc<RwLock<QueueId>>,
    next_msg_id: Arc<RwLock<u64>>,
    process_queues: Arc<RwLock<HashMap<Pid, HashSet<QueueId>>>>,
    pubsub_receivers: Arc<RwLock<HashMap<(QueueId, Pid), mpsc::UnboundedReceiver<QueueMessage>>>>,
    memory_usage: Arc<RwLock<usize>>,
}

impl QueueManager {
    pub fn new() -> Self {
        info!(
            "Queue manager initialized (capacity: {}, memory: {}MB)",
            MAX_QUEUE_CAPACITY,
            GLOBAL_QUEUE_MEMORY_LIMIT / (1024 * 1024)
        );
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            next_msg_id: Arc::new(RwLock::new(1)),
            process_queues: Arc::new(RwLock::new(HashMap::new())),
            pubsub_receivers: Arc::new(RwLock::new(HashMap::new())),
            memory_usage: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new queue
    pub fn create(
        &self,
        owner_pid: Pid,
        queue_type: QueueType,
        capacity: Option<Size>,
    ) -> IpcResult<QueueId> {
        // Check process queue limit
        let mut process_queues = self.process_queues.write();
        let count = process_queues.entry(owner_pid).or_default().len();
        if count >= MAX_QUEUES_PER_PROCESS {
            return Err(IpcError::LimitExceeded(format!(
                "Process queue limit exceeded: {}/{}",
                count, MAX_QUEUES_PER_PROCESS
            )));
        }

        let queue_id = {
            let mut next_id = self.next_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let capacity = capacity.unwrap_or(1000);
        let queue = match queue_type {
            QueueType::Fifo => Queue::Fifo(FifoQueue::new(queue_id, owner_pid, capacity)),
            QueueType::Priority => {
                Queue::Priority(PriorityQueue::new(queue_id, owner_pid, capacity))
            }
            QueueType::PubSub => Queue::PubSub(PubSubQueue::new(queue_id, owner_pid, capacity)),
        };

        self.queues.write().insert(queue_id, queue);
        process_queues.entry(owner_pid).or_default().insert(queue_id);

        info!(
            "PID {} created {:?} queue {} (capacity: {})",
            owner_pid, queue_type, queue_id, capacity
        );
        Ok(queue_id)
    }

    /// Send message to queue
    pub fn send(
        &self,
        queue_id: QueueId,
        from_pid: Pid,
        data: Vec<u8>,
        priority: Option<Priority>,
    ) -> IpcResult<()> {
        // Validate message size
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(IpcError::LimitExceeded(format!(
                "Message size {} exceeds limit {}",
                data.len(),
                MAX_MESSAGE_SIZE
            )));
        }

        // Check global memory
        let message_size = std::mem::size_of::<QueueMessage>() + data.len();
        {
            let mut mem_usage = self.memory_usage.write();
            if *mem_usage + message_size > GLOBAL_QUEUE_MEMORY_LIMIT {
                return Err(IpcError::LimitExceeded(format!(
                    "Global queue memory limit exceeded: {}/{}",
                    *mem_usage, GLOBAL_QUEUE_MEMORY_LIMIT
                )));
            }
            *mem_usage += message_size;
        }

        let message = {
            let mut msg_id = self.next_msg_id.write();
            let id = *msg_id;
            *msg_id += 1;
            QueueMessage::new(id, from_pid, data, priority.unwrap_or(0))
        };

        let mut queues = self.queues.write();
        let queue = queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        match queue {
            Queue::Fifo(q) => q.push(message)?,
            Queue::Priority(q) => q.push(message)?,
            Queue::PubSub(q) => {
                let sent = q.publish(message)?;
                debug!("Published to {} subscribers", sent);
            }
        }

        Ok(())
    }

    /// Receive message from queue (non-blocking)
    pub fn receive(&self, queue_id: QueueId, pid: Pid) -> IpcResult<Option<QueueMessage>> {
        // For PubSub queues, read from subscriber's receiver
        {
            let queues = self.queues.read();
            if let Some(Queue::PubSub(_)) = queues.get(&queue_id) {
                let mut receivers = self.pubsub_receivers.write();
                if let Some(rx) = receivers.get_mut(&(queue_id, pid)) {
                    match rx.try_recv() {
                        Ok(message) => {
                            let mut mem_usage = self.memory_usage.write();
                            *mem_usage = mem_usage.saturating_sub(message.size());
                            return Ok(Some(message));
                        }
                        Err(_) => return Ok(None), // No message available
                    }
                } else {
                    return Err(IpcError::PermissionDenied(
                        "Not subscribed to this PubSub queue".into(),
                    ));
                }
            }
        }

        // For FIFO and Priority queues
        let mut queues = self.queues.write();
        let queue = queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        let msg = match queue {
            Queue::Fifo(q) => q.pop(),
            Queue::Priority(q) => q.pop(),
            Queue::PubSub(_) => unreachable!(), // Already handled above
        };

        if let Some(ref message) = msg {
            let mut mem_usage = self.memory_usage.write();
            *mem_usage = mem_usage.saturating_sub(message.size());
        }

        Ok(msg)
    }

    /// Subscribe to PubSub queue
    pub fn subscribe(&self, queue_id: QueueId, pid: Pid) -> IpcResult<()> {
        let mut queues = self.queues.write();
        let queue = queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        if let Queue::PubSub(q) = queue {
            let rx = q.subscribe(pid);
            self.pubsub_receivers.write().insert((queue_id, pid), rx);
            Ok(())
        } else {
            Err(IpcError::InvalidOperation(
                "Subscribe only works for PubSub queues".into(),
            ))
        }
    }

    /// Unsubscribe from PubSub queue
    pub fn unsubscribe(&self, queue_id: QueueId, pid: Pid) -> IpcResult<()> {
        let mut queues = self.queues.write();
        let queue = queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        if let Queue::PubSub(q) = queue {
            q.unsubscribe(pid);
            self.pubsub_receivers.write().remove(&(queue_id, pid));
            Ok(())
        } else {
            Err(IpcError::InvalidOperation(
                "Unsubscribe only works for PubSub queues".into(),
            ))
        }
    }

    /// Poll for message (async-friendly)
    pub async fn poll(&self, queue_id: QueueId, pid: Pid) -> IpcResult<QueueMessage> {
        // Check if we have a PubSub receiver
        let receiver_key = (queue_id, pid);
        if let Some(rx) = self.pubsub_receivers.write().get_mut(&receiver_key) {
            return rx
                .recv()
                .await
                .ok_or_else(|| IpcError::Closed("Subscription closed".into()));
        }

        // For FIFO/Priority queues, poll with notify
        let notify = {
            let queues = self.queues.read();
            let queue = queues
                .get(&queue_id)
                .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

            match queue {
                Queue::Fifo(q) => q.notify.clone(),
                Queue::Priority(q) => q.notify.clone(),
                Queue::PubSub(_) => {
                    return Err(IpcError::InvalidOperation(
                        "Use subscribe for PubSub queues".into(),
                    ))
                }
            }
        };

        loop {
            // Try to receive
            if let Some(msg) = self.receive(queue_id, pid)? {
                return Ok(msg);
            }

            // Wait for notification
            notify.notified().await;

            // Check if queue was closed
            let queues = self.queues.read();
            let queue = queues
                .get(&queue_id)
                .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

            let closed = match queue {
                Queue::Fifo(q) => q.closed,
                Queue::Priority(q) => q.closed,
                Queue::PubSub(_) => unreachable!(),
            };

            if closed {
                return Err(IpcError::Closed("Queue closed".into()));
            }
        }
    }

    /// Close queue
    pub fn close(&self, queue_id: QueueId, pid: Pid) -> IpcResult<()> {
        let mut queues = self.queues.write();
        let queue = queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        // Only owner can close
        if queue.owner() != pid {
            return Err(IpcError::PermissionDenied(
                "Only queue owner can close".into(),
            ));
        }

        queue.close();
        info!("PID {} closed queue {}", pid, queue_id);
        Ok(())
    }

    /// Destroy queue
    pub fn destroy(&self, queue_id: QueueId, pid: Pid) -> IpcResult<()> {
        let mut queues = self.queues.write();
        let queue = queues
            .get(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        // Only owner can destroy
        if queue.owner() != pid {
            return Err(IpcError::PermissionDenied(
                "Only queue owner can destroy".into(),
            ));
        }

        queues.remove(&queue_id);
        self.process_queues.write().entry(pid).and_modify(|qs| {
            qs.remove(&queue_id);
        });

        info!("PID {} destroyed queue {}", pid, queue_id);
        Ok(())
    }

    /// Get queue statistics
    pub fn stats(&self, queue_id: QueueId) -> IpcResult<QueueStats> {
        let queues = self.queues.read();
        let queue = queues
            .get(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        let stats = match queue {
            Queue::Fifo(q) => QueueStats {
                id: q.id,
                queue_type: QueueType::Fifo,
                owner_pid: q.owner,
                capacity: q.capacity,
                length: q.len(),
                subscriber_count: 0,
                closed: q.closed,
            },
            Queue::Priority(q) => QueueStats {
                id: q.id,
                queue_type: QueueType::Priority,
                owner_pid: q.owner,
                capacity: q.capacity,
                length: q.len(),
                subscriber_count: 0,
                closed: q.closed,
            },
            Queue::PubSub(q) => QueueStats {
                id: q.id,
                queue_type: QueueType::PubSub,
                owner_pid: q.owner,
                capacity: q.capacity,
                length: 0,
                subscriber_count: q.subscriber_count(),
                closed: q.closed,
            },
        };

        Ok(stats)
    }

    /// Clean up process queues
    pub fn cleanup_process(&self, pid: Pid) -> Size {
        let mut freed = 0;
        let queue_ids: Vec<QueueId> = {
            let process_queues = self.process_queues.read();
            process_queues
                .get(&pid)
                .map(|qs| qs.iter().copied().collect())
                .unwrap_or_default()
        };

        for queue_id in queue_ids {
            if self.destroy(queue_id, pid).is_ok() {
                freed += 1;
            }
        }

        if freed > 0 {
            info!("Cleaned up {} queues for PID {}", freed, pid);
        }

        freed
    }

    /// Get global memory usage
    pub fn memory_usage(&self) -> usize {
        *self.memory_usage.read()
    }
}

impl Clone for QueueManager {
    fn clone(&self) -> Self {
        Self {
            queues: Arc::clone(&self.queues),
            next_id: Arc::clone(&self.next_id),
            next_msg_id: Arc::clone(&self.next_msg_id),
            process_queues: Arc::clone(&self.process_queues),
            pubsub_receivers: Arc::clone(&self.pubsub_receivers),
            memory_usage: Arc::clone(&self.memory_usage),
        }
    }
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fifo_queue() {
        let manager = QueueManager::new();
        let queue_id = manager.create(1, QueueType::Fifo, Some(100)).unwrap();

        manager
            .send(queue_id, 1, b"message1".to_vec(), None)
            .unwrap();
        manager
            .send(queue_id, 1, b"message2".to_vec(), None)
            .unwrap();

        let msg1 = manager.receive(queue_id, 1).unwrap().unwrap();
        assert_eq!(msg1.data, b"message1");

        let msg2 = manager.receive(queue_id, 1).unwrap().unwrap();
        assert_eq!(msg2.data, b"message2");
    }

    #[tokio::test]
    async fn test_priority_queue() {
        let manager = QueueManager::new();
        let queue_id = manager.create(1, QueueType::Priority, Some(100)).unwrap();

        manager
            .send(queue_id, 1, b"low".to_vec(), Some(1))
            .unwrap();
        manager
            .send(queue_id, 1, b"high".to_vec(), Some(10))
            .unwrap();
        manager
            .send(queue_id, 1, b"medium".to_vec(), Some(5))
            .unwrap();

        let msg1 = manager.receive(queue_id, 1).unwrap().unwrap();
        assert_eq!(msg1.data, b"high");
        assert_eq!(msg1.priority, 10);

        let msg2 = manager.receive(queue_id, 1).unwrap().unwrap();
        assert_eq!(msg2.data, b"medium");
        assert_eq!(msg2.priority, 5);
    }

    #[tokio::test]
    async fn test_pubsub_queue() {
        let manager = QueueManager::new();
        let queue_id = manager.create(1, QueueType::PubSub, Some(100)).unwrap();

        manager.subscribe(queue_id, 2).unwrap();
        manager.subscribe(queue_id, 3).unwrap();

        manager
            .send(queue_id, 1, b"broadcast".to_vec(), None)
            .unwrap();

        // Both subscribers should receive
        let stats = manager.stats(queue_id).unwrap();
        assert_eq!(stats.subscriber_count, 2);
    }

    #[tokio::test]
    async fn test_async_poll() {
        let manager = QueueManager::new();
        let queue_id = manager.create(1, QueueType::Fifo, Some(100)).unwrap();

        let manager_clone = manager.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            manager_clone
                .send(queue_id, 1, b"async message".to_vec(), None)
                .unwrap();
        });

        let msg = manager.poll(queue_id, 1).await.unwrap();
        assert_eq!(msg.data, b"async message");
    }

    #[test]
    fn test_cleanup() {
        let manager = QueueManager::new();
        let q1 = manager.create(1, QueueType::Fifo, Some(10)).unwrap();
        let q2 = manager.create(1, QueueType::Priority, Some(10)).unwrap();

        let freed = manager.cleanup_process(1);
        assert_eq!(freed, 2);

        assert!(manager.stats(q1).is_err());
        assert!(manager.stats(q2).is_err());
    }
}
