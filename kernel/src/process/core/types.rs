/*!
 * Process Types
 * Common types for process management
 */

use crate::core::serialization::serde::{is_false, is_none, is_zero_u64, is_zero_usize};
use crate::core::types::{Pid, Priority};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Process operation result
///
/// # Must Use
/// Process operations can fail and must be handled to prevent resource leaks
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
    /// Process structure is being created (not yet initialized)
    Creating,
    /// Process resources are being initialized (lifecycle hooks running)
    Initializing,
    /// Process is ready to run (fully initialized)
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
///
/// # Performance
/// - Cache-line aligned for frequent scheduler access
#[repr(C, align(64))]
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
    #[inline]
    #[must_use]
    pub fn new(pid: Pid, name: String, priority: Priority) -> Self {
        Self {
            pid,
            name,
            state: ProcessState::Creating, // Start in Creating state
            priority,
            os_pid: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn with_os_pid(mut self, os_pid: u32) -> Self {
        self.os_pid = Some(os_pid);
        self
    }

    /// Check if process is running
    ///
    /// # Performance
    /// Hot path - frequently checked by scheduler
    #[inline(always)]
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self.state, ProcessState::Running)
    }

    /// Check if process is ready
    ///
    /// # Performance
    /// Hot path - frequently checked by scheduler
    #[inline(always)]
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self.state, ProcessState::Ready)
    }

    /// Check if process is terminated
    #[inline(always)]
    #[must_use]
    pub const fn is_terminated(&self) -> bool {
        matches!(self.state, ProcessState::Terminated)
    }

    /// Check if process is initializing
    #[inline(always)]
    #[must_use]
    pub const fn is_initializing(&self) -> bool {
        matches!(
            self.state,
            ProcessState::Creating | ProcessState::Initializing
        )
    }

    /// Check if process can be scheduled
    ///
    /// # Performance
    /// Hot path - checked by scheduler before adding to queue
    #[inline(always)]
    #[must_use]
    pub const fn can_be_scheduled(&self) -> bool {
        matches!(
            self.state,
            ProcessState::Ready | ProcessState::Running | ProcessState::Waiting
        )
    }
}

/// Configuration for process execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionConfig {
    pub command: String,
    #[serde(skip_serializing_if = "crate::core::serialization::serde::is_empty_vec")]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "crate::core::serialization::serde::is_empty_vec")]
    pub env_vars: Vec<(String, String)>,
    #[serde(skip_serializing_if = "is_none")]
    pub working_dir: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub capture_output: bool,
    #[serde(skip_serializing_if = "is_none")]
    pub limits: Option<crate::security::types::Limits>,
}

impl ExecutionConfig {
    #[inline]
    #[must_use]
    pub fn new(command: String) -> Self {
        Self {
            command,
            args: vec![],
            env_vars: vec![],
            working_dir: None,
            capture_output: true,
            limits: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    #[inline]
    #[must_use]
    pub fn with_env(mut self, env_vars: Vec<(String, String)>) -> Self {
        self.env_vars = env_vars;
        self
    }

    #[inline]
    #[must_use]
    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    #[inline]
    #[must_use]
    pub fn with_output_capture(mut self, capture: bool) -> Self {
        self.capture_output = capture;
        self
    }

    #[inline]
    #[must_use]
    pub fn with_limits(mut self, limits: crate::security::types::Limits) -> Self {
        self.limits = Some(limits);
        self
    }
}

/// Scheduler statistics
///
/// # Performance
/// - Cache-line aligned for frequent reads by monitoring
#[repr(C, align(64))]
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
///
/// # Performance
/// - Packed C layout for efficient copying in scheduler
#[repr(C)]
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
    /// Create new process stats
    #[inline]
    #[must_use]
    pub const fn new(pid: Pid, priority: Priority) -> Self {
        Self {
            pid,
            priority,
            cpu_time_micros: 0,
            vruntime: 0,
            is_current: false,
        }
    }

    /// Get CPU time as Duration
    ///
    /// # Performance
    /// Hot path - frequently called by scheduler
    #[inline(always)]
    #[must_use]
    pub const fn cpu_time(&self) -> Duration {
        Duration::from_micros(self.cpu_time_micros)
    }

    /// Get CPU time in milliseconds
    ///
    /// # Performance
    /// Hot path - frequently called for scheduling decisions
    #[inline(always)]
    #[must_use]
    pub const fn cpu_time_ms(&self) -> f64 {
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
    #[inline]
    #[must_use]
    pub const fn new(pid: Pid, os_pid: Option<u32>) -> Self {
        Self {
            pid,
            os_pid,
            cpu_time_micros: 0,
            wall_time_micros: 0,
            context_switches: 0,
        }
    }

    #[inline]
    #[must_use]
    pub const fn cpu_usage_percent(&self) -> f64 {
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
    #[serde(skip_serializing_if = "crate::core::serialization::serde::is_default")]
    pub open_files: u32,
    #[serde(skip_serializing_if = "crate::core::serialization::serde::is_default")]
    pub child_processes: u32,
}

impl ProcessResources {
    #[inline]
    #[must_use]
    pub const fn new(pid: Pid) -> Self {
        Self {
            pid,
            memory_bytes: 0,
            cpu_time_micros: 0,
            open_files: 0,
            child_processes: 0,
        }
    }

    /// Check if process is using significant resources
    ///
    /// # Performance
    /// Hot path - frequently checked for resource management decisions
    #[inline(always)]
    #[must_use]
    pub const fn is_high_usage(&self) -> bool {
        self.memory_bytes > crate::core::limits::HIGH_MEMORY_THRESHOLD
            || self.open_files > crate::core::limits::HIGH_FD_THRESHOLD
            || self.child_processes > 10
    }
}
