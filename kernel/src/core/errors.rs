/*!
 * Error Types
 * Centralized error handling with thiserror
 */

use thiserror::Error;

// Re-export MemoryError from memory module
pub use crate::memory::types::MemoryError;

// Re-export SandboxError from security module
pub use crate::security::types::SandboxError;

// Re-export SyscallError from syscalls module
pub use crate::syscalls::types::SyscallError;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Process {0} not found")]
    NotFound(u32),

    #[error("Failed to create process: {0}")]
    CreationFailed(String),

    #[error("Memory allocation failed: {0}")]
    MemoryError(#[from] MemoryError),
}

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Process {0} not found in scheduler")]
    ProcessNotFound(u32),

    #[error("Scheduler queue full: {0}")]
    QueueFull(String),

    #[error("Invalid scheduling policy: {0}")]
    InvalidPolicy(String),

    #[error("Cannot schedule: {0}")]
    SchedulingFailed(String),
}

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),

    #[error("Process error: {0}")]
    Process(#[from] ProcessError),

    #[error("Sandbox error: {0}")]
    Sandbox(#[from] SandboxError),

    #[error("Syscall error: {0}")]
    Syscall(#[from] SyscallError),

    #[error("Scheduler error: {0}")]
    Scheduler(#[from] SchedulerError),

    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for kernel operations
pub type Result<T> = std::result::Result<T, KernelError>;
