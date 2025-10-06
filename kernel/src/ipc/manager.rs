/*!
 * Inter-Process Communication (IPC)
 * Unified IPC system: messages, pipes, and shared memory
 */

use crate::ipc::pipe::PipeManager;
use crate::ipc::shm::ShmManager;
use log::{info, warn};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: u32,
    pub to: u32,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

// Queue limits to prevent DoS
const MAX_QUEUE_SIZE: usize = 1000;
const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB
const GLOBAL_IPC_MEMORY_LIMIT: usize = 100 * 1024 * 1024; // 100MB total across all queues

// Global IPC memory tracking to prevent system-wide DoS
static GLOBAL_IPC_MEMORY: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct IPCManager {
    message_queues: Arc<RwLock<HashMap<u32, VecDeque<Message>>>>,
    pipe_manager: PipeManager,
    shm_manager: ShmManager,
}

impl IPCManager {
    pub fn new() -> Self {
        info!(
            "IPC manager initialized with queue limit: {}, global memory limit: {} MB",
            MAX_QUEUE_SIZE,
            GLOBAL_IPC_MEMORY_LIMIT / (1024 * 1024)
        );
        Self {
            message_queues: Arc::new(RwLock::new(HashMap::new())),
            pipe_manager: PipeManager::new(),
            shm_manager: ShmManager::new(),
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

    pub fn send_message(&self, from: u32, to: u32, data: Vec<u8>) -> Result<(), String> {
        // Validate message size
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(format!(
                "Message size {} exceeds limit {}",
                data.len(),
                MAX_MESSAGE_SIZE
            ));
        }

        // Check global IPC memory budget (prevents system-wide DoS)
        let current_global = GLOBAL_IPC_MEMORY.load(Ordering::Acquire);
        let message_size = std::mem::size_of::<Message>() + data.len();

        if current_global + message_size > GLOBAL_IPC_MEMORY_LIMIT {
            warn!(
                "Global IPC memory limit reached: {} bytes used, {} bytes requested",
                current_global, message_size
            );
            return Err(format!(
                "Global IPC memory limit exceeded: {} / {} bytes used",
                current_global, GLOBAL_IPC_MEMORY_LIMIT
            ));
        }

        let mut message_queues = self.message_queues.write();
        let queue = message_queues.entry(to).or_default();

        // Check per-process queue bounds
        if queue.len() >= MAX_QUEUE_SIZE {
            return Err(format!(
                "Queue for PID {} is full ({} messages)",
                to, MAX_QUEUE_SIZE
            ));
        }

        let message = Message {
            from,
            to,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        // Atomically increment global memory counter
        GLOBAL_IPC_MEMORY.fetch_add(message_size, Ordering::Release);

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

    pub fn receive_message(&self, pid: u32) -> Option<Message> {
        let mut message_queues = self.message_queues.write();
        if let Some(queue) = message_queues.get_mut(&pid) {
            if let Some(message) = queue.pop_front() {
                // Atomically decrement global memory counter
                let message_size = std::mem::size_of::<Message>() + message.data.len();
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

    pub fn has_messages(&self, pid: u32) -> bool {
        self.message_queues
            .read()
            .get(&pid)
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    /// Clear all IPC resources for a process (called on process termination)
    pub fn clear_process_queue(&self, pid: u32) -> usize {
        let mut total_cleaned = 0;

        // Clean up message queues
        let mut message_queues = self.message_queues.write();
        if let Some(queue) = message_queues.remove(&pid) {
            // Reclaim global memory for all messages in queue
            let freed_bytes: usize = queue
                .iter()
                .map(|msg| std::mem::size_of::<Message>() + msg.data.len())
                .sum();

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
    pub fn get_global_memory_usage(&self) -> usize {
        GLOBAL_IPC_MEMORY.load(Ordering::Relaxed)
    }
}

impl Default for IPCManager {
    fn default() -> Self {
        Self::new()
    }
}
