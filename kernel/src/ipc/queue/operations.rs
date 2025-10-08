/*!
 * Queue Operations
 * Send, receive, and message data handling operations
 */

use super::super::types::{IpcError, IpcResult, QueueId};
use super::manager::{Queue, QueueManager};
use super::types::{QueueMessage, MAX_MESSAGE_SIZE};
use crate::core::types::{Pid, Priority};
use crate::monitoring::{Category, Event, Payload, Severity};
use log::{debug, warn};
use std::sync::atomic::Ordering;
use std::time::Instant;

impl QueueManager {
    /// Send message to queue
    pub fn send(
        &self,
        queue_id: QueueId,
        from_pid: Pid,
        data: Vec<u8>,
        priority: Option<Priority>,
    ) -> IpcResult<()> {
        self.validate_and_allocate_message(queue_id, from_pid, data, priority)
    }

    /// Validate message and allocate memory
    fn validate_and_allocate_message(
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
            ).into()));
        }

        let data_address = self.allocate_message_memory(from_pid, &data)?;
        let message = self.create_queue_message(from_pid, data_address, data.len(), priority);
        self.enqueue_message(queue_id, message)
    }

    /// Allocate memory for message data
    fn allocate_message_memory(&self, from_pid: Pid, data: &[u8]) -> IpcResult<usize> {
        let data_len = data.len();
        let data_address = self
            .memory_manager
            .allocate(data_len, from_pid)
            .map_err(|e| IpcError::InvalidOperation(format!("Memory allocation failed: {}", e).into()))?;

        if data_len > 0 {
            self.memory_manager
                .write_bytes(data_address, data)
                .map_err(|e| {
                    let _ = self.memory_manager.deallocate(data_address);
                    IpcError::InvalidOperation(format!("Memory write failed: {}", e))
                })?;
        }

        Ok(data_address)
    }

    /// Create a queue message structure
    fn create_queue_message(
        &self,
        from_pid: Pid,
        data_address: usize,
        data_len: usize,
        priority: Option<Priority>,
    ) -> QueueMessage {
        QueueMessage::new(
            self.next_msg_id.fetch_add(1, Ordering::SeqCst),
            from_pid,
            data_address,
            data_len,
            priority.unwrap_or(0),
        )
    }

    /// Enqueue message to appropriate queue type
    fn enqueue_message(&self, queue_id: QueueId, message: QueueMessage) -> IpcResult<()> {
        // Capture data for event emission before message is moved
        let from_pid = message.from;
        let data_length = message.data_length;

        let mut queue = self
            .queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id).into()))?;

        match queue.value_mut() {
            Queue::Fifo(q) => q.push(message)?,
            Queue::Priority(q) => q.push(message)?,
            Queue::PubSub(q) => {
                let sent = q.publish(message)?;
                debug!("Published to {} subscribers", sent);
            }
        }

        // Emit message sent event
        if let Some(ref collector) = self.collector {
            collector.emit(
                Event::new(
                    Severity::Debug,
                    Category::Ipc,
                    Payload::MessageSent {
                        queue_id: queue_id as u64,
                        size: data_length,
                    },
                )
                .with_pid(from_pid),
            );
        }

        Ok(())
    }

    /// Receive message from queue (non-blocking)
    pub fn receive(&self, queue_id: QueueId, pid: Pid) -> IpcResult<Option<QueueMessage>> {
        let start = Instant::now();

        // Check for PubSub receiver
        if let Some(message) = self.try_receive_pubsub(queue_id, pid)? {
            // Emit message received event
            if let Some(ref collector) = self.collector {
                let wait_time_us = start.elapsed().as_micros() as u64;
                collector.emit(
                    Event::new(
                        Severity::Debug,
                        Category::Ipc,
                        Payload::MessageReceived {
                            queue_id: queue_id as u64,
                            size: message.data_length,
                            wait_time_us,
                        },
                    )
                    .with_pid(pid),
                );
            }
            return Ok(Some(message));
        }

        // For FIFO and Priority queues
        let message = self.receive_from_standard_queue(queue_id)?;

        // Emit message received event if message was received
        if let Some(ref msg) = message {
            if let Some(ref collector) = self.collector {
                let wait_time_us = start.elapsed().as_micros() as u64;
                collector.emit(
                    Event::new(
                        Severity::Debug,
                        Category::Ipc,
                        Payload::MessageReceived {
                            queue_id: queue_id as u64,
                            size: msg.data_length,
                            wait_time_us,
                        },
                    )
                    .with_pid(pid),
                );
            }
        }

        Ok(message)
    }

    /// Try to receive from PubSub queue
    fn try_receive_pubsub(&self, queue_id: QueueId, pid: Pid) -> IpcResult<Option<QueueMessage>> {
        if let Some(queue) = self.queues.get(&queue_id) {
            if matches!(queue.value(), Queue::PubSub(_)) {
                if let Some(rx) = self.pubsub_receivers.get_mut(&(queue_id, pid)) {
                    match rx.try_recv() {
                        Ok(message) => return Ok(Some(message).into()),
                        Err(_) => return Ok(None),
                    }
                } else {
                    return Err(IpcError::PermissionDenied(
                        "Not subscribed to this PubSub queue".into(),
                    ));
                }
            }
        }
        Ok(None)
    }

    /// Receive from FIFO or Priority queue
    fn receive_from_standard_queue(&self, queue_id: QueueId) -> IpcResult<Option<QueueMessage>> {
        let mut queue = self
            .queues
            .get_mut(&queue_id)
            .ok_or_else(|| IpcError::NotFound(format!("Queue {} not found", queue_id).into()))?;

        let msg = match queue.value_mut() {
            Queue::Fifo(q) => q.pop(),
            Queue::Priority(q) => q.pop(),
            Queue::PubSub(_) => unreachable!(),
        };

        Ok(msg)
    }

    /// Read message data from MemoryManager and deallocate
    pub fn read_message_data(&self, message: &QueueMessage) -> IpcResult<Vec<u8>> {
        let data = self.read_data_from_memory(message)?;
        self.deallocate_message_memory(message);
        Ok(data)
    }

    /// Read data from memory manager
    fn read_data_from_memory(&self, message: &QueueMessage) -> IpcResult<Vec<u8>> {
        self.memory_manager
            .read_bytes(message.data_address, message.data_length)
            .map_err(|e| IpcError::InvalidOperation(format!("Failed to read message data: {}", e).into()))
    }

    /// Deallocate message memory
    fn deallocate_message_memory(&self, message: &QueueMessage) {
        if let Err(e) = self.memory_manager.deallocate(message.data_address) {
            warn!(
                "Failed to deallocate message data at 0x{:x}: {}",
                message.data_address, e
            );
        }
    }
}
