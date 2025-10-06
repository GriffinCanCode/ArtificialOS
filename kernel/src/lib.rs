/*!
 * AgentOS Kernel Library
 * Core kernel functionality exposed as a library
 */

// Module organization
pub mod api;
pub mod core;
pub mod ipc;
pub mod memory;
pub mod monitoring;
pub mod process;
pub mod security;
pub mod syscalls;
pub mod vfs;

// Re-exports for backwards compatibility
// API
pub use api::start_grpc_server;

// Core
pub use core::*;

// IPC
pub use ipc::{
    IPCManager, PipeError, PipeManager, PipeStats, ShmError, ShmManager, ShmPermission, ShmStats,
};

// Memory
pub use memory::{MemoryBlock, MemoryError, MemoryManager, MemoryStats, ProcessMemoryStats};

// Monitoring
pub use monitoring::{MetricsCollector, MetricsSnapshot};

// Process
pub use process::{
    ExecutionConfig, ProcessExecutorImpl as ProcessExecutor, ProcessInfo as Process,
    ProcessManagerBuilder, ProcessManagerImpl as ProcessManager, ProcessState, ProcessStats,
    Scheduler, SchedulerStats, SchedulingPolicy as Policy,
};

// Security
pub use security::{
    Capability, LimitManager, Limits, LimitsError, ResourceLimits, SandboxConfig, SandboxError,
    SandboxManager, SandboxStats, SecurityError, SecurityEvent,
};

// Syscalls
pub use syscalls::{
    FileDescriptorSyscalls, FileSystemSyscalls, IpcSyscalls, MemorySyscalls, NetworkSyscalls,
    ProcessSyscalls, SchedulerSyscalls, SignalSyscalls, Syscall, SyscallError, SyscallExecutor,
    SyscallExecutorTrait, SyscallResult, SystemInfoSyscalls, TimeSyscalls,
};

// VFS
pub use vfs::{
    Entry, FileSystem, FileType, LocalFS, MemFS, Metadata, MountManager, MountPoint, OpenFile,
    OpenFlags, OpenMode, Permissions, VfsError, VfsResult,
};
