/*!
 * Queue Manager
 * Central manager for all queue types with async support
 */

use super::super::types::{IpcError, IpcResult, QueueId, QueueType};
use super::fifo::FifoQueue;
use super::priority::PriorityQueue;
use super::pubsub::PubSubQueue;
use super::types::{
    QueueMessage, QueueStats, MAX_MESSAGE_SIZE, MAX_QUEUES_PER_PROCESS, MAX_QUEUE_CAPACITY,
};
use crate::core::types::{Pid, Priority, Size};
use crate::memory::MemoryManager;
use dashmap::DashMap;
use ahash::RandomState;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Unified queue wrapper
pub(super) enum Queue {
    Fifo(FifoQueue),
    Priority(PriorityQueue),
    PubSub(PubSubQueue),
}

impl Queue {
    pub fn owner(&self) -> Pid {
        match self {
            Queue::Fifo(q) => q.owner,
            Queue::Priority(q) => q.owner,
            Queue::PubSub(q) => q.owner,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Queue::Fifo(q) => q.len(),
            Queue::Priority(q) => q.len(),
            Queue::PubSub(q) => q.subscriber_count(),
        }
    }

    pub fn close(&mut self) {
        match self {
            Queue::Fifo(q) => q.close(),
            Queue::Priority(q) => q.close(),
            Queue::PubSub(q) => q.close(),
        }
    }
}

/// Async queue manager
pub struct QueueManager {
    queues: Arc<DashMap<QueueId, Queue, RandomState>>,
    next_id: Arc<AtomicU64>,
    next_msg_id: Arc<AtomicU64>,
    process_queues: Arc<DashMap<Pid, HashSet<QueueId>, RandomState>>,
    pubsub_receivers: Arc<DashMap<(QueueId, Pid), mpsc::UnboundedReceiver<QueueMessage>, RandomState>>,
    memory_manager: MemoryManager,
    // Free IDs for recycling (prevents ID exhaustion)
    free_ids: Arc<Mutex<Vec<QueueId>>>,
}

