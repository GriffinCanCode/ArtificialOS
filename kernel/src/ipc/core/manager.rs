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
use crate::memory::MemoryManager;
use log::{info, warn};
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Queue limits to prevent DoS
const MAX_QUEUE_SIZE: usize = 1000;
const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB
const GLOBAL_IPC_MEMORY_LIMIT: usize = 100 * 1024 * 1024; // 100MB total across all queues

// Global IPC memory tracking to prevent system-wide DoS
static GLOBAL_IPC_MEMORY: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct IPCManager {
    message_queues: Arc<RwLock<HashMap<Pid, VecDeque<Message>>>>,
    pipe_manager: PipeManager,
    shm_manager: ShmManager,
    queue_manager: QueueManager,
}

impl IPCManager {
    pub fn new(memory_manager: MemoryManager) -> Self {
        info!(
            "IPC manager initialized with unified memory management (queue limit: {}, global memory limit: {} MB)",
            MAX_QUEUE_SIZE,
            GLOBAL_IPC_MEMORY_LIMIT / (1024 * 1024)
        );
        Self {
            message_queues: Arc::new(RwLock::new(HashMap::new())),
            pipe_manager: PipeManager::new(memory_manager.clone()),
            shm_manager: ShmManager::new(memory_manager.clone()),
            queue_manager: QueueManager::new(memory_manager),
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

    pub fn send_message(&self, from: Pid, to: Pid, data: Vec<u8>) -> IpcResult<()> {
        // Validate message size
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(IpcError::LimitExceeded(format!(
                "Message size {} exceeds limit {}",
                data.len(),
                MAX_MESSAGE_SIZE
            )));
        }

        // Check global IPC memory budget (prevents system-wide DoS)
        let current_global = GLOBAL_IPC_MEMORY.load(Ordering::Acquire);
        let message_size = std::mem::size_of::<Message>() + data.len();

        if current_global + message_size > GLOBAL_IPC_MEMORY_LIMIT {
            warn!(
                "Global IPC memory limit reached: {} bytes used, {} bytes requested",
                current_global, message_size
            );
            return Err(IpcError::LimitExceeded(format!(
                "Global IPC memory limit exceeded: {} / {} bytes used",
                current_global, GLOBAL_IPC_MEMORY_LIMIT
            )));
        }

        let mut message_queues = self.message_queues.write();
        let queue = message_queues.entry(to).or_default();

        // Check per-process queue bounds
        if queue.len() >= MAX_QUEUE_SIZE {
            return Err(IpcError::LimitExceeded(format!(
                "Queue for PID {} is full ({} messages)",
                to, MAX_QUEUE_SIZE
            )));
        }

        let message = Message::new(from, to, data);

        // Atomically increment global memory counter
        let message_actual_size = message.size();
        GLOBAL_IPC_MEMORY.fetch_add(message_actual_size, Ordering::Release);

        queue.push_back(message);
        info!(
            "Message sent from PID {} to PID {} ({} in queue, {} bytes global IPC memory)",
            from,
            to,
            queue.len(),
            GLOBAL_IPC_MEMORY.load(Ordering::Relaxed)
        );
        Ok(())
    }

    pub fn receive_message(&self, pid: Pid) -> Option<Message> {
        let mut message_queues = self.message_queues.write();
        if let Some(queue) = message_queues.get_mut(&pid) {
            if let Some(message) = queue.pop_front() {
                // Atomically decrement global memory counter
                let message_size = message.size();
                GLOBAL_IPC_MEMORY.fetch_sub(message_size, Ordering::Release);

                info!(
                    "Message received by PID {} ({} bytes global IPC memory)",
                    pid,
                    GLOBAL_IPC_MEMORY.load(Ordering::Relaxed)
                );
                return Some(message);
            }
        }
        None
    }

    pub fn has_messages(&self, pid: Pid) -> bool {
        self.message_queues
            .read()
            .get(&pid)
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    /// Clear all IPC resources for a process (called on process termination)
    pub fn clear_process_queue(&self, pid: Pid) -> Size {
        let mut total_cleaned = 0;

        // Clean up message queues
        let mut message_queues = self.message_queues.write();
        if let Some(queue) = message_queues.remove(&pid) {
            // Reclaim global memory for all messages in queue
            let freed_bytes: usize = queue.iter().map(|msg| msg.size()).sum();

            GLOBAL_IPC_MEMORY.fetch_sub(freed_bytes, Ordering::Release);

            total_cleaned += queue.len();

            info!(
                "Cleared {} messages for PID {} (freed {} bytes, {} bytes global IPC memory)",
                queue.len(),
                pid,
                freed_bytes,
                GLOBAL_IPC_MEMORY.load(Ordering::Relaxed)
            );
        }

        // Clean up pipes
        let pipes_cleaned = self.pipe_manager.cleanup_process(pid);
        total_cleaned += pipes_cleaned;

        // Clean up shared memory
        let shm_cleaned = self.shm_manager.cleanup_process(pid);
        total_cleaned += shm_cleaned;

        if total_cleaned > 0 {
            info!(
                "Total IPC cleanup for PID {}: {} resources ({} messages, {} pipes, {} shm segments)",
                pid, total_cleaned, total_cleaned - pipes_cleaned - shm_cleaned, pipes_cleaned, shm_cleaned
            );
        }

        total_cleaned
    }

    /// Get current global IPC memory usage
    pub fn get_global_memory_usage(&self) -> Size {
        GLOBAL_IPC_MEMORY.load(Ordering::Relaxed)
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

    fn stats(
        &self,
        queue_id: super::types::QueueId,
    ) -> IpcResult<crate::ipc::queue::QueueStats> {
        QueueManager::stats(self, queue_id)
    }
}
