/*!

* Time Syscalls
* Time and sleep operations
*/

use crate::core::types::Pid;
use crate::permissions::{PermissionChecker, PermissionRequest, Resource, Action};

use log::info;
use std::time::Duration;

use crate::security::Capability;

use super::executor::{SyscallExecutor, SYSTEM_START};
use super::types::SyscallResult;

impl SyscallExecutor {
    pub(super) fn sleep(&self, pid: Pid, duration_ms: u64) -> SyscallResult {
        // Check permission using centralized manager
        let request = PermissionRequest::new(pid, Resource::System("time".to_string()), Action::Read);
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        // Limit sleep duration to prevent DoS
        const MAX_SLEEP_MS: u64 = 60_000; // 1 minute
        if duration_ms > MAX_SLEEP_MS {
            return SyscallResult::error(format!(
                "Sleep duration exceeds maximum ({} ms)",
                MAX_SLEEP_MS
            ));
        }

        info!("PID {} sleeping for {} ms", pid, duration_ms);
        std::thread::sleep(Duration::from_millis(duration_ms));
        SyscallResult::success()
    }

    pub(super) fn get_uptime(&self, pid: Pid) -> SyscallResult {
        // Check permission using centralized manager
        let request = PermissionRequest::new(pid, Resource::System("time".to_string()), Action::Read);
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let start = SYSTEM_START
            .get()
            .expect("System start time not initialized");
        let uptime = start.elapsed().as_secs();

        info!("PID {} retrieved system uptime: {} seconds", pid, uptime);
        let data = uptime.to_le_bytes().to_vec();
        SyscallResult::success_with_data(data)
    }
}
