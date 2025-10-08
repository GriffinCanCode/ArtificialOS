/*!
 * Network Namespace Types
 * Platform-agnostic types for network isolation
 */

use crate::core::types::Pid;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use thiserror::Error;

/// Result type for namespace operations
pub type NamespaceResult<T> = Result<T, NamespaceError>;

/// Network namespace errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error", content = "details")]
pub enum NamespaceError {
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Namespace not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Network operation failed: {0}")]
    NetworkError(String),

    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for NamespaceError {
    fn from(err: std::io::Error) -> Self {
        NamespaceError::IoError(err.to_string())
    }
}

/// Unique identifier for a network namespace
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespaceId(pub String);

impl NamespaceId {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn from_pid(pid: Pid) -> Self {
        Self(format!("ns-{}", pid))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NamespaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Network isolation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationMode {
    /// Full network isolation (no external access)
    Full,
    /// Private network with NAT to host
    Private,
    /// Shared host network
    Shared,
    /// Custom bridge network
    Bridged,
}

/// Virtual network interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    /// Interface name
    pub name: String,
    /// IP address and prefix length
    pub ip_addr: IpAddr,
    /// Subnet prefix length (e.g., 24 for /24)
    pub prefix_len: u8,
    /// Gateway IP (optional)
    pub gateway: Option<IpAddr>,
    /// MTU size
    pub mtu: u32,
}

impl Default for InterfaceConfig {
    fn default() -> Self {
        Self {
            name: "veth0".to_string(),
            ip_addr: IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 2).into()),
            prefix_len: 24,
            gateway: Some(IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1).into())),
            mtu: 1500,
        }
    }
}

/// Network namespace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceConfig {
    /// Unique namespace identifier
    pub id: NamespaceId,
    /// Process ID that owns this namespace
    pub pid: Pid,
    /// Isolation mode
    pub mode: IsolationMode,
    /// Interface configuration
    pub interface: Option<InterfaceConfig>,
    /// DNS servers
    pub dns_servers: Vec<IpAddr>,
    /// Enable IPv6
    pub enable_ipv6: bool,
    /// Port forwarding rules (host_port -> namespace_port)
    pub port_forwards: Vec<(u16, u16)>,
}

impl NamespaceConfig {
    /// Create a fully isolated namespace
    pub fn full_isolation(pid: Pid) -> Self {
        Self {
            id: NamespaceId::from_pid(pid),
            pid,
            mode: IsolationMode::Full,
            interface: None,
            dns_servers: vec![],
            enable_ipv6: false,
            port_forwards: vec![],
        }
    }

    /// Create a private network with NAT
    pub fn private_network(pid: Pid) -> Self {
        Self {
            id: NamespaceId::from_pid(pid),
            pid,
            mode: IsolationMode::Private,
            interface: Some(InterfaceConfig::default().into()),
            dns_servers: vec![
                IpAddr::V4(std::net::Ipv4Addr::new(8, 8, 8, 8).into()),
                IpAddr::V4(std::net::Ipv4Addr::new(8, 8, 4, 4).into()),
            ],
            enable_ipv6: true,
            port_forwards: vec![],
        }
    }

    /// Create a shared network (host namespace)
    pub fn shared_network(pid: Pid) -> Self {
        Self {
            id: NamespaceId::from_pid(pid),
            pid,
            mode: IsolationMode::Shared,
            interface: None,
            dns_servers: vec![],
            enable_ipv6: true,
            port_forwards: vec![],
        }
    }
}

/// Network namespace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceStats {
    /// Namespace ID
    pub id: NamespaceId,
    /// Number of active interfaces
    pub interface_count: usize,
    /// Total bytes sent
    pub tx_bytes: u64,
    /// Total bytes received
    pub rx_bytes: u64,
    /// Total packets sent
    pub tx_packets: u64,
    /// Total packets received
    pub rx_packets: u64,
    /// Timestamp of creation
    pub created_at: std::time::SystemTime,
}

/// Network namespace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceInfo {
    /// Configuration
    pub config: NamespaceConfig,
    /// Statistics
    pub stats: Option<NamespaceStats>,
    /// Platform-specific implementation type
    pub platform: PlatformType,
}

/// Platform implementation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformType {
    /// Linux network namespaces
    LinuxNetns,
    /// macOS network filters
    MacOSFilter,
    /// Windows Filtering Platform
    WindowsWFP,
    /// Simulation mode (capability-based only)
    Simulation,
}
