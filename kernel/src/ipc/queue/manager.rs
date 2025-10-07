/*!
 * Queue Manager
 * Central manager for all queue types with async support
 */

use super::super::types::QueueId;
use super::fifo::FifoQueue;
use super::priority::PriorityQueue;
use super::pubsub::PubSubQueue;
use super::types::{QueueMessage, MAX_QUEUE_CAPACITY};
use crate::core::types::Pid;
use crate::memory::MemoryManager;
use dashmap::DashMap;
use ahash::RandomState;
use flume;
use log::info;
use std::collections::HashSet;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

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
///
/// # Performance
/// - Cache-line aligned to prevent false sharing of atomic ID generators
#[repr(C, align(64))]
pub struct QueueManager {
    pub(super) queues: Arc<DashMap<QueueId, Queue, RandomState>>,
    pub(super) next_id: Arc<AtomicU64>,
    pub(super) next_msg_id: Arc<AtomicU64>,
    pub(super) process_queues: Arc<DashMap<Pid, HashSet<QueueId>, RandomState>>,
    pub(super) pubsub_receivers: Arc<DashMap<(QueueId, Pid), flume::Receiver<QueueMessage>, RandomState>>,
    pub(super) memory_manager: MemoryManager,
    pub(super) free_ids: Arc<Mutex<Vec<QueueId>>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::types::QueueType;

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
