/*!
 * Memory Syscalls
 * Memory management and garbage collection
 */

use log::{error, info};

use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::{ProcessMemoryStats, SyscallResult};

impl SyscallExecutor {
    pub(super) fn get_memory_stats(&self, pid: u32) -> SyscallResult {
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

        let stats = memory_manager.get_detailed_stats();
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

    pub(super) fn get_process_memory_stats(&self, pid: u32, target_pid: u32) -> SyscallResult {
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

        let memory_used = memory_manager.get_process_memory(target_pid);
        let stats = ProcessMemoryStats {
            pid: target_pid,
            bytes_allocated: memory_used,
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

    pub(super) fn trigger_gc(&self, pid: u32, target_pid: Option<u32>) -> SyscallResult {
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
                let freed = memory_manager.free_process_memory(target);
                info!("PID {} triggered GC for PID {}, freed {} bytes", pid, target, freed);
                let data = freed.to_le_bytes().to_vec();
                SyscallResult::success_with_data(data)
            }
            None => {
                // Global GC would be more complex in a real implementation
                info!("PID {} triggered global GC", pid);
                SyscallResult::success()
            }
        }
    }
}
