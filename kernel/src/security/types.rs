/*!
 * Security Types
 * Common types for security and sandboxing
 */

use crate::core::serde::{is_empty_vec, is_none, is_zero_u64, is_zero_usize};
use crate::core::types::{Pid, ResourceLimits};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;

/// Security operation result
///
/// # Must Use
/// Security operations can fail and must be handled to prevent vulnerabilities
#[must_use = "security operations can fail and must be handled"]
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Sandbox operation result
///
/// # Must Use
/// Sandbox operations can fail and must be handled
#[must_use = "sandbox operations can fail and must be handled"]
pub type SandboxResult<T> = Result<T, SandboxError>;

/// Limits operation result
///
/// # Must Use
/// Resource limit operations can fail and must be handled
#[must_use = "limit operations can fail and must be handled"]
pub type LimitsResult<T> = Result<T, LimitsError>;

/// Unified security error type
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error", content = "details")]
pub enum SecurityError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Capability missing: {0}")]
    CapabilityMissing(String),

    #[error("Sandbox not found for PID {0}")]
    SandboxNotFound(Pid),

    #[error("Path access denied: {0}")]
    PathAccessDenied(String),

    #[error("Resource limit exceeded: {0}")]
    LimitExceeded(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Sandbox error: {0}")]
    Sandbox(#[from] SandboxError),

    #[error("Limits error: {0}")]
    Limits(#[from] LimitsError),
}

/// Sandbox-specific errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error", content = "details")]
pub enum SandboxError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Sandbox not found for PID {0}")]
    NotFound(Pid),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Capability {0:?} not granted")]
    MissingCapability(String),

    #[error("Path {0:?} not accessible")]
    PathBlocked(String),
}

/// Resource limits errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error", content = "details")]
pub enum LimitsError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid limit: {0}")]
    InvalidLimit(String),
}

// Allow conversion from std::io::Error
impl From<std::io::Error> for LimitsError {
    fn from(err: std::io::Error) -> Self {
        LimitsError::IoError(err.to_string())
    }
}

/// Network access rules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "rule")]
pub enum NetworkRule {
    /// Allow all network access
    AllowAll,
    /// Allow specific host (with optional port)
    AllowHost { host: String, port: Option<u16> },
    /// Allow CIDR block
    AllowCIDR(String),
    /// Block specific host
    BlockHost { host: String, port: Option<u16> },
}

/// Capabilities that can be granted to sandboxed processes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "capability", content = "scope")]
pub enum Capability {
    // File system - granular per-path
    ReadFile(Option<PathBuf>),      // None = all files
    WriteFile(Option<PathBuf>),     // None = all files
    CreateFile(Option<PathBuf>),    // None = anywhere
    DeleteFile(Option<PathBuf>),    // None = anywhere
    ListDirectory(Option<PathBuf>), // None = all dirs

    // Process
    SpawnProcess,
    KillProcess,

    // Network - with rules
    NetworkAccess(NetworkRule),
    BindPort(Option<u16>), // None = any port
    /// Network namespace isolation capability
    NetworkNamespace,

    // System
    SystemInfo,
    TimeAccess,

    // IPC
    SendMessage,
    ReceiveMessage,
}

impl Capability {
    /// Check if this capability matches or is more permissive than another
    #[inline]
    #[must_use]
    pub fn grants(&self, required: &Capability) -> bool {
        match (self, required) {
            (Capability::ReadFile(None), Capability::ReadFile(_)) => true,
            (Capability::ReadFile(Some(a)), Capability::ReadFile(Some(b))) => b.starts_with(a),
            (Capability::WriteFile(None), Capability::WriteFile(_)) => true,
            (Capability::WriteFile(Some(a)), Capability::WriteFile(Some(b))) => b.starts_with(a),
            (Capability::CreateFile(None), Capability::CreateFile(_)) => true,
            (Capability::CreateFile(Some(a)), Capability::CreateFile(Some(b))) => b.starts_with(a),
            (Capability::DeleteFile(None), Capability::DeleteFile(_)) => true,
            (Capability::DeleteFile(Some(a)), Capability::DeleteFile(Some(b))) => b.starts_with(a),
            (Capability::ListDirectory(None), Capability::ListDirectory(_)) => true,
            (Capability::ListDirectory(Some(a)), Capability::ListDirectory(Some(b))) => {
                b.starts_with(a)
            }
            (Capability::NetworkAccess(NetworkRule::AllowAll), Capability::NetworkAccess(_)) => {
                true
            }
            (Capability::BindPort(None), Capability::BindPort(_)) => true,
            (Capability::BindPort(Some(a)), Capability::BindPort(Some(b))) => a == b,
            (a, b) => a == b,
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Capability::ReadFile(p) => write!(f, "ReadFile({:?})", p),
            Capability::WriteFile(p) => write!(f, "WriteFile({:?})", p),
            Capability::CreateFile(p) => write!(f, "CreateFile({:?})", p),
            Capability::DeleteFile(p) => write!(f, "DeleteFile({:?})", p),
            Capability::ListDirectory(p) => write!(f, "ListDirectory({:?})", p),
            Capability::SpawnProcess => write!(f, "SpawnProcess"),
            Capability::KillProcess => write!(f, "KillProcess"),
            Capability::NetworkAccess(r) => write!(f, "NetworkAccess({:?})", r),
            Capability::BindPort(p) => write!(f, "BindPort({:?})", p),
            Capability::NetworkNamespace => write!(f, "NetworkNamespace"),
            Capability::SystemInfo => write!(f, "SystemInfo"),
            Capability::TimeAccess => write!(f, "TimeAccess"),
            Capability::SendMessage => write!(f, "SendMessage"),
            Capability::ReceiveMessage => write!(f, "ReceiveMessage"),
        }
    }
}

