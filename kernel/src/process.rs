/**
 * Process Management
 * Handles process creation, scheduling, and lifecycle
 */

use log::info;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub state: ProcessState,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessState {
    Ready,
    Running,
    Waiting,
    Terminated,
}

pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<u32, Process>>>,
    next_pid: Arc<RwLock<u32>>,
}

use std::sync::Arc;
use parking_lot::RwLock;

impl ProcessManager {
    pub fn new() -> Self {
        info!("Process manager initialized");
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            next_pid: Arc::new(RwLock::new(1)),
        }
    }

    pub fn create_process(&self, name: String, priority: u8) -> u32 {
        let mut next_pid = self.next_pid.write();
        let pid = *next_pid;
        *next_pid += 1;

        let process = Process {
            pid,
            name: name.clone(),
            state: ProcessState::Ready,
            priority,
        };

        self.processes.write().insert(pid, process);
        info!("Created process: {} (PID: {})", name, pid);
        pid
    }

    pub fn get_process(&self, pid: u32) -> Option<Process> {
        self.processes.read().get(&pid).cloned()
    }

    pub fn terminate_process(&self, pid: u32) -> bool {
        let mut processes = self.processes.write();
        if let Some(process) = processes.get_mut(&pid) {
            process.state = ProcessState::Terminated;
            info!("Terminated process: PID {}", pid);
            true
        } else {
            false
        }
    }

    pub fn list_processes(&self) -> Vec<Process> {
        self.processes.read().values().cloned().collect()
    }
}

impl Clone for ProcessManager {
    fn clone(&self) -> Self {
        Self {
            processes: Arc::clone(&self.processes),
            next_pid: Arc::clone(&self.next_pid),
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

