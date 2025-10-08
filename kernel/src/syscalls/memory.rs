/*!

* Memory Syscalls
* Memory management and garbage collection
*/

use crate::core::serialization::json;
use crate::core::types::Pid;
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};

use log::{error, info, warn};

use crate::memory::types::ProcessMemoryStats;

use super::executor::SyscallExecutorWithIpc;
use super::types::SyscallResult;

impl SyscallExecutorWithIpc {
    pub(super) fn get_memory_stats(&self, pid: Pid) -> SyscallResult {
        // NOTE: SyscallGuard not needed here - executor already provides comprehensive
        // syscall tracing via span_syscall() and collector.syscall_exit()

        // Check permission using centralized manager
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "memory".to_string(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let memory_manager = match &self.optional.memory_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Memory manager not available"),
        };

        let stats = memory_manager.stats();
        match json::to_vec(&stats) {
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
        // Check permission using centralized manager
        let request =
            PermissionRequest::new(pid, Resource::Process { pid: target_pid }, Action::Inspect);
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let memory_manager = match &self.optional.memory_manager {
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

        match json::to_vec(&stats) {
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
        // Check permission using centralized manager
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "gc".to_string(),
            },
            Action::Execute,
        );
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let memory_manager = match &self.optional.memory_manager {
            Some(mm) => mm,
            None => return SyscallResult::error("Memory manager not available"),
        };

        let result = match target_pid {
            Some(target) => {
                // Targeted GC for specific process
                let freed = memory_manager.free_process_memory(target);
                info!(
                    "PID {} triggered targeted GC for PID {}, freed {} bytes",
                    pid, target, freed
                );

                match json::to_vec(&serde_json::json!({
                    "freed_bytes": freed,
                    "target_pid": target
                })) {
                    Ok(data) => Ok(SyscallResult::success_with_data(data)),
                    Err(e) => {
                        warn!("Failed to serialize GC result: {}", e);
                        Err("Internal serialization error".to_string())
                    }
                }
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

                match json::to_vec(&serde_json::json!({
                    "freed_bytes": stats.freed_bytes,
                    "freed_blocks": stats.freed_blocks,
                    "processes_cleaned": stats.processes_cleaned,
                    "duration_ms": stats.duration_ms
                })) {
                    Ok(data) => Ok(SyscallResult::success_with_data(data)),
                    Err(e) => {
                        warn!("Failed to serialize global GC result: {}", e);
                        Err("Internal serialization error".to_string())
                    }
                }
            }
        };

        result.unwrap_or_else(|e| SyscallResult::error(e))
    }
}
