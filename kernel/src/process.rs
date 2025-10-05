/*!
 * Process Management
 * Handles process creation, scheduling, and lifecycle
 */

use log::info;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::executor::{ExecutionConfig, ProcessExecutor};
use crate::ipc::IPCManager;
use crate::limits::{LimitManager, Limits};
use crate::memory::MemoryManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub state: ProcessState,
    pub priority: u8,
    #[serde(skip)]
    pub os_pid: Option<u32>,
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
    executor: Option<ProcessExecutor>,
    limit_manager: Option<LimitManager>,
    ipc_manager: Option<IPCManager>,
    // Track child processes per parent PID for limit enforcement
    child_counts: Arc<RwLock<HashMap<u32, u32>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        info!("Process manager initialized");
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            next_pid: Arc::new(RwLock::new(1)),
            memory_manager: None,
            executor: None,
            limit_manager: None,
            ipc_manager: None,
            child_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create process manager with full features
    pub fn with_memory_manager(memory_manager: MemoryManager) -> Self {
        info!("Process manager initialized with memory management");
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            next_pid: Arc::new(RwLock::new(1)),
            memory_manager: Some(memory_manager),
            executor: None,
            limit_manager: None,
            ipc_manager: None,
            child_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create process manager with OS execution capabilities
    pub fn with_executor() -> Self {
        info!("Process manager initialized with OS execution");
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            next_pid: Arc::new(RwLock::new(1)),
            memory_manager: None,
            executor: Some(ProcessExecutor::new()),
            limit_manager: LimitManager::new().ok(),
            ipc_manager: None,
            child_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create process manager with full capabilities
    pub fn full(memory_manager: MemoryManager) -> Self {
        info!("Process manager initialized with full capabilities");
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            next_pid: Arc::new(RwLock::new(1)),
            memory_manager: Some(memory_manager),
            executor: Some(ProcessExecutor::new()),
            limit_manager: LimitManager::new().ok(),
            ipc_manager: None,
            child_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set IPC manager for automatic cleanup on process termination
    pub fn with_ipc_manager(mut self, ipc_manager: IPCManager) -> Self {
        info!("Process manager configured with IPC cleanup");
        self.ipc_manager = Some(ipc_manager);
        self
    }

    /// Create a process (metadata only, no OS process)
    pub fn create_process(&self, name: String, priority: u8) -> u32 {
        self.create_process_with_command(name, priority, None)
    }

    /// Create a process with optional OS execution
    pub fn create_process_with_command(
        &self,
        name: String,
        priority: u8,
        config: Option<ExecutionConfig>,
    ) -> u32 {
        // Allocate PID
        let processes = self.processes.write();
        let mut next_pid = self.next_pid.write();

        let pid = *next_pid;
        *next_pid += 1;

        // Drop locks early to avoid deadlock when reacquiring later
        drop(processes);
        drop(next_pid);

        // Spawn OS process if command provided and executor available
        let os_pid = if let Some(cfg) = config {
            if let Some(ref executor) = self.executor {
                match executor.spawn(pid, name.clone(), cfg) {
                    Ok(os_pid) => {
                        info!("Spawned OS process {} for PID {}", os_pid, pid);

                        // Apply resource limits based on priority
                        if let Some(ref limit_mgr) = self.limit_manager {
                            let limits = self.priority_to_limits(priority);
                            if let Err(e) = limit_mgr.apply(os_pid, &limits) {
                                log::warn!("Failed to apply limits: {}", e);
                            }
                        }

                        Some(os_pid)
                    }
                    Err(e) => {
                        log::error!("Failed to spawn OS process: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // Reacquire locks if needed
        let mut processes = self.processes.write();

        let process = Process {
            pid,
            name: name.clone(),
            state: ProcessState::Ready,
            priority,
            os_pid,
        };

        processes.insert(pid, process);
        info!("Created process: {} (PID: {}, OS PID: {:?})", name, pid, os_pid);
        pid
    }

    /// Convert priority to resource limits
    fn priority_to_limits(&self, priority: u8) -> Limits {
        match priority {
            0..=3 => Limits::new()
                .with_memory(128 * 1024 * 1024) // 128 MB
                .with_cpu_shares(50)
                .with_max_pids(5),
            4..=7 => Limits::default(), // 512 MB, 100 shares
            _ => Limits::new()
                .with_memory(2 * 1024 * 1024 * 1024) // 2 GB
                .with_cpu_shares(200)
                .with_max_pids(50),
        }
    }

    pub fn get_process(&self, pid: u32) -> Option<Process> {
        self.processes.read().get(&pid).cloned()
    }

    pub fn terminate_process(&self, pid: u32) -> bool {
        let mut processes = self.processes.write();
        if let Some(process) = processes.remove(&pid) {
            info!("Terminating process: PID {}", pid);

            // Kill OS process if it exists
            if let Some(os_pid) = process.os_pid {
                if let Some(ref executor) = self.executor {
                    drop(processes); // Release lock before potentially blocking operation

                    if let Err(e) = executor.kill(pid) {
                        log::warn!("Failed to kill OS process: {}", e);
                    }

                    // Remove resource limits
                    if let Some(ref limit_mgr) = self.limit_manager {
                        if let Err(e) = limit_mgr.remove(os_pid) {
                            log::warn!("Failed to remove limits: {}", e);
                        }
                    }

                    // Reacquire lock
                    let _ = self.processes.write();
                }
            }

            // Clean up memory if memory manager is available
            if let Some(ref mem_mgr) = self.memory_manager {
                let freed = mem_mgr.free_process_memory(pid);
                if freed > 0 {
                    info!("Freed {} bytes from terminated PID {}", freed, pid);
                }
            }

            // Clean up IPC resources if IPC manager is available
            if let Some(ref ipc_mgr) = self.ipc_manager {
                let cleaned = ipc_mgr.clear_process_queue(pid);
                if cleaned > 0 {
                    info!("Cleaned up {} IPC resources for terminated PID {}", cleaned, pid);
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

    /// Get executor reference
    pub fn executor(&self) -> Option<&ProcessExecutor> {
        self.executor.as_ref()
    }

    /// Check if process has OS execution
    pub fn has_os_process(&self, pid: u32) -> bool {
        self.processes.read().get(&pid).and_then(|p| p.os_pid).is_some()
    }

    /// Get child process count for a PID
    pub fn get_child_count(&self, pid: u32) -> u32 {
        *self.child_counts.read().get(&pid).unwrap_or(&0)
    }

    /// Increment child count for a PID
    fn increment_child_count(&self, pid: u32) {
        let mut counts = self.child_counts.write();
        *counts.entry(pid).or_insert(0) += 1;
    }

    /// Decrement child count for a PID
    fn decrement_child_count(&self, pid: u32) {
        let mut counts = self.child_counts.write();
        if let Some(count) = counts.get_mut(&pid) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                counts.remove(&pid);
            }
        }
    }
}

impl Clone for ProcessManager {
    fn clone(&self) -> Self {
        Self {
            processes: Arc::clone(&self.processes),
            next_pid: Arc::clone(&self.next_pid),
            memory_manager: self.memory_manager.clone(),
            executor: self.executor.clone(),
            limit_manager: None, // Limit manager is not Clone, create new if needed
            ipc_manager: self.ipc_manager.clone(),
            child_counts: Arc::clone(&self.child_counts),
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
