/*!
 * Queue Lifecycle Operations
 * Create, close, destroy, and cleanup operations
 */

use super::super::types::{IpcError, IpcResult, QueueId, QueueType};
use super::fifo::FifoQueue;
use super::manager::{Queue, QueueManager};
use super::priority::PriorityQueue;
use super::pubsub::PubSubQueue;
use super::types::{QueueStats, MAX_QUEUES_PER_PROCESS};
use crate::core::types::{Pid, Size};
use log::{info, warn};
use std::sync::atomic::Ordering;

impl QueueManager {
    /// Create a new queue
    pub fn create(
        &self,
        owner_pid: Pid,
        queue_type: QueueType,
        capacity: Option<Size>,
    ) -> IpcResult<QueueId> {
        self.check_process_queue_limit(owner_pid)?;
        let queue_id = self.allocate_queue_id(owner_pid);
        let capacity = capacity.unwrap_or(1000);

        let queue = self.create_queue_instance(queue_id, owner_pid, queue_type, capacity);
        self.register_queue(queue_id, queue, owner_pid);

        info!(
            "PID {} created {:?} queue {} (capacity: {})",
            owner_pid, queue_type, queue_id, capacity
        );
        Ok(queue_id)
    }

    /// Check if process has reached queue limit
    fn check_process_queue_limit(&self, owner_pid: Pid) -> IpcResult<()> {
        let count = self.process_queues.entry(owner_pid).or_default().len();
        if count >= MAX_QUEUES_PER_PROCESS {
            return Err(IpcError::LimitExceeded(format!(
                "Process queue limit exceeded: {}/{}",
                count, MAX_QUEUES_PER_PROCESS
            )));
        }
        Ok(())
    }

    /// Allocate a queue ID (recycle or create new, lock-free)
    fn allocate_queue_id(&self, owner_pid: Pid) -> QueueId {
        if let Some(recycled_id) = self.free_ids.pop() {
            info!(
                "Recycled queue ID {} for PID {} (lock-free)",
                recycled_id, owner_pid
            );
            recycled_id
        } else {
            self.next_id.fetch_add(1, Ordering::SeqCst) as u32
        }
    }

    /// Create queue instance based on type
    fn create_queue_instance(
        &self,
        queue_id: QueueId,
        owner_pid: Pid,
        queue_type: QueueType,
        capacity: usize,
    ) -> Queue {
        match queue_type {
            QueueType::Fifo => Queue::Fifo(FifoQueue::new(queue_id, owner_pid, capacity)),
            QueueType::Priority => {
                Queue::Priority(PriorityQueue::new(queue_id, owner_pid, capacity))
            }
            QueueType::PubSub => Queue::PubSub(PubSubQueue::new(queue_id, owner_pid, capacity)),
        }
    }

    /// Register queue in manager
    fn register_queue(&self, queue_id: QueueId, queue: Queue, owner_pid: Pid) {
        self.queues.insert(queue_id, queue);
        self.process_queues.alter(&owner_pid, |_, mut queues| {
            queues.insert(queue_id);
            queues
        });
    }

    /// Close queue
    pub fn close(&self, queue_id: QueueId, pid: Pid) -> IpcResult<()> {
        let mut queue = self
            .queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

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
        self.verify_queue_ownership(queue_id, pid)?;
        let freed_count = self.drain_queue_messages(queue_id)?;
        self.remove_queue(queue_id, pid);

        info!(
            "PID {} destroyed queue {} (freed {} messages)",
            pid, queue_id, freed_count
        );
        Ok(())
    }

