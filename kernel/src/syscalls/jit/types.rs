/*!
 * JIT Types
 * Data structures for JIT compilation
 */

use crate::core::types::Pid;
use crate::syscalls::types::{Syscall, SyscallResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Pattern representing a syscall for JIT compilation
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum SyscallPattern {
    /// Simple syscalls without parameters
    Simple(SimpleSyscallType),
    /// File operations
    FileOp(FileOpType),
    /// IPC operations
    IpcOp(IpcOpType),
    /// Network operations
    NetworkOp(NetworkOpType),
    /// Memory operations
    MemoryOp(MemoryOpType),
}

impl SyscallPattern {
    /// Extract pattern from a syscall
    pub fn from_syscall(syscall: &Syscall) -> Self {
        match syscall {
            Syscall::GetProcessList => Self::Simple(SimpleSyscallType::GetProcessList),
            Syscall::GetProcessInfo { .. } => Self::Simple(SimpleSyscallType::GetProcessInfo),
            Syscall::KillProcess { .. } => Self::Simple(SimpleSyscallType::KillProcess),

            Syscall::Open { .. } => Self::FileOp(FileOpType::Open),
            Syscall::ReadFile { .. } => Self::FileOp(FileOpType::Read),
            Syscall::WriteFile { .. } => Self::FileOp(FileOpType::Write),
            Syscall::Close { .. } => Self::FileOp(FileOpType::Close),
            Syscall::FileStat { .. } => Self::FileOp(FileOpType::Stat),

            Syscall::CreatePipe { .. } => Self::IpcOp(IpcOpType::PipeCreate),
            Syscall::WritePipe { .. } => Self::IpcOp(IpcOpType::PipeWrite),
            Syscall::ReadPipe { .. } => Self::IpcOp(IpcOpType::PipeRead),
            Syscall::SendQueue { .. } => Self::IpcOp(IpcOpType::QueueSend),
            Syscall::ReceiveQueue { .. } => Self::IpcOp(IpcOpType::QueueReceive),

            Syscall::NetworkRequest { .. } => Self::NetworkOp(NetworkOpType::Request),
            Syscall::Socket { .. } => Self::NetworkOp(NetworkOpType::Socket),
            Syscall::Connect { .. } => Self::NetworkOp(NetworkOpType::Connect),
            Syscall::Send { .. } => Self::NetworkOp(NetworkOpType::Send),
            Syscall::Recv { .. } => Self::NetworkOp(NetworkOpType::Receive),

            Syscall::Mmap { .. } => Self::MemoryOp(MemoryOpType::Mmap),
            Syscall::Munmap { .. } => Self::MemoryOp(MemoryOpType::Munmap),

            _ => Self::Simple(SimpleSyscallType::Other),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum SimpleSyscallType {
    GetProcessList,
    GetProcessInfo,
    KillProcess,
    Other,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum FileOpType {
    Open,
    Read,
    Write,
    Close,
    Stat,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum IpcOpType {
    PipeCreate,
    PipeWrite,
    PipeRead,
    QueueSend,
    QueueReceive,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum NetworkOpType {
    Request,
    Socket,
    Connect,
    Send,
    Receive,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MemoryOpType {
    Mmap,
    Munmap,
    Malloc,
    Free,
}

/// Compiled handler for a syscall pattern
pub struct CompiledHandler {
    pattern: SyscallPattern,
    optimizations: Vec<Optimization>,
    executor: Box<dyn Fn(Pid, &Syscall) -> SyscallResult + Send + Sync>,
}

impl CompiledHandler {
    /// Create a new compiled handler
    pub fn new(
        pattern: SyscallPattern,
        optimizations: Vec<Optimization>,
        executor: Box<dyn Fn(Pid, &Syscall) -> SyscallResult + Send + Sync>,
    ) -> Self {
        Self {
            pattern,
            optimizations,
            executor,
        }
    }

    /// Execute the compiled handler
    pub fn execute(&self, pid: Pid, syscall: &Syscall) -> SyscallResult {
        (self.executor)(pid, syscall)
    }

    /// Get the pattern
    pub fn pattern(&self) -> &SyscallPattern {
        &self.pattern
    }

    /// Get applied optimizations
    pub fn optimizations(&self) -> &[Optimization] {
        &self.optimizations
    }
}

/// Optimization applied to a compiled handler
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Optimization {
    /// Inline small functions
    Inlining,
    /// Skip unnecessary permission checks
    SkipPermissionCheck,
    /// Use fast path for common cases
    FastPath,
    /// Eliminate bounds checks
    EliminateBoundsCheck,
    /// Use specialized implementation
    Specialize,
    /// Cache frequently accessed data
    DataCache,
}

/// JIT compilation error
#[derive(Error, Debug)]
pub enum JitError {
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),

    #[error("Unsupported pattern: {0:?}")]
    UnsupportedPattern(SyscallPattern),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Statistics for JIT compilation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JitStats {
    /// Number of compiled hot paths
    pub compiled_paths: usize,
    /// Number of JIT cache hits
    pub jit_hits: u64,
    /// Number of JIT cache misses
    pub jit_misses: u64,
    /// Total time saved by JIT (estimated in microseconds)
    pub time_saved_us: u64,
}

impl fmt::Display for JitStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JIT Stats: {} compiled paths, {} hits, {} misses, {} us saved",
            self.compiled_paths, self.jit_hits, self.jit_misses, self.time_saved_us
        )
    }
}

