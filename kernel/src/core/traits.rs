/*!
 * Core Traits
 * Fundamental kernel abstractions
 */

use super::errors::*;
use super::types::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

/// Scheduler statistics (now serializable for monitoring)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct SchedulerStatistics {
    pub total_scheduled: u64,
    pub context_switches: u64,
    pub preemptions: u64,
    pub active_processes: usize,
}

impl Default for SchedulerStatistics {
    fn default() -> Self {
        Self {
            total_scheduled: 0,
            context_switches: 0,
            preemptions: 0,
            active_processes: 0,
        }
    }
}

impl SchedulerStatistics {
    /// Create new empty statistics
    pub const fn new() -> Self {
        Self {
            total_scheduled: 0,
            context_switches: 0,
            preemptions: 0,
            active_processes: 0,
        }
    }

    /// Calculate the average context switches per process
    pub fn avg_context_switches_per_process(&self) -> f64 {
        if self.active_processes == 0 {
            0.0
        } else {
            self.context_switches as f64 / self.active_processes as f64
        }
    }

    /// Calculate preemption ratio
    pub fn preemption_ratio(&self) -> f64 {
        if self.total_scheduled == 0 {
            0.0
        } else {
            self.preemptions as f64 / self.total_scheduled as f64
        }
    }
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

/// Binary serialization trait for internal IPC operations
///
/// Types that implement this trait can be efficiently serialized using bincode
/// for internal kernel-to-kernel communication, providing 5-10x better performance
/// than JSON for binary payloads.
///
/// This trait is separate from the standard Serialize/Deserialize traits to allow
/// types to choose different serialization strategies for internal vs external APIs:
/// - Use `BincodeSerializable` for internal kernel IPC (fast, compact binary)
/// - Use JSON for external APIs and debugging (human-readable)
pub trait BincodeSerializable: Serialize + DeserializeOwned + Send + Sync {
    /// Serialize to binary format using bincode
    fn to_bincode(&self) -> std::result::Result<Vec<u8>, String> {
        crate::core::serialization::bincode::to_vec(self)
            .map_err(|e| format!("Bincode serialization failed: {}", e))
    }

    /// Deserialize from binary format using bincode
    fn from_bincode(bytes: &[u8]) -> std::result::Result<Self, String>
    where
        Self: Sized,
    {
        crate::core::serialization::bincode::from_slice(bytes)
            .map_err(|e| format!("Bincode deserialization failed: {}", e))
    }

    /// Get the serialized size without actually serializing
    fn bincode_size(&self) -> std::result::Result<u64, String> {
        crate::core::serialization::bincode::serialized_size(self)
            .map_err(|e| format!("Failed to calculate bincode size: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_statistics_default() {
        let stats = SchedulerStatistics::default();
        assert_eq!(stats.total_scheduled, 0);
        assert_eq!(stats.active_processes, 0);
    }

    #[test]
    fn test_scheduler_statistics_calculations() {
        let stats = SchedulerStatistics {
            total_scheduled: 100,
            context_switches: 50,
            preemptions: 10,
            active_processes: 5,
        };

        assert_eq!(stats.avg_context_switches_per_process(), 10.0);
        assert_eq!(stats.preemption_ratio(), 0.1);
    }

    #[test]
    fn test_scheduler_statistics_edge_cases() {
        let empty_stats = SchedulerStatistics::new();
        assert_eq!(empty_stats.avg_context_switches_per_process(), 0.0);
        assert_eq!(empty_stats.preemption_ratio(), 0.0);
    }

    #[test]
    fn test_scheduler_statistics_serialization() {
        let stats = SchedulerStatistics {
            total_scheduled: 100,
            context_switches: 50,
            preemptions: 10,
            active_processes: 5,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: SchedulerStatistics = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
    }
}
