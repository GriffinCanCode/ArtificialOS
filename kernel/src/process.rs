/**
 * Process Management
 * Handles process creation, scheduling, and lifecycle
 */

use log::info;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

use crate::memory::MemoryManager;

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
    memory_manager: Option<MemoryManager>,
}

impl ProcessManager {
    pub fn new() -> Self {
        info!("Process manager initialized");
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            next_pid: Arc::new(RwLock::new(1)),
            memory_manager: None,
        }
    }

    /// Create a new process manager with memory management integration
    pub fn with_memory_manager(memory_manager: MemoryManager) -> Self {
        info!("Process manager initialized with memory management");
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            next_pid: Arc::new(RwLock::new(1)),
            memory_manager: Some(memory_manager),
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
            
            // Clean up memory if memory manager is available
            if let Some(ref mem_mgr) = self.memory_manager {
                let freed = mem_mgr.free_process_memory(pid);
                if freed > 0 {
                    info!("Freed {} bytes from terminated process PID {}", freed, pid);
                }
            }
            
            true
        } else {
            false
        }
    }

    pub fn list_processes(&self) -> Vec<Process> {
        self.processes.read().values().cloned().collect()
    }

    /// Get memory manager reference
    pub fn memory_manager(&self) -> Option<&MemoryManager> {
        self.memory_manager.as_ref()
    }
}

impl Clone for ProcessManager {
    fn clone(&self) -> Self {
        Self {
            processes: Arc::clone(&self.processes),
            next_pid: Arc::clone(&self.next_pid),
            memory_manager: self.memory_manager.clone(),
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

