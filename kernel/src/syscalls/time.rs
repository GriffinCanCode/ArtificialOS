/*!
 * Time Syscalls
 * Time and sleep operations
 */

use log::info;
use std::time::Duration;

use crate::security::Capability;

use super::executor::{SyscallExecutor, SYSTEM_START};
use super::types::SyscallResult;

impl SyscallExecutor {
    pub(super) fn sleep(&self, pid: u32, duration_ms: u64) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::TimeAccess)
        {
            return SyscallResult::permission_denied("Missing TimeAccess capability");
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

    pub(super) fn get_uptime(&self, pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::TimeAccess)
        {
            return SyscallResult::permission_denied("Missing TimeAccess capability");
        }

        let start = SYSTEM_START.get().expect("System start time not initialized");
        let uptime = start.elapsed().as_secs();

        info!("PID {} retrieved system uptime: {} seconds", pid, uptime);
        let data = uptime.to_le_bytes().to_vec();
        SyscallResult::success_with_data(data)
    }
}
