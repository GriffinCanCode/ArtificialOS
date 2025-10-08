/*!

* System Info Syscalls
* System information and environment operations
*/

use crate::core::json;
use crate::core::types::Pid;
use crate::monitoring::span_operation;
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};

use log::{error, info, trace, warn};

use super::executor::SyscallExecutor;
use super::types::{SyscallResult, SystemInfo};

impl SyscallExecutor {
    pub(super) fn get_system_info(&self, pid: Pid) -> SyscallResult {
        let span = span_operation("get_system_info");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));

        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "info".to_string(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        let info = SystemInfo::current();
        trace!("System info: os={}, arch={}, family={}", info.os, info.arch, info.family);

        info!("PID {} retrieved system info", pid);
        span.record_result(true);
        match json::to_vec(&info) {
            Ok(data) => SyscallResult::success_with_data(data),
            Err(e) => {
                error!("Failed to serialize system info: {}", e);
                span.record_error("Serialization failed");
                SyscallResult::error(format!("Failed to serialize system info: {}", e))
            }
        }
    }

    pub(super) fn get_current_time(&self, pid: Pid) -> SyscallResult {
        let span = span_operation("get_time");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));

        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "time".to_string(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => {
                let timestamp = duration.as_secs();
                info!("PID {} retrieved current time: {}", pid, timestamp);
                span.record("timestamp", &format!("{}", timestamp));
                span.record_result(true);
                let data = timestamp.to_le_bytes().to_vec();
                SyscallResult::success_with_data(data)
            }
            Err(e) => {
                error!("System time error: {}", e);
                span.record_error(&format!("Time error: {}", e));
                SyscallResult::error(format!("Failed to get system time: {}", e))
            }
        }
    }

    pub(super) fn get_env_var(&self, pid: Pid, key: &str) -> SyscallResult {
        let span = span_operation("env_get");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("key", key);

        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "env".to_string(),
            },
            Action::Read,
        );
        let response = self.permission_manager.check(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        match std::env::var(key) {
            Ok(value) => {
                info!("PID {} read env var: {} = {}", pid, key, value);
                span.record_result(true);
                SyscallResult::success_with_data(value.into_bytes())
            }
            Err(_) => {
                span.record_error(&format!("Environment variable not found: {}", key));
                SyscallResult::error(format!("Environment variable not found: {}", key))
            }
        }
    }

    pub(super) fn set_env_var(&self, pid: Pid, key: &str, value: &str) -> SyscallResult {
        let span = span_operation("env_set");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("key", key);

        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "env".to_string(),
            },
            Action::Write,
        );
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        std::env::set_var(key, value);
        info!("PID {} set env var: {} = {}", pid, key, value);
        span.record_result(true);
        SyscallResult::success()
    }

    pub(super) fn network_request(&self, pid: Pid, url: &str) -> SyscallResult {
        let span = span_operation("http_request");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("url", url);

        // Parse URL to extract host and use proper network permission check
        let host = url
            .split("://")
            .nth(1)
            .unwrap_or(url)
            .split('/')
            .next()
            .unwrap_or("unknown")
            .to_string();

        trace!("Extracted host from URL: {}", host);

        let request = PermissionRequest::net_connect(pid, host.clone(), None);
        let response = self.permission_manager.check_and_audit(&request);

        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        span.record("host", &host);

        // Use reqwest for robust HTTP/HTTPS support with timeouts, connection pooling,
        // redirect handling, and compression support
        match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("ai-os-kernel/0.1.0")
            .build()
        {
            Ok(client) => match client.get(url).send() {
                Ok(response) => {
                    let status = response.status();

                    if !status.is_success() {
                        warn!("PID {} received HTTP {} for {}", pid, status.as_u16(), url);
                        span.record("status_code", &format!("{}", status.as_u16()));
                    }

                    match response.bytes() {
                        Ok(body) => {
                            info!(
                                "PID {} fetched {} ({} bytes, status: {})",
                                pid,
                                url,
                                body.len(),
                                status.as_u16()
                            );
                            span.record("bytes_received", &format!("{}", body.len()));
                            span.record("status_code", &format!("{}", status.as_u16()));
                            span.record_result(true);
                            SyscallResult::success_with_data(body.to_vec())
                        }
                        Err(e) => {
                            error!("Failed to read response body from {}: {}", url, e);
                            span.record_error(&format!("Failed to read response body: {}", e));
                            SyscallResult::error(format!("Failed to read response body: {}", e))
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch {}: {}", url, e);
                    span.record_error(&format!("Network request failed: {}", e));
                    SyscallResult::error(format!("Network request failed: {}", e))
                }
            },
            Err(e) => {
                error!("Failed to create HTTP client: {}", e);
                span.record_error(&format!("Failed to create HTTP client: {}", e));
                SyscallResult::error(format!("Failed to create HTTP client: {}", e))
            }
        }
    }
}
