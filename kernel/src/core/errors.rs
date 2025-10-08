/*!
 * Error Types
 * Centralized error handling with thiserror, miette, and serde support
 */

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-export MemoryError from memory module
pub use crate::memory::MemoryError;

// Re-export SandboxError from security module
pub use crate::security::types::SandboxError;

// Re-export SyscallError from syscalls module
pub use crate::syscalls::types::SyscallError;

/// Process-related errors with serialization support
#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Diagnostic)]
#[serde(tag = "error_type", content = "details", rename_all = "snake_case")]
pub enum ProcessError {
    #[error("Process {0} not found")]
    #[diagnostic(
        code(process::not_found),
        help("The process may have terminated or never existed. Check PID validity.")
    )]
    NotFound(u32),

    #[error("Failed to create process: {0}")]
    #[diagnostic(
        code(process::creation_failed),
        help("Check system resources and permissions. View logs for details.")
    )]
    CreationFailed(String),

    #[error("Memory allocation failed: {0}")]
    #[diagnostic(
        code(process::memory_allocation_failed),
        help("System may be low on memory. Consider freeing resources.")
    )]
    MemoryAllocationFailed(String),

    #[error("Invalid process state: {0}")]
    #[diagnostic(
        code(process::invalid_state),
        help("Operation cannot be performed in current process state.")
    )]
    InvalidState(String),

    #[error("Process limit reached: {0}")]
    #[diagnostic(
        code(process::limit_reached),
        help("Maximum number of processes reached. Terminate unused processes.")
    )]
    LimitReached(String),

    #[error("Permission denied: {0}")]
    #[diagnostic(
        code(process::permission_denied),
        help("Insufficient permissions to perform this operation.")
    )]
    PermissionDenied(String),
}

// Allow conversion from MemoryError to ProcessError
impl From<MemoryError> for ProcessError {
    fn from(err: MemoryError) -> Self {
        ProcessError::MemoryAllocationFailed(err.to_string())
    }
}

/// Scheduler-related errors with serialization support
#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Diagnostic)]
#[serde(tag = "error_type", content = "details", rename_all = "snake_case")]
pub enum SchedulerError {
    #[error("Process {0} not found in scheduler")]
    #[diagnostic(
        code(scheduler::process_not_found),
        help("Process may not be scheduled or has been removed.")
    )]
    ProcessNotFound(u32),

    #[error("Scheduler queue full: {0}")]
    #[diagnostic(
        code(scheduler::queue_full),
        help("Too many processes in scheduler queue. Wait for processes to complete.")
    )]
    QueueFull(String),

    #[error("Invalid scheduling policy: {0}")]
    #[diagnostic(
        code(scheduler::invalid_policy),
        help("Use RoundRobin, Priority, or Fair scheduling policy.")
    )]
    InvalidPolicy(String),

    #[error("Cannot schedule: {0}")]
    #[diagnostic(
        code(scheduler::scheduling_failed),
        help("Scheduling operation failed. Check system state and resources.")
    )]
    SchedulingFailed(String),

    #[error("Priority out of range: {0}")]
    #[diagnostic(
        code(scheduler::invalid_priority),
        help("Priority must be between 0 and 255.")
    )]
    InvalidPriority(String),

    #[error("Deadlock detected: {0}")]
    #[diagnostic(
        code(scheduler::deadlock_detected),
        help("Circular dependency detected between processes. Review process dependencies.")
    )]
    DeadlockDetected(String),
}

/// Unified kernel error type with miette diagnostics
/// Note: Some variants don't support Serialize due to complex error types
#[derive(Error, Debug, Diagnostic)]
pub enum KernelError {
    #[error("Memory error: {0}")]
    #[diagnostic(transparent)]
    Memory(#[from] MemoryError),

    #[error("Process error: {0}")]
    #[diagnostic(transparent)]
    Process(#[from] ProcessError),

    #[error("Sandbox error: {0}")]
    Sandbox(#[from] SandboxError),

    #[error("Syscall error: {0}")]
    Syscall(#[from] SyscallError),

    #[error("Scheduler error: {0}")]
    #[diagnostic(transparent)]
    Scheduler(#[from] SchedulerError),

    #[error("gRPC error: {0}")]
    #[diagnostic(
        code(kernel::grpc_error),
        help("Network or gRPC communication failed. Check connectivity.")
    )]
    Grpc(#[from] tonic::Status),

    #[error("Internal error: {0}")]
    #[diagnostic(
        code(kernel::internal_error),
        help("An unexpected internal error occurred. Please report this issue.")
    )]
    Internal(String),

    #[error("I/O error: {0}")]
    #[diagnostic(
        code(kernel::io_error),
        help("Filesystem or I/O operation failed. Check file permissions and disk space.")
    )]
    Io(String),

