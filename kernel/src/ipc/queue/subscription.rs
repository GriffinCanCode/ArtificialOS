/*!
 * Queue Subscription Operations
 * PubSub subscribe/unsubscribe and async polling
 */

use super::manager::{Queue, QueueManager};
use super::types::QueueMessage;
use super::super::types::{IpcError, IpcResult, QueueId};
use crate::core::types::Pid;

impl QueueManager {
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
        // Check for PubSub receiver
        if let Some(message) = self.poll_pubsub_receiver(queue_id, pid).await? {
            return Ok(message);
        }

        // Poll FIFO/Priority queues
        self.poll_standard_queue(queue_id, pid).await
    }

    /// Poll PubSub receiver
    async fn poll_pubsub_receiver(
        &self,
        queue_id: QueueId,
        pid: Pid,
    ) -> IpcResult<Option<QueueMessage>> {
        let receiver_key = (queue_id, pid);
        if let Some(rx) = self.pubsub_receivers.get(&receiver_key) {
            let message = rx
                .recv_async()
                .await
                .map_err(|_| IpcError::Closed("Subscription closed".into()))?;
            return Ok(Some(message));
        }
        Ok(None)
    }

    /// Poll standard queue (FIFO/Priority)
    ///
    /// # Performance
    ///
    /// Uses centralized WaitQueue with async adapter for futex-based blocking (Linux)
    async fn poll_standard_queue(&self, queue_id: QueueId, pid: Pid) -> IpcResult<QueueMessage> {
        let wait_queue = self.get_queue_wait_queue(queue_id)?;
        let timeout = std::time::Duration::from_millis(100); // Poll every 100ms

        loop {
            // Fast path: try to receive immediately
            if let Some(msg) = self.receive(queue_id, pid)? {
                return Ok(msg);
            }

            // Check if closed before waiting
            if self.is_queue_closed(queue_id)? {
                return Err(IpcError::Closed("Queue closed".into()));
            }

            // Slow path: async wait on centralized WaitQueue
            // This uses futex on Linux (zero CPU spinning) via spawn_blocking
            let wait_queue_clone = wait_queue.clone();
            let _ = tokio::task::spawn_blocking(move || {
                wait_queue_clone.wait(queue_id, Some(timeout))
            })
            .await
            .map_err(|e| IpcError::InvalidOperation(format!("Wait task failed: {}", e)))?;

            // After wake or timeout, check for message again (loop)
        }
    }

    /// Get WaitQueue handle for queue
    ///
    /// # Performance
    ///
    /// Returns centralized WaitQueue that uses futex on Linux
    fn get_queue_wait_queue(&self, queue_id: QueueId) -> IpcResult<std::sync::Arc<crate::core::sync::WaitQueue<QueueId>>> {
        let queue = self
            .queues
            .get(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        match queue.value() {
            Queue::Fifo(q) => Ok(q.wait_queue.clone()),
            Queue::Priority(q) => Ok(q.wait_queue.clone()),
            Queue::PubSub(_) => Err(IpcError::InvalidOperation(
                "Use subscribe for PubSub queues".into(),
            )),
        }
    }

    /// Check if queue is closed
    fn is_queue_closed(&self, queue_id: QueueId) -> IpcResult<bool> {
        let queue = self
            .queues
            .get(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        let closed = match queue.value() {
            Queue::Fifo(q) => q.closed,
            Queue::Priority(q) => q.closed,
            Queue::PubSub(_) => false,
        };

        Ok(closed)
    }
}
