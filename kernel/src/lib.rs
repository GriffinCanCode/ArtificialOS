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
pub mod permissions;
pub mod process;
pub mod scheduler;
pub mod security;
pub mod signals;
pub mod syscalls;
pub mod vfs;

// Re-exports for backwards compatibility
// API
pub use api::start_grpc_server;

// Core
pub use core::*;

// IPC
pub use ipc::{
    IPCManager, MapFlags, MmapEntry, MmapId, MmapManager, PipeError, PipeManager, PipeStats,
    ProtFlags, ShmError, ShmManager, ShmPermission, ShmStats,
};

// Memory
pub use memory::{MemoryBlock, MemoryError, MemoryManager, MemoryStats, ProcessMemoryStats};

// Monitoring
pub use monitoring::{init_tracing, MetricsCollector, MetricsSnapshot};

// Process
pub use process::{
    ExecutionConfig, ProcessExecutorImpl as ProcessExecutor, ProcessInfo as Process,
    ProcessManagerBuilder, ProcessManagerImpl as ProcessManager, ProcessState, ProcessStats,
    Scheduler, SchedulerCommand, SchedulerStats, SchedulerTask, SchedulingPolicy,
};

// Scheduler
pub use scheduler::{
    apply_priority_op, validate_priority, Policy as SchedulerPolicy, PriorityControl, PriorityOp,
    SchedulerControl, SchedulerStats as SchedulerStatsTrait, TimeQuantum, DEFAULT_PRIORITY,
    MAX_PRIORITY, MIN_PRIORITY,
};

// Permissions
pub use permissions::{
    Action, AuditEvent, AuditLogger, AuditSeverity, EvaluationContext, PermissionCache,
    PermissionChecker, PermissionManager, PermissionProvider, PermissionRequest,
    PermissionResponse, PolicyDecision as PermissionPolicyDecision,
    PolicyEngine, RequestContext, Resource, ResourceType,
};

// Security
pub use security::{
    Capability, EbpfManagerImpl, LimitManager, Limits, LimitsError, ResourceLimits, SandboxConfig,
    SandboxError, SandboxManager, SandboxStats, SecurityError, SecurityEvent,
};

// Signals
pub use signals::{
    PendingSignal, ProcessSignalState, Signal, SignalAction, SignalDelivery, SignalDisposition,
    SignalError, SignalHandler, SignalHandlerRegistry, SignalManager, SignalManagerImpl,
    SignalMasking, SignalOutcome, SignalQueue, SignalResult, SignalStateManager, SignalStats,
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