    #[error("Configuration error: {0}")]
    #[diagnostic(
        code(kernel::configuration_error),
        help("Invalid configuration. Review configuration parameters.")
    )]
    Configuration(String),

    #[error("Not supported: {0}")]
    #[diagnostic(
        code(kernel::not_supported),
        help("This operation is not supported on this platform or configuration.")
    )]
    NotSupported(String),

    #[error("Timeout: {0}")]
    #[diagnostic(
        code(kernel::timeout),
        help("Operation exceeded timeout limit. Try increasing timeout or check system load.")
    )]
    Timeout(String),
}

// Implement conversion from std::io::Error
impl From<std::io::Error> for KernelError {
    fn from(err: std::io::Error) -> Self {
        KernelError::Io(err.to_string())
    }
}

// Implement conversion from String for convenience
impl From<String> for KernelError {
    fn from(msg: String) -> Self {
        KernelError::Internal(msg)
    }
}

impl From<&str> for KernelError {
    fn from(msg: &str) -> Self {
        KernelError::Internal(msg.to_string())
    }
}

/// Serializable error representation for API responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct SerializableError {
    pub error_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl SerializableError {
    /// Create a new serializable error
    pub fn new(error_type: String, message: String) -> Self {
        Self {
            error_type,
            message,
            details: None,
        }
    }

    /// Create a new serializable error with details
    pub fn with_details(error_type: String, message: String, details: String) -> Self {
        Self {
            error_type,
            message,
            details: Some(details),
        }
    }
}

impl From<ProcessError> for SerializableError {
    fn from(err: ProcessError) -> Self {
        SerializableError::new("process_error".to_string(), err.to_string())
    }
}

impl From<SchedulerError> for SerializableError {
    fn from(err: SchedulerError) -> Self {
        SerializableError::new("scheduler_error".to_string(), err.to_string())
    }
}

impl From<KernelError> for SerializableError {
    fn from(err: KernelError) -> Self {
        let error_type = match &err {
            KernelError::Memory(_) => "memory_error",
            KernelError::Process(_) => "process_error",
            KernelError::Sandbox(_) => "sandbox_error",
            KernelError::Syscall(_) => "syscall_error",
            KernelError::Scheduler(_) => "scheduler_error",
            KernelError::Grpc(_) => "grpc_error",
            KernelError::Internal(_) => "internal_error",
            KernelError::Io(_) => "io_error",
            KernelError::Configuration(_) => "configuration_error",
            KernelError::NotSupported(_) => "not_supported",
            KernelError::Timeout(_) => "timeout",
        };
        SerializableError::new(error_type.to_string(), err.to_string())
    }
}

/// Result type for kernel operations
///
/// # Must Use
/// Kernel operations can fail and must be handled to prevent undefined behavior
pub type Result<T> = std::result::Result<T, KernelError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_error_serialization() {
        let error = ProcessError::NotFound(123);
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ProcessError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn test_scheduler_error_serialization() {
        let error = SchedulerError::QueueFull("limit reached".to_string());
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: SchedulerError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn test_serializable_error_creation() {
        let error = SerializableError::new("test_error".to_string(), "test message".to_string());
        assert_eq!(error.error_type, "test_error");
        assert_eq!(error.message, "test message");
        assert_eq!(error.details, None);
    }

    #[test]
    fn test_serializable_error_with_details() {
        let error = SerializableError::with_details(
            "test_error".to_string(),
            "test message".to_string(),
            "extra info".to_string(),
        );
        assert_eq!(error.details, Some("extra info".to_string()));
    }

    #[test]
    fn test_serializable_error_from_process_error() {
        let process_error = ProcessError::NotFound(123);
        let serializable: SerializableError = process_error.into();
        assert_eq!(serializable.error_type, "process_error");
    }

    #[test]
    fn test_kernel_error_display() {
        let error = KernelError::Internal("test error".to_string());
        assert_eq!(error.to_string(), "Internal error: test error");
    }

    #[test]
    fn test_kernel_error_from_string() {
        let error: KernelError = "test error".into();
        assert!(matches!(error, KernelError::Internal(_)));
    }

    #[test]
    fn test_process_error_from_memory_error() {
        let memory_error = MemoryError::OutOfMemory {
            requested: 1024,
            available: 512,
            used: 512,
            total: 1024,
        };
        let process_error: ProcessError = memory_error.into();
        assert!(matches!(
            process_error,
            ProcessError::MemoryAllocationFailed(_)
        ));
    }
}
