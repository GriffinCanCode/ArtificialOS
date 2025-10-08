/*!
 * Clipboard Syscalls
 * Implementation of clipboard operations
 */

use crate::core::clipboard::{
    ClipboardData, ClipboardError, ClipboardFormat, ClipboardManager,
};
use crate::core::serialization::json;
use crate::core::types::Pid;
use crate::monitoring::span_operation;
use crate::permissions::{Action, PermissionChecker, PermissionRequest, Resource};
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::types::SyscallResult;
use log::{debug, error, info};

impl SyscallExecutorWithIpc {
    /// Get clipboard manager
    pub(crate) fn clipboard(&self) -> &ClipboardManager {
        self.clipboard_manager()
    }

    pub(in crate::syscalls) fn clipboard_copy(
        &self,
        pid: Pid,
        data: &[u8],
        format: &str,
        global: bool,
    ) -> SyscallResult {
        let span = span_operation("clipboard_copy");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("format", format);
        span.record("global", &format!("{}", global));

        // Permission check
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "clipboard".into(),
            },
            Action::Write,
        );
        let response = self.permission_manager().check(&request);
        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Parse format and create clipboard data
        let clipboard_data = match Self::parse_clipboard_data(data, format) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to parse clipboard data: {}", e);
                span.record_error(&format!("Parse error: {}", e));
                return SyscallResult::error(format!("Invalid clipboard data: {}", e));
            }
        };

        // Copy to clipboard
        let result = if global {
            self.clipboard().copy_global(pid, clipboard_data)
        } else {
            self.clipboard().copy(pid, clipboard_data)
        };

        match result {
            Ok(entry_id) => {
                info!(
                    "PID {} copied to {} clipboard: entry {}",
                    pid,
                    if global { "global" } else { "process" },
                    entry_id
                );
                span.record("entry_id", &format!("{}", entry_id));
                span.record_result(true);
                match json::to_vec(&entry_id) {
                    Ok(bytes) => SyscallResult::success_with_data(bytes),
                    Err(e) => {
                        error!("Failed to serialize entry ID: {}", e);
                        SyscallResult::error(format!("Serialization error: {}", e))
                    }
                }
            }
            Err(e) => {
                error!("Clipboard copy failed: {}", e);
                span.record_error(&format!("{}", e));
                SyscallResult::error(format!("Clipboard copy failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn clipboard_paste(&self, pid: Pid, global: bool) -> SyscallResult {
        let span = span_operation("clipboard_paste");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("global", &format!("{}", global));

        // Permission check
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "clipboard".into(),
            },
            Action::Read,
        );
        let response = self.permission_manager().check(&request);
        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Paste from clipboard
        let result = if global {
            self.clipboard().paste_global()
        } else {
            self.clipboard().paste(pid)
        };

        match result {
            Ok(entry) => {
                debug!(
                    "PID {} pasted from {} clipboard: entry {}",
                    pid,
                    if global { "global" } else { "process" },
                    entry.id
                );
                span.record("entry_id", &format!("{}", entry.id));
                span.record_result(true);
                match json::to_vec(&entry) {
                    Ok(bytes) => SyscallResult::success_with_data(bytes),
                    Err(e) => {
                        error!("Failed to serialize clipboard entry: {}", e);
                        SyscallResult::error(format!("Serialization error: {}", e))
                    }
                }
            }
            Err(ClipboardError::Empty) => {
                span.record_error("Clipboard is empty");
                SyscallResult::error("Clipboard is empty".to_string())
            }
            Err(e) => {
                error!("Clipboard paste failed: {}", e);
                span.record_error(&format!("{}", e));
                SyscallResult::error(format!("Clipboard paste failed: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn clipboard_history(
        &self,
        pid: Pid,
        global: bool,
        limit: Option<usize>,
    ) -> SyscallResult {
        let span = span_operation("clipboard_history");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("global", &format!("{}", global));

        // Permission check
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "clipboard".into(),
            },
            Action::Read,
        );
        let response = self.permission_manager().check(&request);
        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        let history = if global {
            self.clipboard().history_global(limit)
        } else {
            self.clipboard().history(pid, limit)
        };

        debug!("PID {} retrieved clipboard history: {} entries", pid, history.len());
        span.record("count", &format!("{}", history.len()));
        span.record_result(true);

        match json::to_vec(&history) {
            Ok(bytes) => SyscallResult::success_with_data(bytes),
            Err(e) => {
                error!("Failed to serialize clipboard history: {}", e);
                SyscallResult::error(format!("Serialization error: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn clipboard_get_entry(
        &self,
        pid: Pid,
        entry_id: u64,
    ) -> SyscallResult {
        let span = span_operation("clipboard_get_entry");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("entry_id", &format!("{}", entry_id));

        // Permission check
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "clipboard".into(),
            },
            Action::Read,
        );
        let response = self.permission_manager().check(&request);
        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        match self.clipboard().get_entry(pid, entry_id) {
            Ok(entry) => {
                debug!("PID {} retrieved clipboard entry {}", pid, entry_id);
                span.record_result(true);
                match json::to_vec(&entry) {
                    Ok(bytes) => SyscallResult::success_with_data(bytes),
                    Err(e) => {
                        error!("Failed to serialize clipboard entry: {}", e);
                        SyscallResult::error(format!("Serialization error: {}", e))
                    }
                }
            }
            Err(e) => {
                error!("Failed to get clipboard entry: {}", e);
                span.record_error(&format!("{}", e));
                SyscallResult::error(format!("Entry not found: {}", e))
            }
        }
    }

    pub(in crate::syscalls) fn clipboard_clear(&self, pid: Pid, global: bool) -> SyscallResult {
        let span = span_operation("clipboard_clear");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));
        span.record("global", &format!("{}", global));

        // Permission check
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "clipboard".into(),
            },
            Action::Write,
        );
        let response = self.permission_manager().check(&request);
        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        if global {
            self.clipboard().clear_global();
        } else {
            self.clipboard().clear(pid);
        }

        info!(
            "PID {} cleared {} clipboard",
            pid,
            if global { "global" } else { "process" }
        );
        span.record_result(true);
        SyscallResult::success()
    }

    pub(in crate::syscalls) fn clipboard_subscribe(
        &self,
        pid: Pid,
        formats: Vec<String>,
    ) -> SyscallResult {
        let span = span_operation("clipboard_subscribe");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));

        // Permission check
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "clipboard".into(),
            },
            Action::Read,
        );
        let response = self.permission_manager().check(&request);
        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        // Parse formats
        let parsed_formats: Vec<ClipboardFormat> = formats
            .iter()
            .filter_map(|f| Self::parse_clipboard_format(f))
            .collect();

        self.clipboard().subscribe(pid, parsed_formats);
        info!("PID {} subscribed to clipboard changes", pid);
        span.record_result(true);
        SyscallResult::success()
    }

    pub(in crate::syscalls) fn clipboard_unsubscribe(&self, pid: Pid) -> SyscallResult {
        let span = span_operation("clipboard_unsubscribe");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));

        self.clipboard().unsubscribe(pid);
        info!("PID {} unsubscribed from clipboard changes", pid);
        span.record_result(true);
        SyscallResult::success()
    }

    pub(in crate::syscalls) fn clipboard_stats(&self, pid: Pid) -> SyscallResult {
        let span = span_operation("clipboard_stats");
        let _guard = span.enter();
        span.record("pid", &format!("{}", pid));

        // Permission check
        let request = PermissionRequest::new(
            pid,
            Resource::System {
                name: "clipboard".into(),
            },
            Action::Inspect,
        );
        let response = self.permission_manager().check(&request);
        if !response.is_allowed() {
            span.record_error(response.reason());
            return SyscallResult::permission_denied(response.reason());
        }

        let stats = self.clipboard().stats();
        debug!("PID {} retrieved clipboard stats", pid);
        span.record_result(true);

        match json::to_vec(&stats) {
            Ok(bytes) => SyscallResult::success_with_data(bytes),
            Err(e) => {
                error!("Failed to serialize clipboard stats: {}", e);
                SyscallResult::error(format!("Serialization error: {}", e))
            }
        }
    }

    /// Parse clipboard data from bytes and format
    fn parse_clipboard_data(data: &[u8], format: &str) -> Result<ClipboardData, String> {
        match format {
            "text" | "text/plain" => {
                let text =
                    String::from_utf8(data.to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e))?;
                Ok(ClipboardData::Text(text))
            }
            "html" | "text/html" => {
                let html =
                    String::from_utf8(data.to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e))?;
                Ok(ClipboardData::Html(html))
            }
            "bytes" | "application/octet-stream" => Ok(ClipboardData::Bytes(data.to_vec())),
            format if format.starts_with("image/") => Ok(ClipboardData::Image {
                data: data.to_vec(),
                mime_type: format.to_string(),
            }),
            _ => Err(format!("Unsupported format: {}", format)),
        }
    }

    /// Parse clipboard format from string
    fn parse_clipboard_format(format: &str) -> Option<ClipboardFormat> {
        match format {
            "text" | "text/plain" => Some(ClipboardFormat::Text),
            "html" | "text/html" => Some(ClipboardFormat::Html),
            "bytes" | "application/octet-stream" => Some(ClipboardFormat::Bytes),
            "files" | "text/uri-list" => Some(ClipboardFormat::Files),
            format if format.starts_with("image/") => Some(ClipboardFormat::Image {
                mime_type: format.to_string(),
            }),
            _ => Some(ClipboardFormat::Custom {
                mime_type: format.to_string(),
            }),
        }
    }
}

