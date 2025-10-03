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
    processes: HashMap<u32, Process>,
    next_pid: u32,
}

impl ProcessManager {
    pub fn new() -> Self {
        info!("Process manager initialized");
        Self {
            processes: HashMap::new(),
            next_pid: 1,
        }
    }

    pub fn create_process(&mut self, name: String, priority: u8) -> u32 {
        let pid = self.next_pid;
        self.next_pid += 1;

        let process = Process {
            pid,
            name: name.clone(),
            state: ProcessState::Ready,
            priority,
        };

        self.processes.insert(pid, process);
        info!("Created process: {} (PID: {})", name, pid);
        pid
    }

    pub fn get_process(&self, pid: u32) -> Option<&Process> {
        self.processes.get(&pid)
    }

    pub fn terminate_process(&mut self, pid: u32) -> bool {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.state = ProcessState::Terminated;
            info!("Terminated process: PID {}", pid);
            true
        } else {
            false
        }
    }

    pub fn list_processes(&self) -> Vec<&Process> {
        self.processes.values().collect()
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

