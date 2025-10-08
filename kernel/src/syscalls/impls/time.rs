/*!

* Time Syscalls
* Time and sleep operations
*/

use crate::core::types::Pid;
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};

use log::info;
use std::time::Duration;

use crate::syscalls::core::executor::{SyscallExecutorWithIpc, SYSTEM_START};
use crate::syscalls::types::SyscallResult;

impl SyscallExecutorWithIpc {
    pub(in crate::syscalls) fn sleep(&self, pid: Pid, duration_ms: u64) -> SyscallResult {
        // Check permission using centralized manager
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "time".into(),
            },
            Action::Read,
        );
        let response = self.permission_manager().check(&request);

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

    pub(in crate::syscalls) fn get_uptime(&self, pid: Pid) -> SyscallResult {
        // Check permission using centralized manager
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "time".into(),
            },
            Action::Read,
        );
        let response = self.permission_manager().check(&request);

        if !response.is_allowed() {
            return SyscallResult::permission_denied(response.reason());
        }

        let start = SYSTEM_START.get().ok_or_else(|| {
            log::error!("System start time not initialized");
            "System start time not initialized"
        });

        match start {
            Ok(start_time) => {
                let uptime = start_time.elapsed().as_secs();
                info!("PID {} retrieved system uptime: {} seconds", pid, uptime);
                use crate::core::PooledBuffer;
                let bytes = uptime.to_le_bytes();
                let mut buf = PooledBuffer::small();
                buf.extend_from_slice(&bytes);
                SyscallResult::success_with_data(buf.into_vec())
            }
            Err(e) => SyscallResult::error(e),
        }
    }
}
