/*!
 * Inter-Process Communication (IPC)
 * Unified IPC system: messages, pipes, and shared memory
 */

use super::traits::{
    AsyncQueue, IpcCleanup, IpcManager as IpcManagerTrait, MessageQueue, PipeChannel, SharedMemory,
};
use super::types::{IpcError, IpcResult, Message};
use crate::core::types::{Pid, Size};
use crate::ipc::pipe::PipeManager;
use crate::ipc::queue::QueueManager;
use crate::ipc::shm::ShmManager;
use crate::ipc::zerocopy::ZeroCopyIpc;
use crate::memory::MemoryManager;
use dashmap::DashMap;
use ahash::RandomState;
use log::info;
use std::collections::VecDeque;
use std::sync::Arc;

// Queue limits to prevent DoS
const MAX_QUEUE_SIZE: usize = 1000;
const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB

/// IPC Manager
///
/// # Performance
/// - Cache-line aligned for optimal concurrent IPC operations
#[repr(C, align(64))]
#[derive(Clone)]
pub struct IPCManager {
    message_queues: Arc<DashMap<Pid, VecDeque<Message>, RandomState>>,
    pipe_manager: PipeManager,
    shm_manager: ShmManager,
    queue_manager: QueueManager,
    zerocopy_ipc: Option<ZeroCopyIpc>,
    memory_manager: MemoryManager,
}

impl IPCManager {
    pub fn new(memory_manager: MemoryManager) -> Self {
        info!(
            "IPC manager initialized with unified memory management (queue limit: {})",
            MAX_QUEUE_SIZE
        );
        Self {
            message_queues: Arc::new(DashMap::with_hasher(RandomState::new())),
            pipe_manager: PipeManager::new(memory_manager.clone()),
            shm_manager: ShmManager::new(memory_manager.clone()),
            queue_manager: QueueManager::new(memory_manager.clone()),
            zerocopy_ipc: None,
            memory_manager,
        }
    }

    /// Create IPCManager with zero-copy IPC support
    pub fn with_zerocopy(memory_manager: MemoryManager) -> Self {
        info!(
            "IPC manager initialized with zero-copy support (queue limit: {})",
            MAX_QUEUE_SIZE
        );
        Self {
            message_queues: Arc::new(DashMap::with_hasher(RandomState::new())),
            pipe_manager: PipeManager::new(memory_manager.clone()),
            shm_manager: ShmManager::new(memory_manager.clone()),
            queue_manager: QueueManager::new(memory_manager.clone()),
            zerocopy_ipc: Some(ZeroCopyIpc::new(memory_manager.clone())),
            memory_manager,
        }
    }

    /// Get reference to pipe manager
    pub fn pipes(&self) -> &PipeManager {
        &self.pipe_manager
    }

    /// Get reference to shared memory manager
    pub fn shm(&self) -> &ShmManager {
        &self.shm_manager
    }

    /// Get reference to async queue manager
    pub fn queues(&self) -> &QueueManager {
        &self.queue_manager
    }

    /// Get reference to zero-copy IPC manager (if enabled)
    pub fn zerocopy(&self) -> Option<&ZeroCopyIpc> {
        self.zerocopy_ipc.as_ref()
    }

    pub fn send_message(&self, from: Pid, to: Pid, data: Vec<u8>) -> IpcResult<()> {
        // Validate message size
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(IpcError::LimitExceeded(format!(
                "Message size {} exceeds limit {}",
                data.len(),
                MAX_MESSAGE_SIZE
            )));
        }

        let mut queue = self.message_queues.entry(to).or_default();

        // Check per-process queue bounds
        if queue.len() >= MAX_QUEUE_SIZE {
            return Err(IpcError::LimitExceeded(format!(
                "Queue for PID {} is full ({} messages)",
                to, MAX_QUEUE_SIZE
            )));
        }

        let message = Message::new(from, to, data);
        let message_size = message.size();

        // Allocate memory through MemoryManager (integrated tracking)
        let _address = self
            .memory_manager
            .allocate(message_size, from)
            .map_err(|e| IpcError::LimitExceeded(format!("Memory allocation failed: {}", e)))?;

        queue.push_back(message);

