/*!
 * Core Traits
 * Fundamental kernel abstractions
 */

use super::errors::*;
use super::types::*;

/// Process lifecycle management
pub trait ProcessLifecycle: Send + Sync {
    /// Create a new process
    fn create(&self, name: String, priority: Priority) -> KernelResult<Pid>;

    /// Terminate a process
    fn terminate(&self, pid: Pid) -> KernelResult<()>;

    /// Check if a process exists
    fn exists(&self, pid: Pid) -> bool;

    /// Wait for a process to complete
    fn wait(&self, pid: Pid, timeout_ms: Option<u64>) -> KernelResult<()>;
}

/// Resource management trait
pub trait ResourceManager: Send + Sync {
    /// Allocate a resource
    fn allocate(&self, pid: Pid, size: Size) -> KernelResult<Address>;

    /// Deallocate a resource
    fn deallocate(&self, address: Address) -> KernelResult<()>;

    /// Get resource usage for a process
    fn usage(&self, pid: Pid) -> KernelResult<Size>;

    /// Check if allocation would exceed limits
    fn check_limits(&self, pid: Pid, size: Size) -> KernelResult<()>;
}

/// Security policy enforcement
pub trait SecurityPolicy: Send + Sync {
    /// Check if an operation is permitted
    fn check_permission(&self, pid: Pid, operation: &str) -> std::result::Result<(), SandboxError>;

    /// Grant a capability to a process
    fn grant_capability(&self, pid: Pid, capability: &str) -> KernelResult<()>;

    /// Revoke a capability from a process
    fn revoke_capability(&self, pid: Pid, capability: &str) -> KernelResult<()>;

    /// Check if a path is accessible
    fn check_path_access(
        &self,
        pid: Pid,
        path: &str,
        write: bool,
    ) -> std::result::Result<(), SandboxError>;
}

/// Scheduler interface
pub trait Scheduler: Send + Sync {
    /// Select the next process to run
    fn schedule_next(&self) -> Option<Pid>;

    /// Yield current process
    fn yield_process(&self, pid: Pid) -> KernelResult<()>;

    /// Set process priority
    fn set_priority(&self, pid: Pid, priority: Priority) -> KernelResult<()>;

    /// Get scheduler statistics
    fn statistics(&self) -> SchedulerStatistics;
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStatistics {
    pub total_scheduled: u64,
    pub context_switches: u64,
    pub preemptions: u64,
    pub active_processes: usize,
}

/// Event notification trait
pub trait EventNotifier: Send + Sync {
    /// Register for event notifications
    fn subscribe(&self, pid: Pid, event_type: &str) -> KernelResult<()>;

    /// Unregister from event notifications
    fn unsubscribe(&self, pid: Pid, event_type: &str) -> KernelResult<()>;

    /// Emit an event
    fn emit(&self, event_type: &str, data: Vec<u8>) -> KernelResult<()>;
}

/// Clonable manager trait for shared kernel components
pub trait ClonableManager: Clone + Send + Sync {}

/// Statistics provider trait
pub trait Statistics {
    type Stats;

    /// Get current statistics
    fn stats(&self) -> Self::Stats;
}