    /// Verify queue ownership
    fn verify_queue_ownership(&self, queue_id: QueueId, pid: Pid) -> IpcResult<()> {
        let queue = self
            .queues
            .get(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        if queue.owner() != pid {
            return Err(IpcError::PermissionDenied(
                "Only queue owner can destroy".into(),
            ));
        }
        Ok(())
    }

    /// Drain all messages from queue and deallocate
    fn drain_queue_messages(&self, queue_id: QueueId) -> IpcResult<usize> {
        let mut queue = self.queues.get_mut(&queue_id).ok_or_else(|| {
            IpcError::NotFound(format!("Queue {} not found for draining", queue_id))
        })?;
        let mut freed_count = 0;

        loop {
            let message = match queue.value_mut() {
                Queue::Fifo(q) => q.pop(),
                Queue::Priority(q) => q.pop(),
                Queue::PubSub(_) => break,
            };

            match message {
                Some(msg) => {
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

        Ok(freed_count)
    }

    /// Remove queue from manager
    fn remove_queue(&self, queue_id: QueueId, pid: Pid) {
        self.queues.remove(&queue_id);

        // Recycle queue ID (lock-free)
        self.free_ids.push(queue_id);
        info!(
            "Added queue ID {} to lock-free free list for recycling",
            queue_id
        );

        // Remove from process queues
        self.process_queues.alter(&pid, |_, mut qs| {
            qs.remove(&queue_id);
            qs
        });

        // Clean up PubSub receivers
        self.pubsub_receivers.retain(|(qid, _), _| *qid != queue_id);
    }

    /// Get queue statistics
    pub fn stats(&self, queue_id: QueueId) -> IpcResult<QueueStats> {
        let queue = self
            .queues
            .get(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        let stats = match queue.value() {
            Queue::Fifo(q) => self.create_fifo_stats(q),
            Queue::Priority(q) => self.create_priority_stats(q),
            Queue::PubSub(q) => self.create_pubsub_stats(q),
        };

        Ok(stats)
    }

    /// Create FIFO queue stats
    fn create_fifo_stats(&self, q: &FifoQueue) -> QueueStats {
        QueueStats {
            id: q.id,
            queue_type: QueueType::Fifo,
            owner_pid: q.owner,
            capacity: q.capacity,
            length: q.len(),
            subscriber_count: 0,
            closed: q.closed,
        }
    }

    /// Create Priority queue stats
    fn create_priority_stats(&self, q: &PriorityQueue) -> QueueStats {
        QueueStats {
            id: q.id,
            queue_type: QueueType::Priority,
            owner_pid: q.owner,
            capacity: q.capacity,
            length: q.len(),
            subscriber_count: 0,
            closed: q.closed,
        }
    }

    /// Create PubSub queue stats
    fn create_pubsub_stats(&self, q: &PubSubQueue) -> QueueStats {
        QueueStats {
            id: q.id,
            queue_type: QueueType::PubSub,
            owner_pid: q.owner,
            capacity: q.capacity,
            length: 0,
            subscriber_count: q.subscriber_count(),
            closed: q.closed,
        }
    }

    /// Clean up process queues
    pub fn cleanup_process(&self, pid: Pid) -> Size {
        let queue_ids = self.get_process_queue_ids(pid);
        let freed = self.destroy_process_queues(&queue_ids, pid);
        self.cleanup_process_receivers(&queue_ids, pid);
        self.shrink_maps_if_needed(freed);

        if freed > 0 {
            info!("Cleaned up {} queues for PID {}", freed, pid);
        }
        freed
    }

    /// Get all queue IDs for a process
    fn get_process_queue_ids(&self, pid: Pid) -> Vec<QueueId> {
        self.process_queues
            .get(&pid)
            .map(|qs| qs.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Destroy all queues for a process
    fn destroy_process_queues(&self, queue_ids: &[QueueId], pid: Pid) -> Size {
        let mut freed = 0;
        for queue_id in queue_ids {
            if self.destroy(*queue_id, pid).is_ok() {
                freed += 1;
            }
        }
        freed
    }

    /// Clean up PubSub receivers for process
    fn cleanup_process_receivers(&self, queue_ids: &[QueueId], pid: Pid) {
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
    }

    /// Shrink maps if significant cleanup occurred
    fn shrink_maps_if_needed(&self, freed: Size) {
        if freed > 10 {
            self.queues.shrink_to_fit();
            self.process_queues.shrink_to_fit();
        }
    }

    /// Get global memory usage from MemoryManager
    pub fn memory_usage(&self) -> usize {
        let (_, used, _) = self.memory_manager.info();
        used
    }
}
