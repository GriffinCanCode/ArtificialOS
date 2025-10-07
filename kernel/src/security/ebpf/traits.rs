/*!
 * eBPF Traits
 * Platform-agnostic abstractions for eBPF operations
 */

use super::types::*;
use crate::core::types::Pid;

/// Core eBPF provider interface
pub trait EbpfProvider: Send + Sync {
    /// Check if eBPF is supported on this platform
    fn is_supported(&self) -> bool;

    /// Get the platform type
    fn platform(&self) -> EbpfPlatform;

    /// Load an eBPF program
    fn load_program(&self, config: ProgramConfig) -> EbpfResult<()>;

    /// Unload an eBPF program
    fn unload_program(&self, name: &str) -> EbpfResult<()>;

    /// Attach a loaded program
    fn attach_program(&self, name: &str) -> EbpfResult<()>;

    /// Detach an attached program
    fn detach_program(&self, name: &str) -> EbpfResult<()>;

    /// List all loaded programs
    fn list_programs(&self) -> Vec<ProgramInfo>;

    /// Get program info
    fn get_program_info(&self, name: &str) -> Option<ProgramInfo>;

    /// Get statistics
    fn stats(&self) -> EbpfStats;
}

/// Syscall filtering interface
pub trait SyscallFilterProvider: Send + Sync {
    /// Add a syscall filter rule
    fn add_filter(&self, filter: SyscallFilter) -> EbpfResult<()>;

    /// Remove a filter rule
    fn remove_filter(&self, filter_id: &str) -> EbpfResult<()>;

    /// Get all active filters
    fn get_filters(&self) -> Vec<SyscallFilter>;

    /// Clear all filters
    fn clear_filters(&self) -> EbpfResult<()>;

    /// Check if a syscall would be allowed
    fn check_syscall(&self, pid: Pid, syscall_nr: u64) -> bool;
}

/// Event monitoring interface
pub trait EventMonitor: Send + Sync {
    /// Subscribe to syscall events
    fn subscribe_syscall(&self, callback: EventCallback) -> EbpfResult<String>;

    /// Subscribe to network events
    fn subscribe_network(&self, callback: EventCallback) -> EbpfResult<String>;

    /// Subscribe to file events
    fn subscribe_file(&self, callback: EventCallback) -> EbpfResult<String>;

    /// Subscribe to all events
    fn subscribe_all(&self, callback: EventCallback) -> EbpfResult<String>;

    /// Unsubscribe from events
    fn unsubscribe(&self, subscription_id: &str) -> EbpfResult<()>;

    /// Get recent events
    fn get_recent_events(&self, limit: usize) -> Vec<EbpfEvent>;

    /// Get events for a specific PID
    fn get_events_by_pid(&self, pid: Pid, limit: usize) -> Vec<EbpfEvent>;
}

/// Process-specific monitoring
pub trait ProcessMonitor: Send + Sync {
    /// Start monitoring a specific process
    fn monitor_process(&self, pid: Pid) -> EbpfResult<()>;

    /// Stop monitoring a process
    fn unmonitor_process(&self, pid: Pid) -> EbpfResult<()>;

    /// Get monitored processes
    fn get_monitored_pids(&self) -> Vec<Pid>;

    /// Get syscall count for a process
    fn get_syscall_count(&self, pid: Pid) -> u64;

    /// Get network activity for a process
    fn get_network_activity(&self, pid: Pid) -> Vec<NetworkEvent>;

    /// Get file activity for a process
    fn get_file_activity(&self, pid: Pid) -> Vec<FileEvent>;
}

/// Complete eBPF manager trait combining all capabilities
pub trait EbpfManager:
    EbpfProvider + SyscallFilterProvider + EventMonitor + ProcessMonitor + Clone + Send + Sync
{
    /// Initialize the eBPF subsystem
    fn init(&self) -> EbpfResult<()>;

    /// Shutdown the eBPF subsystem
    fn shutdown(&self) -> EbpfResult<()>;

    /// Health check
    fn health_check(&self) -> bool;
}
