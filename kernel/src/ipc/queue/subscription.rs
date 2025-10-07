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
    async fn poll_standard_queue(&self, queue_id: QueueId, pid: Pid) -> IpcResult<QueueMessage> {
        let notify = self.get_queue_notify(queue_id)?;

        loop {
            if let Some(msg) = self.receive(queue_id, pid)? {
                return Ok(msg);
            }

            notify.notified().await;

            if self.is_queue_closed(queue_id)? {
                return Err(IpcError::Closed("Queue closed".into()));
            }
        }
    }

    /// Get notification handle for queue
    fn get_queue_notify(&self, queue_id: QueueId) -> IpcResult<std::sync::Arc<tokio::sync::Notify>> {
        let queue = self
            .queues
            .get(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id)))?;

        match queue.value() {
            Queue::Fifo(q) => Ok(q.notify.clone()),
            Queue::Priority(q) => Ok(q.notify.clone()),
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
