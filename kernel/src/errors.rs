/*!
 * Error Types
 * Centralized error handling with thiserror
 */

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Out of memory: requested {requested} bytes, available {available} bytes ({used} used / {total} total)")]
    OutOfMemory {
        requested: usize,
        available: usize,
        used: usize,
        total: usize,
    },

    #[error("Process memory limit exceeded: requested {requested} bytes, limit {limit} bytes, current {current} bytes")]
    ProcessLimitExceeded {
        requested: usize,
        limit: usize,
        current: usize,
    },

    #[error("Invalid memory address: 0x{0:x}")]
    InvalidAddress(usize),
}

#[derive(Error, Debug)]
pub enum SyscallError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Path error: {0}")]
    PathError(String),

    #[error("Process error: {0}")]
    ProcessError(String),
}

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Sandbox not found for PID {0}")]
    NotFound(u32),

    #[error("Capability {0:?} not granted")]
    MissingCapability(String),

    #[error("Path {0:?} not accessible")]
    PathBlocked(String),
}

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

