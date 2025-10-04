/*!
 * Inter-Process Communication (IPC)
 * Handles communication between kernel and AI service
 */

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

pub struct IPCManager {
    message_queues: HashMap<u32, Vec<Message>>,
}

impl IPCManager {
    pub fn new() -> Self {
        info!(
            "IPC manager initialized with queue limit: {}",
            MAX_QUEUE_SIZE
        );
        Self {
            message_queues: HashMap::new(),
        }
    }

    pub fn send_message(&mut self, from: u32, to: u32, data: Vec<u8>) -> Result<(), String> {
        // Validate message size
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(format!(
                "Message size {} exceeds limit {}",
                data.len(),
                MAX_MESSAGE_SIZE
            ));
        }

        let queue = self.message_queues.entry(to).or_default();

        // Check queue bounds
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

        queue.push(message);
        info!(
            "Message sent from PID {} to PID {} ({} in queue)",
            from,
            to,
            queue.len()
        );
        Ok(())
    }

    pub fn receive_message(&mut self, pid: u32) -> Option<Message> {
        if let Some(queue) = self.message_queues.get_mut(&pid) {
            if !queue.is_empty() {
                let message = queue.remove(0);
                info!("Message received by PID {}", pid);
                return Some(message);
            }
        }
        None
    }

    pub fn has_messages(&self, pid: u32) -> bool {
        self.message_queues
            .get(&pid)
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }
}

impl Default for IPCManager {
    fn default() -> Self {
        Self::new()
    }
}
