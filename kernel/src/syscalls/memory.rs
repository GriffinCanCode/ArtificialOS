/*!

* Memory Syscalls
* Memory management and garbage collection
*/

use crate::core::types::Pid;

use log::{error, info};

use crate::memory::types::ProcessMemoryStats;
use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::SyscallResult;

impl SyscallExecutor {
    pub(super) fn get_memory_stats(&self, pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let memory_manager = match &self.memory_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Memory manager not available"),
        };

        let stats = memory_manager.stats();
        match serde_json::to_vec(&stats) {
            Ok(data) => {
                info!("PID {} retrieved global memory stats", pid);
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Failed to serialize memory stats: {}", e);
                SyscallResult::error("Serialization failed")
            }
        }
    }

    pub(super) fn get_process_memory_stats(&self, pid: Pid, target_pid: Pid) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let memory_manager = match &self.memory_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Memory manager not available"),
        };

        let (allocated_bytes, peak_bytes, allocation_count) =
            memory_manager.get_process_memory_details(target_pid);

        let stats = ProcessMemoryStats {
            pid: target_pid,
            allocated_bytes,
            peak_bytes,
            allocation_count,
        };

        match serde_json::to_vec(&stats) {
            Ok(data) => {
                info!("PID {} retrieved memory stats for PID {}", pid, target_pid);
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("Failed to serialize process memory stats: {}", e);
                SyscallResult::error("Serialization failed")
            }
        }
    }

    pub(super) fn trigger_gc(&self, pid: Pid, target_pid: Option<u32>) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let memory_manager = match &self.memory_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Memory manager not available"),
        };

        match target_pid {
            Some(target) => {
                // Targeted GC for specific process
                let freed = memory_manager.free_process_memory(target);
                info!(
                    "PID {} triggered targeted GC for PID {}, freed {} bytes",
                    pid, target, freed
                );

                let data = serde_json::to_vec(&serde_json::json!({
                    "freed_bytes": freed,
                    "target_pid": target
                }))
                .unwrap();

                SyscallResult::success_with_data(data)
            }
            None => {
                // Global GC - run comprehensive cleanup
                use crate::memory::{GcStrategy, GlobalGarbageCollector};

                info!("PID {} triggered global GC", pid);

                // Create global GC instance
                let gc = GlobalGarbageCollector::new(memory_manager.clone());

                // Run global collection
                let stats = gc.collect(GcStrategy::Global);

                info!(
                    "Global GC completed: freed {} bytes ({} blocks) from {} processes in {}ms",
                    stats.freed_bytes,
                    stats.freed_blocks,
                    stats.processes_cleaned,
                    stats.duration_ms
                );

                let data = serde_json::to_vec(&serde_json::json!({
                    "freed_bytes": stats.freed_bytes,
                    "freed_blocks": stats.freed_blocks,
                    "processes_cleaned": stats.processes_cleaned,
                    "duration_ms": stats.duration_ms
                }))
                .unwrap();

                SyscallResult::success_with_data(data)
            }
        }
    }
}