/// Sandbox configuration for a process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SandboxConfig {
    pub pid: Pid,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub capabilities: HashSet<Capability>,
    pub resource_limits: ResourceLimits,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub allowed_paths: Vec<PathBuf>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub blocked_paths: Vec<PathBuf>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub network_rules: Vec<NetworkRule>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub environment_vars: Vec<(String, String)>,
}

impl SandboxConfig {
    /// Create a minimal sandbox (most restrictive)
    #[must_use]
    pub fn minimal(pid: Pid) -> Self {
        let mut config = Self {
            pid,
            capabilities: HashSet::new(),
            resource_limits: ResourceLimits::minimal(),
            allowed_paths: vec![],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/bin"),
                PathBuf::from("/sbin"),
                PathBuf::from("/usr/bin"),
                PathBuf::from("/usr/sbin"),
            ],
            network_rules: vec![],
            environment_vars: vec![],
        };
        // Canonicalize all paths for security
        config.canonicalize_paths();
        config
    }

    /// Create a standard sandbox (balanced)
    #[must_use]
    pub fn standard(pid: Pid) -> Self {
        let mut capabilities = HashSet::new();
        capabilities.insert(Capability::ReadFile(None));
        capabilities.insert(Capability::WriteFile(None));
        capabilities.insert(Capability::SystemInfo);
        capabilities.insert(Capability::TimeAccess);

        let mut config = Self {
            pid,
            capabilities,
            resource_limits: ResourceLimits::default(),
            allowed_paths: vec![PathBuf::from("/tmp"), PathBuf::from("/var/tmp")],
            blocked_paths: vec![PathBuf::from("/etc/passwd"), PathBuf::from("/etc/shadow")],
            network_rules: vec![],
            environment_vars: vec![],
        };
        // Canonicalize all paths for security
        config.canonicalize_paths();
        config
    }

    /// Create a privileged sandbox (for trusted apps)
    #[must_use]
    pub fn privileged(pid: Pid) -> Self {
        let mut capabilities = HashSet::new();
        capabilities.insert(Capability::ReadFile(None));
        capabilities.insert(Capability::WriteFile(None));
        capabilities.insert(Capability::CreateFile(None));
        capabilities.insert(Capability::DeleteFile(None));
        capabilities.insert(Capability::ListDirectory(None));
        capabilities.insert(Capability::SpawnProcess);
        capabilities.insert(Capability::KillProcess);
        capabilities.insert(Capability::NetworkAccess(NetworkRule::AllowAll));
        capabilities.insert(Capability::SystemInfo);
        capabilities.insert(Capability::TimeAccess);
        capabilities.insert(Capability::SendMessage);
        capabilities.insert(Capability::ReceiveMessage);

        let mut config = Self {
            pid,
            capabilities,
            resource_limits: ResourceLimits {
                max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB
                max_cpu_time_ms: 300_000,                 // 5 minutes
                max_file_descriptors: 500,
                max_processes: 50,
                max_network_connections: 100,
            },
            allowed_paths: vec![PathBuf::from("/")],
            blocked_paths: vec![],
            network_rules: vec![NetworkRule::AllowAll],
            environment_vars: vec![],
        };
        // Canonicalize all paths for security
        config.canonicalize_paths();
        config
    }
}

/// Resource limits to enforce at OS level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Limits {
    #[serde(skip_serializing_if = "is_none")]
    pub memory_bytes: Option<u64>,
    #[serde(skip_serializing_if = "is_none")]
    pub cpu_shares: Option<u32>, // Linux: 1-10000, higher = more CPU
    #[serde(skip_serializing_if = "is_none")]
    pub max_pids: Option<u32>,
    #[serde(skip_serializing_if = "is_none")]
    pub max_open_files: Option<u32>,
}

impl Limits {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            memory_bytes: None,
            cpu_shares: None,
            max_pids: None,
            max_open_files: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    #[inline]
    #[must_use]
    pub fn with_cpu_shares(mut self, shares: u32) -> Self {
        self.cpu_shares = Some(shares);
        self
    }

    #[inline]
    #[must_use]
    pub fn with_max_pids(mut self, pids: u32) -> Self {
        self.max_pids = Some(pids);
        self
    }

    #[inline]
    #[must_use]
    pub fn with_max_open_files(mut self, files: u32) -> Self {
        self.max_open_files = Some(files);
        self
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            memory_bytes: Some(512 * 1024 * 1024), // 512 MB
            cpu_shares: Some(100),                 // Standard priority
            max_pids: Some(10),
            max_open_files: Some(1024),
        }
    }
}

/// Sandbox statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SandboxStats {
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub total_sandboxes: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub active_processes: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub permission_denials: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub capability_checks: u64,
}

/// Security audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum SecurityEvent {
    PermissionDenied {
        pid: Pid,
        capability: String,
        reason: String,
    },
    PathAccessDenied {
        pid: Pid,
        path: String,
        reason: String,
    },
    LimitExceeded {
        pid: Pid,
        limit_type: String,
        value: u64,
    },
    CapabilityGranted {
        pid: Pid,
        capability: String,
    },
    CapabilityRevoked {
        pid: Pid,
        capability: String,
    },
    SandboxCreated {
        pid: Pid,
        config_type: String,
    },
    SandboxDestroyed {
        pid: Pid,
    },
}
