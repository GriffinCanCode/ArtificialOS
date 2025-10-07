/*!

* System Info Syscalls
* System information and environment operations
*/

use crate::core::types::Pid;

use log::{error, info};

use crate::security::Capability;

use super::executor::SyscallExecutor;
use super::types::{SyscallResult, SystemInfo};

impl SyscallExecutor {
    pub(super) fn get_system_info(&self, pid: Pid) -> SyscallResult {
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

    pub(super) fn get_current_time(&self, pid: Pid) -> SyscallResult {
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

    pub(super) fn get_env_var(&self, pid: Pid, key: &str) -> SyscallResult {
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

    pub(super) fn set_env_var(&self, pid: Pid, key: &str, value: &str) -> SyscallResult {
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

    pub(super) fn network_request(&self, pid: Pid, url: &str) -> SyscallResult {
        if !self
            .sandbox_manager
            .check_permission(pid, &Capability::NetworkAccess)
        {
            return SyscallResult::permission_denied("Missing NetworkAccess capability");
        }

        use std::io::{Read, Write};
        use std::net::TcpStream;

        // Parse URL to extract host and path
        let url_str = if url.starts_with("http://") {
            &url[7..]
        } else if url.starts_with("https://") {
            log::warn!("HTTPS not fully supported, attempting plain HTTP");
            &url[8..]
        } else {
            url
        };

        let (host, path) = match url_str.split_once('/') {
            Some((h, p)) => (h, format!("/{}", p)),
            None => (url_str, "/".to_string()),
        };

        // Extract port if specified
        let (host_name, port) = match host.split_once(':') {
            Some((h, p)) => (h, p.parse::<u16>().unwrap_or(80)),
            None => (host, 80),
        };

        let address = format!("{}:{}", host_name, port);

        // Make HTTP request
        match TcpStream::connect(&address) {
            Ok(mut stream) => {
                let request = format!(
                    "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                    path, host_name
                );

                if let Err(e) = stream.write_all(request.as_bytes()) {
                    log::error!("Failed to send HTTP request: {}", e);
                    return SyscallResult::error(format!("Failed to send request: {}", e));
                }

                let mut response = Vec::new();
                if let Err(e) = stream.read_to_end(&mut response) {
                    log::error!("Failed to read HTTP response: {}", e);
                    return SyscallResult::error(format!("Failed to read response: {}", e));
                }

                info!("PID {} fetched {} ({} bytes)", pid, url, response.len());
                SyscallResult::success_with_data(response)
            }
            Err(e) => {
                log::error!("Failed to connect to {}: {}", address, e);
                SyscallResult::error(format!("Network request failed: {}", e))
            }
        }
    }
}
