/*!
 * Core Types
 * Common types used across the kernel
 */

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Process ID type
pub type Pid = u32;

/// File descriptor type
pub type Fd = u32;

/// Socket descriptor type
pub type SockFd = u32;

/// Address type for memory operations
pub type Address = usize;

/// Size type for memory operations
pub type Size = usize;

/// Timestamp in microseconds since boot
pub type Timestamp = u64;

/// Priority level (0-255, higher is more important)
pub type Priority = u8;

/// Signal type
pub type Signal = u32;

/// Common result type for kernel operations
pub type KernelResult<T> = Result<T, super::errors::KernelError>;

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub hostname: String,
    pub uptime_secs: u64,
    pub boot_time: SystemTime,
}

/// Resource limits for processes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: usize,
    pub max_cpu_time_ms: u64,
    pub max_file_descriptors: u32,
    pub max_processes: u32,
    pub max_network_connections: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            max_cpu_time_ms: 60_000,              // 1 minute
            max_file_descriptors: 1024,
            max_processes: 10,
            max_network_connections: 100,
        }
    }
}

impl ResourceLimits {
    /// Create minimal resource limits (for untrusted processes)
    pub fn minimal() -> Self {
        Self {
            max_memory_bytes: 10 * 1024 * 1024, // 10MB
            max_cpu_time_ms: 5_000,              // 5 seconds
            max_file_descriptors: 10,
            max_processes: 1,
            max_network_connections: 0,
        }
    }

    /// Create privileged resource limits (for system processes)
    pub fn privileged() -> Self {
        Self {
            max_memory_bytes: 500 * 1024 * 1024, // 500MB
            max_cpu_time_ms: 0,                   // Unlimited
            max_file_descriptors: 10000,
            max_processes: 100,
            max_network_connections: 1000,
        }
    }
}

/// Execution configuration for processes
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub working_dir: Option<String>,
}

impl ExecutionConfig {
    pub fn new(command: String) -> Self {
        Self {
            command,
            args: Vec::new(),
            env: Vec::new(),
            working_dir: None,
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_env(mut self, env: Vec<(String, String)>) -> Self {
        self.env = env;
        self
    }

    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }
}
