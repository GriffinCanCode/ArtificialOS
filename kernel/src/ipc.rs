/**
 * Inter-Process Communication (IPC)
 * Handles communication between kernel and AI service
 */

use log::info;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: u32,
    pub to: u32,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

pub struct IPCManager {
    message_queues: HashMap<u32, Vec<Message>>,
}

impl IPCManager {
    pub fn new() -> Self {
        info!("IPC manager initialized");
        Self {
            message_queues: HashMap::new(),
        }
    }

    pub fn send_message(&mut self, from: u32, to: u32, data: Vec<u8>) {
        let message = Message {
            from,
            to,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.message_queues
            .entry(to)
            .or_insert_with(Vec::new)
            .push(message);

        info!("Message sent from PID {} to PID {}", from, to);
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