impl QueueManager {
    pub fn new(memory_manager: MemoryManager) -> Self {
        info!(
            "Queue manager initialized with ID recycling (capacity: {})",
            MAX_QUEUE_CAPACITY
        );
        Self {
            queues: Arc::new(DashMap::with_hasher(RandomState::new())),
            next_id: Arc::new(AtomicU64::new(1)),
            next_msg_id: Arc::new(AtomicU64::new(1)),
            process_queues: Arc::new(DashMap::with_hasher(RandomState::new())),
            pubsub_receivers: Arc::new(DashMap::with_hasher(RandomState::new())),
            memory_manager,
            free_ids: Arc::new(Mutex::new(Vec::new())),
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
        let count = self.process_queues.entry(owner_pid).or_default().len();
        if count >= MAX_QUEUES_PER_PROCESS {
            return Err(IpcError::LimitExceeded(format!(
                "Process queue limit exceeded: {}/{}",
                count, MAX_QUEUES_PER_PROCESS
            )));
        }

        // Try to recycle an ID from the free list, otherwise allocate new
        let queue_id = {
            let mut free_ids = self.free_ids.lock().unwrap();
            if let Some(recycled_id) = free_ids.pop() {
                info!("Recycled queue ID {} for PID {}", recycled_id, owner_pid);
                recycled_id
            } else {
                self.next_id.fetch_add(1, Ordering::SeqCst) as u32
            }
        };

        let capacity = capacity.unwrap_or(1000);
        let queue = match queue_type {
            QueueType::Fifo => Queue::Fifo(FifoQueue::new(queue_id, owner_pid, capacity)),
            QueueType::Priority => {
                Queue::Priority(PriorityQueue::new(queue_id, owner_pid, capacity))
            }
            QueueType::PubSub => Queue::PubSub(PubSubQueue::new(queue_id, owner_pid, capacity)),
        };

        self.queues.insert(queue_id, queue);

        // Use alter() for atomic insertion - more efficient than entry() for simple updates
        self.process_queues.alter(&owner_pid, |_, mut queues| {
            queues.insert(queue_id);
            queues
        });

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

        // Allocate memory and write data through MemoryManager (unified storage)
        let data_len = data.len();
        let data_address = self
            .memory_manager
            .allocate(data_len, from_pid)
            .map_err(|e| IpcError::InvalidOperation(format!("Memory allocation failed: {}", e)))?;

        // Write data to allocated memory
        if data_len > 0 {
            self.memory_manager
                .write_bytes(data_address, &data)
                .map_err(|e| {
                    // Clean up allocation on write failure
                    let _ = self.memory_manager.deallocate(data_address);
                    IpcError::InvalidOperation(format!("Memory write failed: {}", e))
                })?;
        }

        let message = QueueMessage::new(
            self.next_msg_id.fetch_add(1, Ordering::SeqCst),
            from_pid,
            data_address,
            data_len,
            priority.unwrap_or(0),
        );

        let mut queue = self
            .queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        match queue.value_mut() {
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
    /// Returns message with data still in MemoryManager - caller must read and deallocate
    pub fn receive(&self, queue_id: QueueId, pid: Pid) -> IpcResult<Option<QueueMessage>> {
        // For PubSub queues, read from subscriber's receiver
        {
            if let Some(queue) = self.queues.get(&queue_id) {
                if matches!(queue.value(), Queue::PubSub(_)) {
                    if let Some(mut rx) = self.pubsub_receivers.get_mut(&(queue_id, pid)) {
                        match rx.try_recv() {
                            Ok(message) => {
                                // Don't deallocate here - caller must read data first
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
        }

        // For FIFO and Priority queues
        let mut queue = self
            .queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        let msg = match queue.value_mut() {
            Queue::Fifo(q) => q.pop(),
            Queue::Priority(q) => q.pop(),
            Queue::PubSub(_) => unreachable!(), // Already handled above
        };

        // Don't deallocate yet - caller must read data first using read_message_data()
        Ok(msg)
    }

    /// Read message data from MemoryManager and deallocate
    pub fn read_message_data(&self, message: &QueueMessage) -> IpcResult<Vec<u8>> {
        // Read data from MemoryManager
        let data = self
            .memory_manager
            .read_bytes(message.data_address, message.data_length)
            .map_err(|e| {
                IpcError::InvalidOperation(format!("Failed to read message data: {}", e))
            })?;

        // Deallocate memory after reading
        if let Err(e) = self.memory_manager.deallocate(message.data_address) {
            warn!(
                "Failed to deallocate message data at 0x{:x}: {}",
                message.data_address, e
            );
        }

        Ok(data)
    }

    /// Subscribe to PubSub queue
    pub fn subscribe(&self, queue_id: QueueId, pid: Pid) -> IpcResult<()> {
        let mut queue = self
            .queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        if let Queue::PubSub(q) = queue.value_mut() {
            let rx = q.subscribe(pid);
            self.pubsub_receivers.insert((queue_id, pid), rx);
            Ok(())
        } else {
            Err(IpcError::InvalidOperation(
                "Subscribe only works for PubSub queues".into(),
            ))
        }
    }

    /// Unsubscribe from PubSub queue
    pub fn unsubscribe(&self, queue_id: QueueId, pid: Pid) -> IpcResult<()> {
        let mut queue = self
            .queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        if let Queue::PubSub(q) = queue.value_mut() {
            q.unsubscribe(pid);
            self.pubsub_receivers.remove(&(queue_id, pid));
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
        if let Some(mut rx) = self.pubsub_receivers.get_mut(&receiver_key) {
            return rx
                .recv()
                .await
                .ok_or_else(|| IpcError::Closed("Subscription closed".into()));
        }

        // For FIFO/Priority queues, poll with notify
        let notify = {
            let queue = self
                .queues
                .get(&queue_id)
                .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

            match queue.value() {
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
            let queue = self
                .queues
                .get(&queue_id)
                .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

            let closed = match queue.value() {
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
        let mut queue = self
            .queues
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
        let mut queue = self
            .queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        // Only owner can destroy
        if queue.owner() != pid {
            return Err(IpcError::PermissionDenied(
                "Only queue owner can destroy".into(),
            ));
        }

        // Drain all messages and deallocate their memory
        let mut freed_count = 0;
        loop {
            let message = match queue.value_mut() {
                Queue::Fifo(q) => q.pop(),
                Queue::Priority(q) => q.pop(),
                Queue::PubSub(_) => break, // PubSub handled via subscribers
            };

            match message {
                Some(msg) => {
                    // Deallocate message data from MemoryManager
                    if let Err(e) = self.memory_manager.deallocate(msg.data_address) {
                        warn!(
                            "Failed to deallocate message {} data at 0x{:x}: {}",
                            msg.id, msg.data_address, e
                        );
                    }
                    freed_count += 1;
                }
                None => break,
            }
        }

        drop(queue);
        self.queues.remove(&queue_id);

        // Add queue ID to free list for recycling
        {
            let mut free_ids = self.free_ids.lock().unwrap();
            free_ids.push(queue_id);
            info!("Added queue ID {} to free list for recycling", queue_id);
        }

        // Use alter() for atomic removal - more efficient than entry().and_modify()
        self.process_queues.alter(&pid, |_, mut qs| {
            qs.remove(&queue_id);
            qs
        });

        // Clean up PubSub receivers
        self.pubsub_receivers.retain(|(qid, _), _| *qid != queue_id);

        info!(
            "PID {} destroyed queue {} (freed {} messages)",
            pid, queue_id, freed_count
        );
        Ok(())
    }

    /// Get queue statistics
    pub fn stats(&self, queue_id: QueueId) -> IpcResult<QueueStats> {
        let queue = self
            .queues
            .get(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        let stats = match queue.value() {
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
        let queue_ids: Vec<QueueId> = self
            .process_queues
            .get(&pid)
            .map(|qs| qs.iter().copied().collect())
            .unwrap_or_default();

        for queue_id in &queue_ids {
            if self.destroy(*queue_id, pid).is_ok() {
                freed += 1;
            }
        }

        // Batch removal of pubsub receivers for this process - more efficient than per-queue retain
        if !queue_ids.is_empty() {
            let before_count = self.pubsub_receivers.len();
            self.pubsub_receivers.retain(|(qid, subscriber_pid), _| {
                !queue_ids.contains(qid) || *subscriber_pid != pid
            });
            let removed = before_count - self.pubsub_receivers.len();
            if removed > 0 {
                info!("Batch removed {} pubsub receivers for PID {}", removed, pid);
            }
        }

        // Shrink maps after cleanup if significant
        if freed > 10 {
            self.queues.shrink_to_fit();
            self.process_queues.shrink_to_fit();
        }

        if freed > 0 {
            info!("Cleaned up {} queues for PID {}", freed, pid);
        }

        freed
    }

    /// Get global memory usage from MemoryManager
    pub fn memory_usage(&self) -> usize {
        let (_, used, _) = self.memory_manager.info();
        used
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
            memory_manager: self.memory_manager.clone(),
            free_ids: Arc::clone(&self.free_ids),
        }
    }
}

// Note: Default trait removed - QueueManager now requires MemoryManager dependency

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryManager;

    #[tokio::test]
    async fn test_fifo_queue() {
        let memory_manager = MemoryManager::new();
        let manager = QueueManager::new(memory_manager);
        let queue_id = manager.create(1, QueueType::Fifo, Some(100)).unwrap();

        manager
            .send(queue_id, 1, b"message1".to_vec(), None)
            .unwrap();
        manager
            .send(queue_id, 1, b"message2".to_vec(), None)
            .unwrap();

        let msg1 = manager.receive(queue_id, 1).unwrap().unwrap();
        let data1 = manager.read_message_data(&msg1).unwrap();
        assert_eq!(data1, b"message1");

        let msg2 = manager.receive(queue_id, 1).unwrap().unwrap();
        let data2 = manager.read_message_data(&msg2).unwrap();
        assert_eq!(data2, b"message2");
    }

    #[tokio::test]
    async fn test_priority_queue() {
        let memory_manager = MemoryManager::new();
        let manager = QueueManager::new(memory_manager);
        let queue_id = manager.create(1, QueueType::Priority, Some(100)).unwrap();

        manager.send(queue_id, 1, b"low".to_vec(), Some(1)).unwrap();
        manager
            .send(queue_id, 1, b"high".to_vec(), Some(10))
            .unwrap();
        manager
            .send(queue_id, 1, b"medium".to_vec(), Some(5))
            .unwrap();

        let msg1 = manager.receive(queue_id, 1).unwrap().unwrap();
        let data1 = manager.read_message_data(&msg1).unwrap();
        assert_eq!(data1, b"high");
        assert_eq!(msg1.priority, 10);

        let msg2 = manager.receive(queue_id, 1).unwrap().unwrap();
        let data2 = manager.read_message_data(&msg2).unwrap();
        assert_eq!(data2, b"medium");
        assert_eq!(msg2.priority, 5);
    }

    #[tokio::test]
    async fn test_pubsub_queue() {
        let memory_manager = MemoryManager::new();
        let manager = QueueManager::new(memory_manager);
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
        let memory_manager = MemoryManager::new();
        let manager = QueueManager::new(memory_manager);
        let queue_id = manager.create(1, QueueType::Fifo, Some(100)).unwrap();

        let manager_clone = manager.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            manager_clone
                .send(queue_id, 1, b"async message".to_vec(), None)
                .unwrap();
        });

        let msg = manager.poll(queue_id, 1).await.unwrap();
        let data = manager.read_message_data(&msg).unwrap();
        assert_eq!(data, b"async message");
    }

    #[test]
    fn test_cleanup() {
        let memory_manager = MemoryManager::new();
        let manager = QueueManager::new(memory_manager);
        let q1 = manager.create(1, QueueType::Fifo, Some(10)).unwrap();
        let q2 = manager.create(1, QueueType::Priority, Some(10)).unwrap();

        let freed = manager.cleanup_process(1);
        assert_eq!(freed, 2);

        assert!(manager.stats(q1).is_err());
        assert!(manager.stats(q2).is_err());
    }
}