        let (_, used, _) = self.memory_manager.info();
        info!(
            "Message sent from PID {} to PID {} ({} in queue, {} bytes used memory)",
            from,
            to,
            queue.len(),
            used
        );
        Ok(())
    }

    pub fn receive_message(&self, pid: Pid) -> Option<Message> {
        if let Some(mut queue) = self.message_queues.get_mut(&pid) {
            if let Some(message) = queue.pop_front() {
                let message_size = message.size();

                // Deallocate memory through MemoryManager (integrated tracking)
                // Note: We can't store the address in Message, so we rely on MemoryManager's
                // process cleanup to reclaim this memory when the process terminates

                let (_, used, _) = self.memory_manager.info();
                info!(
                    "Message received by PID {} ({} bytes used memory)",
                    pid, used
                );
                return Some(message);
            }
        }
        None
    }

    pub fn has_messages(&self, pid: Pid) -> bool {
        self.message_queues
            .get(&pid)
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    /// Clear all IPC resources for a process (called on process termination)
    pub fn clear_process_queue(&self, pid: Pid) -> Size {
        let mut total_cleaned = 0;

        // Clean up message queues
        if let Some((_, queue)) = self.message_queues.remove(&pid) {
            let message_count = queue.len();
            total_cleaned += message_count;

            info!("Cleared {} messages for PID {}", message_count, pid);
        }

        // Clean up pipes
        let pipes_cleaned = self.pipe_manager.cleanup_process(pid);
        total_cleaned += pipes_cleaned;

        // Clean up shared memory
        let shm_cleaned = self.shm_manager.cleanup_process(pid);
        total_cleaned += shm_cleaned;

        // Clean up queues
        let queues_cleaned = self.queue_manager.cleanup_process(pid);
        total_cleaned += queues_cleaned;

        // Clean up zero-copy rings (if enabled)
        let zerocopy_cleaned = if let Some(ref zc) = self.zerocopy_ipc {
            let (count, _bytes) = zc.cleanup_process_rings(pid);
            count
        } else {
            0
        };
        total_cleaned += zerocopy_cleaned;

        if total_cleaned > 0 {
            info!(
                "Total IPC cleanup for PID {}: {} resources ({} messages, {} pipes, {} shm segments, {} queues, {} zerocopy rings)",
                pid, total_cleaned,
                total_cleaned - pipes_cleaned - shm_cleaned - queues_cleaned - zerocopy_cleaned,
                pipes_cleaned, shm_cleaned, queues_cleaned, zerocopy_cleaned
            );
        }

        total_cleaned
    }

    /// Get current global memory usage from MemoryManager
    pub fn get_global_memory_usage(&self) -> Size {
        let (_, used, _) = self.memory_manager.info();
        used
    }
}

// Note: Default trait removed - IPCManager now requires MemoryManager dependency

// Implement MessageQueue trait
impl MessageQueue for IPCManager {
    fn send(&self, from: Pid, to: Pid, data: Vec<u8>) -> IpcResult<()> {
        self.send_message(from, to, data)
    }

    fn receive(&self, pid: Pid) -> IpcResult<Message> {
        self.receive_message(pid)
            .ok_or_else(|| IpcError::WouldBlock("No messages available".to_string()))
    }

    fn try_receive(&self, pid: Pid) -> IpcResult<Option<Message>> {
        Ok(self.receive_message(pid))
    }

    fn has_messages(&self, pid: Pid) -> bool {
        self.has_messages(pid)
    }

    fn clear(&self, pid: Pid) -> Size {
        self.clear_process_queue(pid)
    }
}

// Implement IpcCleanup trait
impl IpcCleanup for IPCManager {
    fn cleanup_process(&self, pid: Pid) -> Size {
        self.clear_process_queue(pid)
    }

    fn global_memory_usage(&self) -> Size {
        self.get_global_memory_usage()
    }
}

// Implement IpcManager trait
impl IpcManagerTrait for IPCManager {
    fn pipes(&self) -> &dyn PipeChannel {
        &self.pipe_manager
    }

    fn shm(&self) -> &dyn SharedMemory {
        &self.shm_manager
    }

    fn queues(&self) -> &dyn AsyncQueue {
        &self.queue_manager
    }
}

// Implement AsyncQueue trait for QueueManager
impl AsyncQueue for QueueManager {
    fn create(
        &self,
        owner_pid: Pid,
        queue_type: super::types::QueueType,
        capacity: Option<Size>,
    ) -> IpcResult<super::types::QueueId> {
        QueueManager::create(self, owner_pid, queue_type, capacity)
    }

    fn send(
        &self,
        queue_id: super::types::QueueId,
        from_pid: Pid,
        data: Vec<u8>,
        priority: Option<u8>,
    ) -> IpcResult<()> {
        QueueManager::send(self, queue_id, from_pid, data, priority)
    }

    fn receive(
        &self,
        queue_id: super::types::QueueId,
        pid: Pid,
    ) -> IpcResult<Option<crate::ipc::queue::QueueMessage>> {
        QueueManager::receive(self, queue_id, pid)
    }

    fn subscribe(&self, queue_id: super::types::QueueId, pid: Pid) -> IpcResult<()> {
        QueueManager::subscribe(self, queue_id, pid)
    }

    fn unsubscribe(&self, queue_id: super::types::QueueId, pid: Pid) -> IpcResult<()> {
        QueueManager::unsubscribe(self, queue_id, pid)
    }

    fn close(&self, queue_id: super::types::QueueId, pid: Pid) -> IpcResult<()> {
        QueueManager::close(self, queue_id, pid)
    }

    fn destroy(&self, queue_id: super::types::QueueId, pid: Pid) -> IpcResult<()> {
        QueueManager::destroy(self, queue_id, pid)
    }

    fn stats(&self, queue_id: super::types::QueueId) -> IpcResult<crate::ipc::queue::QueueStats> {
        QueueManager::stats(self, queue_id)
    }
}
