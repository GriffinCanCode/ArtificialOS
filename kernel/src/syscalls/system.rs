/*!
 * System Info Syscalls
 * System information and environment operations
 */

use log::{error, info};

use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::{SyscallResult, SystemInfo};

impl SyscallExecutor {
    pub(super) fn get_system_info(&self, pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        let info = SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            family: std::env::consts::FAMILY.to_string(),
        };

        info!("PID {} retrieved system info", pid);
        match serde_json::to_vec(&info) {
            Ok(data) => SyscallResult::success_with_data(data),
            Err(e) => {
                error!("Failed to serialize system info: {}", e);
                SyscallResult::error(format!("Failed to serialize system info: {}", e))
            }
        }
    }

    pub(super) fn get_current_time(&self, pid: u32) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::TimeAccess)
        {
            return SyscallResult::permission_denied("Missing TimeAccess capability");
        }

        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => {
                let timestamp = duration.as_secs();
                info!("PID {} retrieved current time: {}", pid, timestamp);
                let data = timestamp.to_le_bytes().to_vec();
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("System time error: {}", e);
                SyscallResult::error(format!("Failed to get system time: {}", e))
            }
        }
    }

    pub(super) fn get_env_var(&self, pid: u32, key: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        match std::env::var(key) {
            Ok(value) => {
                info!("PID {} read env var: {} = {}", pid, key, value);
                SyscallResult::success_with_data(value.into_bytes())
            }
            Err(_) => SyscallResult::error(format!("Environment variable not found: {}", key)),
        }
    }

    pub(super) fn set_env_var(&self, pid: u32, key: &str, value: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::SystemInfo)
        {
            return SyscallResult::permission_denied("Missing SystemInfo capability");
        }

        std::env::set_var(key, value);
        info!("PID {} set env var: {} = {}", pid, key, value);
        SyscallResult::success()
    }

    pub(super) fn network_request(&self, pid: u32, _url: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        log::warn!("Network operations not yet implemented");
        SyscallResult::error("Network operations not implemented")
    }
}
