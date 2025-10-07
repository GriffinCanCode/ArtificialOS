/*!
 * Process Types
 * Common types for process management
 */

use crate::core::serde::{is_false, is_none, is_zero_u64, is_zero_usize};
use crate::core::types::{Pid, Priority};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Process operation result
pub type ProcessResult<T> = Result<T, ProcessError>;

/// Process errors
#[derive(Error, Debug, Clone)]
pub enum ProcessError {
    #[error("Process not found: {0}")]
    ProcessNotFound(Pid),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Spawn failed: {0}")]
    SpawnFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Process limit exceeded: current {current}, limit {limit}")]
    ProcessLimitExceeded { current: u32, limit: u32 },

    #[error("Invalid state transition: {from:?} -> {to:?}")]
    InvalidStateTransition {
        from: ProcessState,
        to: ProcessState,
    },

    #[error("Execution error: {0}")]
    ExecutionError(String),
}

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    /// Process is ready to run
    Ready,
    /// Process is currently running
    Running,
    /// Process is waiting for I/O or event
    Waiting,
    /// Process has terminated
    Terminated,
}

/// Scheduling policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchedulingPolicy {
    /// Round-robin with fixed time quantum
    RoundRobin,
    /// Priority-based preemptive scheduling
    Priority,
    /// Fair scheduling (CFS-inspired with virtual runtime)
    Fair,
}

/// Process metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub state: ProcessState,
    pub priority: Priority,
    #[serde(skip_serializing_if = "is_none")]
    pub os_pid: Option<u32>,
}

impl ProcessInfo {
    pub fn new(pid: Pid, name: String, priority: Priority) -> Self {
        Self {
            pid,
            name,
            state: ProcessState::Ready,
            priority,
            os_pid: None,
        }
    }

    pub fn with_os_pid(mut self, os_pid: u32) -> Self {
        self.os_pid = Some(os_pid);
        self
    }
}

/// Configuration for process execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionConfig {
    pub command: String,
    #[serde(skip_serializing_if = "crate::core::serde::is_empty_vec")]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "crate::core::serde::is_empty_vec")]
    pub env_vars: Vec<(String, String)>,
    #[serde(skip_serializing_if = "is_none")]
    pub working_dir: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub capture_output: bool,
}

impl ExecutionConfig {
    pub fn new(command: String) -> Self {
        Self {
            command,
            args: vec![],
            env_vars: vec![],
            working_dir: None,
            capture_output: true,
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_env(mut self, env_vars: Vec<(String, String)>) -> Self {
        self.env_vars = env_vars;
        self
    }

    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn with_output_capture(mut self, capture: bool) -> Self {
        self.capture_output = capture;
        self
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SchedulerStats {
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub total_scheduled: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub context_switches: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub preemptions: u64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub active_processes: usize,
    pub policy: SchedulingPolicy,
    pub quantum_micros: u64,
}

/// Per-process CPU usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProcessStats {
    pub pid: Pid,
    pub priority: Priority,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub cpu_time_micros: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub vruntime: u64,
    #[serde(skip_serializing_if = "is_false")]
    pub is_current: bool,
}

impl ProcessStats {
    pub fn cpu_time(&self) -> Duration {
        Duration::from_micros(self.cpu_time_micros)
    }

    pub fn cpu_time_ms(&self) -> f64 {
        self.cpu_time_micros as f64 / 1000.0
    }
}

/// Process execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionStats {
    pub pid: Pid,
    #[serde(skip_serializing_if = "is_none")]
    pub os_pid: Option<u32>,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub cpu_time_micros: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub wall_time_micros: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub context_switches: u64,
}

impl ExecutionStats {
    pub fn new(pid: Pid, os_pid: Option<u32>) -> Self {
        Self {
            pid,
            os_pid,
            cpu_time_micros: 0,
            wall_time_micros: 0,
            context_switches: 0,
        }
    }

    pub fn cpu_usage_percent(&self) -> f64 {
        if self.wall_time_micros == 0 {
            0.0
        } else {
            (self.cpu_time_micros as f64 / self.wall_time_micros as f64) * 100.0
        }
    }
}

/// Process resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProcessResources {
    pub pid: Pid,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub memory_bytes: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub cpu_time_micros: u64,
    #[serde(skip_serializing_if = "crate::core::serde::is_default")]
    pub open_files: u32,
    #[serde(skip_serializing_if = "crate::core::serde::is_default")]
    pub child_processes: u32,
}

impl ProcessResources {
    pub fn new(pid: Pid) -> Self {
        Self {
            pid,
            memory_bytes: 0,
            cpu_time_micros: 0,
            open_files: 0,
            child_processes: 0,
        }
    }
}
