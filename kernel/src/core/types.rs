/*!
 * Core Types
 * Common types used across the kernel
 */

use crate::core::serialization::serde::{is_zero_u32, is_zero_u64, is_zero_usize, system_time_micros};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
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
///
/// # Must Use
/// This Result type must be used - ignoring errors can lead to undefined behavior
pub type KernelResult<T> = Result<T, super::errors::KernelError>;

/// System information
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub hostname: String,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub uptime_secs: u64,
    #[serde(with = "system_time_micros")]
    pub boot_time: SystemTime,
}

impl SystemInfo {
    /// Create a new SystemInfo with the given parameters
    #[inline]
    #[must_use]
    pub fn new(
        os_name: String,
        os_version: String,
        architecture: String,
        hostname: String,
        uptime_secs: u64,
        boot_time: SystemTime,
    ) -> Self {
        Self {
            os_name,
            os_version,
            architecture,
            hostname,
            uptime_secs,
            boot_time,
        }
    }
}

/// Resource limits for processes
///
/// # Performance
/// - Packed C layout for frequent limit checks
/// - Copy-optimized for stack allocation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResourceLimits {
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub max_memory_bytes: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub max_cpu_time_ms: u64,
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub max_file_descriptors: u32,
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub max_processes: u32,
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub max_network_connections: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: crate::core::limits::HIGH_MEMORY_THRESHOLD,
            max_cpu_time_ms: 60_000, // 1 minute
            max_file_descriptors: crate::core::limits::STANDARD_MAX_FILE_DESCRIPTORS as u32,
            max_processes: 10,
            max_network_connections: crate::core::limits::MAX_NETWORK_CONNECTIONS,
        }
    }
}

impl ResourceLimits {
    /// Create minimal resource limits (for untrusted processes)
    pub const fn minimal() -> Self {
        Self {
            max_memory_bytes: 10 * 1024 * 1024, // 10MB - intentionally minimal
            max_cpu_time_ms: 5_000,             // 5 seconds
            max_file_descriptors: 10,
            max_processes: 1,
            max_network_connections: 0,
        }
    }

    /// Create privileged resource limits (for system processes)
    pub const fn privileged() -> Self {
        Self {
            max_memory_bytes: 500 * 1024 * 1024, // 500MB - privileged processes
            max_cpu_time_ms: 0,                  // Unlimited
            max_file_descriptors: 10000,
            max_processes: 100,
            max_network_connections: 1000,
        }
    }

    /// Check if this limit allows unlimited resources (0 = unlimited for numeric fields)
    ///
    /// # Performance
    /// Hot path - frequently checked in scheduling and resource management
    #[inline(always)]
    #[must_use]
    pub const fn is_unlimited_cpu(&self) -> bool {
        self.max_cpu_time_ms == 0
    }

    /// Validate that all limits are within reasonable bounds
    #[must_use = "validation result must be checked"]
    pub const fn validate(&self) -> Result<(), &'static str> {
        if self.max_memory_bytes > 10 * 1024 * 1024 * 1024 {
            // 10GB max
            return Err("max_memory_bytes exceeds 10GB");
        }
        if self.max_file_descriptors > 100_000 {
            return Err("max_file_descriptors exceeds 100,000");
        }
        if self.max_processes > 10_000 {
            return Err("max_processes exceeds 10,000");
        }
        if self.max_network_connections > 100_000 {
            return Err("max_network_connections exceeds 100,000");
        }
        Ok(())
    }
}

/// Execution configuration for processes
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ExecutionConfig {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<(String, String)>,
    pub working_dir: Option<String>,
}

impl ExecutionConfig {
    /// Create a new execution configuration with the given command
    #[inline]
    #[must_use]
    pub fn new(command: String) -> Self {
        Self {
            command,
            args: Vec::new(),
            env: Vec::new(),
            working_dir: None,
        }
    }

    /// Add arguments to the execution configuration (builder pattern)
    #[inline]
    #[must_use]
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Add a single argument to the execution configuration
    #[inline]
    #[must_use]
    pub fn with_arg(mut self, arg: String) -> Self {
        self.args.push(arg);
        self
    }

    /// Add environment variables to the execution configuration (builder pattern)
    #[inline]
    #[must_use]
    pub fn with_env(mut self, env: Vec<(String, String)>) -> Self {
        self.env = env;
        self
    }

    /// Add a single environment variable to the execution configuration
    #[inline]
    #[must_use]
    pub fn with_env_var(mut self, key: String, value: String) -> Self {
        self.env.push((key, value));
        self
    }

    /// Set the working directory for the execution configuration (builder pattern)
    #[inline]
    #[must_use]
    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Validate the execution configuration
    #[must_use = "validation result must be checked"]
    pub fn validate(&self) -> Result<(), String> {
        if self.command.is_empty() {
            return Err("command cannot be empty".to_string());
        }
        Ok(())
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_bytes, 100 * 1024 * 1024);
        assert_eq!(limits.max_cpu_time_ms, 60_000);
    }

    #[test]
    fn test_resource_limits_minimal() {
        let limits = ResourceLimits::minimal();
        assert_eq!(limits.max_memory_bytes, 10 * 1024 * 1024);
        assert_eq!(limits.max_processes, 1);
    }

    #[test]
    fn test_resource_limits_privileged() {
        let limits = ResourceLimits::privileged();
        assert_eq!(limits.max_memory_bytes, 500 * 1024 * 1024);
        assert!(limits.is_unlimited_cpu());
    }

    #[test]
    fn test_resource_limits_validation() {
        let mut limits = ResourceLimits::default();
        assert!(limits.validate().is_ok());

        limits.max_memory_bytes = 20 * 1024 * 1024 * 1024; // 20GB
        assert!(limits.validate().is_err());
    }

    #[test]
    fn test_resource_limits_serialization() {
        let limits = ResourceLimits::default();
        let json = serde_json::to_string(&limits).unwrap();
        let deserialized: ResourceLimits = serde_json::from_str(&json).unwrap();
        assert_eq!(limits, deserialized);
    }

    #[test]
    fn test_execution_config_builder() {
        let config = ExecutionConfig::new("test".to_string())
            .with_arg("arg1".to_string())
            .with_arg("arg2".to_string())
            .with_env_var("KEY".to_string(), "VALUE".to_string())
            .with_working_dir("/tmp".to_string());

        assert_eq!(config.command, "test");
        assert_eq!(config.args, vec!["arg1", "arg2"]);
        assert_eq!(config.env, vec![("KEY".to_string(), "VALUE".to_string())]);
        assert_eq!(config.working_dir, Some("/tmp".to_string()));
    }

    #[test]
    fn test_execution_config_serialization() {
        let config = ExecutionConfig::new("test".to_string())
            .with_args(vec!["arg1".to_string()])
            .with_working_dir("/tmp".to_string());

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ExecutionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_execution_config_validation() {
        let valid_config = ExecutionConfig::new("test".to_string());
        assert!(valid_config.validate().is_ok());

        let invalid_config = ExecutionConfig::new(String::new());
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_execution_config_skip_empty_fields() {
        let config = ExecutionConfig::new("test".to_string());
        let json = serde_json::to_string(&config).unwrap();
        // Empty args and env should not appear in JSON
        assert!(!json.contains("\"args\""));
        assert!(!json.contains("\"env\""));
    }
}
