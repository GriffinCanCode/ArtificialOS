/*!
 * Permission Types
 * Core types for centralized permission checking
 */

use crate::core::types::Pid;
use crate::security::types::Capability;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use thiserror::Error;

/// Result type for permission operations
pub type PermissionResult<T> = Result<T, PermissionError>;

/// Permission errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum PermissionError {
    #[error("Permission denied: {reason}")]
    Denied { reason: String },

    #[error("Invalid request: {reason}")]
    InvalidRequest { reason: String },

    #[error("Context unavailable: {reason}")]
    ContextUnavailable { reason: String },
}

/// Resource type being accessed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "resource")]
pub enum Resource {
    /// File system path
    File(PathBuf),
    /// Directory path
    Directory(PathBuf),
    /// Network host/port
    Network { host: String, port: Option<u16> },
    /// IPC channel
    IpcChannel(u32),
    /// Process
    Process(Pid),
    /// System resource
    System(String),
}

/// Action being performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Read,
    Write,
    Create,
    Delete,
    Execute,
    List,
    Connect,
    Bind,
    Send,
    Receive,
    Kill,
    Inspect,
}

/// Permission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// Process making the request
    pub pid: Pid,
    /// Resource being accessed
    pub resource: Resource,
    /// Action being performed
    pub action: Action,
    /// When the request was made
    pub timestamp: SystemTime,
}

impl PermissionRequest {
    /// Create a new permission request
    pub fn new(pid: Pid, resource: Resource, action: Action) -> Self {
        Self {
            pid,
            resource,
            action,
            timestamp: SystemTime::now(),
        }
    }

    /// File read request
    pub fn file_read(pid: Pid, path: PathBuf) -> Self {
        Self::new(pid, Resource::File(path), Action::Read)
    }

    /// File write request
    pub fn file_write(pid: Pid, path: PathBuf) -> Self {
        Self::new(pid, Resource::File(path), Action::Write)
    }

    /// File create request
    pub fn file_create(pid: Pid, path: PathBuf) -> Self {
        Self::new(pid, Resource::File(path), Action::Create)
    }

    /// File delete request
    pub fn file_delete(pid: Pid, path: PathBuf) -> Self {
        Self::new(pid, Resource::File(path), Action::Delete)
    }

    /// Directory list request
    pub fn dir_list(pid: Pid, path: PathBuf) -> Self {
        Self::new(pid, Resource::Directory(path), Action::List)
    }

    /// Network connect request
    pub fn net_connect(pid: Pid, host: String, port: Option<u16>) -> Self {
        Self::new(pid, Resource::Network { host, port }, Action::Connect)
    }

    /// Process kill request
    pub fn proc_kill(pid: Pid, target: Pid) -> Self {
        Self::new(pid, Resource::Process(target), Action::Kill)
    }

    /// Convert to capability for backward compatibility
    pub fn to_capability(&self) -> Option<Capability> {
        match (&self.resource, self.action) {
            (Resource::File(path), Action::Read) => {
                Some(Capability::ReadFile(Some(path.clone())))
            }
            (Resource::File(path), Action::Write) => {
                Some(Capability::WriteFile(Some(path.clone())))
            }
            (Resource::File(path), Action::Create) => {
                Some(Capability::CreateFile(Some(path.clone())))
            }
            (Resource::File(path), Action::Delete) => {
                Some(Capability::DeleteFile(Some(path.clone())))
            }
            (Resource::Directory(path), Action::List) => {
                Some(Capability::ListDirectory(Some(path.clone())))
            }
            (Resource::Process(_), Action::Kill) => Some(Capability::KillProcess),
            (Resource::Process(_), Action::Create) => Some(Capability::SpawnProcess),
            (Resource::System(_), Action::Inspect) => Some(Capability::SystemInfo),
            _ => None,
        }
    }
}

/// Permission response/decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    /// Request that was evaluated
    pub request: PermissionRequest,
    /// Whether permission is granted
    pub allowed: bool,
    /// Reason for decision
    pub reason: String,
    /// Decision time
    pub decided_at: SystemTime,
    /// Whether result was cached
    pub cached: bool,
}

impl PermissionResponse {
    /// Create allowed response
    pub fn allow(request: PermissionRequest, reason: impl Into<String>) -> Self {
        Self {
            request,
            allowed: true,
            reason: reason.into(),
            decided_at: SystemTime::now(),
            cached: false,
        }
    }

    /// Create denied response
    pub fn deny(request: PermissionRequest, reason: impl Into<String>) -> Self {
        Self {
            request,
            allowed: false,
            reason: reason.into(),
            decided_at: SystemTime::now(),
            cached: false,
        }
    }

    /// Mark as cached
    pub fn with_cached(mut self, cached: bool) -> Self {
        self.cached = cached;
        self
    }

    /// Check if allowed
    pub fn is_allowed(&self) -> bool {
        self.allowed
    }

    /// Get reason
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Resource type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    FileSystem,
    Network,
    Ipc,
    Process,
    System,
}

impl Resource {
    /// Get resource type
    pub fn resource_type(&self) -> ResourceType {
        match self {
            Resource::File(_) | Resource::Directory(_) => ResourceType::FileSystem,
            Resource::Network { .. } => ResourceType::Network,
            Resource::IpcChannel(_) => ResourceType::Ipc,
            Resource::Process(_) => ResourceType::Process,
            Resource::System(_) => ResourceType::System,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let req = PermissionRequest::file_read(100, PathBuf::from("/tmp/test.txt"));
        assert_eq!(req.pid, 100);
        assert_eq!(req.action, Action::Read);
        assert!(matches!(req.resource, Resource::File(_)));
    }

    #[test]
    fn test_response_creation() {
        let req = PermissionRequest::file_read(100, PathBuf::from("/tmp/test.txt"));
        let resp = PermissionResponse::allow(req, "Has capability");
        assert!(resp.is_allowed());
        assert_eq!(resp.reason(), "Has capability");
    }

    #[test]
    fn test_to_capability() {
        let req = PermissionRequest::file_read(100, PathBuf::from("/tmp/test.txt"));
        let cap = req.to_capability();
        assert!(cap.is_some());
        assert!(matches!(cap.unwrap(), Capability::ReadFile(_)));
    }
}
