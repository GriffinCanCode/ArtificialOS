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
pub use api::execution::{IoUringExecutor, IoUringManager};
pub use api::{
    kernel_proto, start_grpc_server, AsyncTaskManager, BatchExecutor, KernelServiceImpl,
    StreamingManager, TaskStatus,
};

// Core
pub use core::*;

// IPC
pub use ipc::{
    IPCManager, MapFlags, MmapEntry, MmapId, MmapManager, PipeError, PipeManager, PipeStats,
    ProtFlags, ShmError, ShmManager, ShmPermission, ShmStats, ZeroCopyIpc, ZeroCopyRing,
    ZeroCopyStats,
};

// Memory
pub use memory::{
    ascii_to_lower,
    ascii_to_upper,
    avg_u64,
    contains_byte,
    count_byte,
    find_byte,
    // SIMD operations
    init_simd,
    is_ascii,
    max_u64,
    min_u64,
    rfind_byte,
    simd_capabilities,
    simd_memcmp,
    simd_memcpy,
    simd_memmove,
    simd_memset,
    sum_u32,
    sum_u64,
    trim,
    MemoryBlock,
    MemoryError,
    MemoryManager,
    MemoryStats,
    ProcessMemoryStats,
    SimdCapabilities,
};

// Monitoring - Dual-layer observability system
pub use monitoring::{
    // Distributed tracing (Layer 1)
    init_tracing, span_syscall, span_operation, span_grpc,
    // Metrics (for gRPC endpoints)
    MetricsCollector, MetricsSnapshot,
    // Event streaming (Layer 2 - new primary API)
    Collector, Event, EventFilter, Category, Severity, Payload,
    Query, QueryResult, CommonQueries,
    Anomaly, Detector, Sampler,
    // Bridge for integration
    init_collector, global_collector,
};

// Process
pub use process::{
    ExecutionConfig, ProcessExecutorImpl as ProcessExecutor, ProcessInfo as Process,
    ProcessManagerBuilder, ProcessManagerImpl as ProcessManager, ProcessState, ProcessStats,
    Scheduler, SchedulerCommand, SchedulerStats, SchedulerTask, SchedulingPolicy,
};

// Process resource cleanup system
pub use process::resources;

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
    PermissionResponse, PolicyDecision as PermissionPolicyDecision, PolicyEngine, RequestContext,
    Resource, ResourceType,
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
    FileDescriptorSyscalls, FileSystemSyscalls, IpcSyscalls, JitManager, JitStats, MemorySyscalls,
    NetworkSyscalls, ProcessSyscalls, SchedulerSyscalls, SignalSyscalls, Syscall, SyscallError,
    SyscallExecutor, SyscallExecutorTrait, SyscallPattern, SyscallResult, SystemInfoSyscalls,
    TimeSyscalls,
};

// VFS
pub use vfs::{
    Entry, FileSystem, FileType, LocalFS, MemFS, Metadata, MountManager, MountPoint, OpenFile,
    OpenFlags, OpenMode, Permissions, VfsError, VfsResult,
};
