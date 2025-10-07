/*!
 * Network Namespace Traits
 * Platform-agnostic abstractions for network isolation
 */

use super::types::*;
use crate::core::types::Pid;

/// Core namespace management operations
pub trait NamespaceProvider: Send + Sync {
    /// Create a new network namespace
    fn create(&self, config: NamespaceConfig) -> NamespaceResult<()>;

    /// Destroy a network namespace
    fn destroy(&self, id: &NamespaceId) -> NamespaceResult<()>;

    /// Check if a namespace exists
    fn exists(&self, id: &NamespaceId) -> bool;

    /// Get namespace information
    fn get_info(&self, id: &NamespaceId) -> Option<NamespaceInfo>;

    /// List all namespaces
    fn list(&self) -> Vec<NamespaceInfo>;

    /// Get namespace for a process
    fn get_by_pid(&self, pid: Pid) -> Option<NamespaceInfo>;

    /// Get statistics for a namespace
    fn get_stats(&self, id: &NamespaceId) -> Option<NamespaceStats>;

    /// Check if this implementation is supported on current platform
    fn is_supported(&self) -> bool;

    /// Get the platform type
    fn platform(&self) -> PlatformType;
}

/// Network interface management
pub trait InterfaceManager: Send + Sync {
    /// Create a virtual interface in a namespace
    fn create_interface(
        &self,
        ns_id: &NamespaceId,
        config: &InterfaceConfig,
    ) -> NamespaceResult<()>;

    /// Delete a virtual interface
    fn delete_interface(&self, ns_id: &NamespaceId, iface_name: &str) -> NamespaceResult<()>;

    /// Configure interface IP address
    fn set_ip_address(
        &self,
        ns_id: &NamespaceId,
        iface_name: &str,
        ip: IpAddr,
        prefix_len: u8,
    ) -> NamespaceResult<()>;

    /// Bring interface up/down
    fn set_interface_state(
        &self,
        ns_id: &NamespaceId,
        iface_name: &str,
        up: bool,
    ) -> NamespaceResult<()>;
}

/// Network routing and NAT
pub trait NetworkRouter: Send + Sync {
    /// Add a default route
    fn add_default_route(&self, ns_id: &NamespaceId, gateway: IpAddr) -> NamespaceResult<()>;

    /// Enable NAT for a namespace
    fn enable_nat(&self, ns_id: &NamespaceId) -> NamespaceResult<()>;

    /// Disable NAT for a namespace
    fn disable_nat(&self, ns_id: &NamespaceId) -> NamespaceResult<()>;

    /// Add port forwarding rule
    fn add_port_forward(
        &self,
        ns_id: &NamespaceId,
        host_port: u16,
        ns_port: u16,
    ) -> NamespaceResult<()>;

    /// Remove port forwarding rule
    fn remove_port_forward(&self, ns_id: &NamespaceId, host_port: u16) -> NamespaceResult<()>;
}

/// Process namespace attachment
pub trait ProcessAttacher: Send + Sync {
    /// Attach a process to a namespace
    fn attach_process(&self, ns_id: &NamespaceId, pid: Pid) -> NamespaceResult<()>;

    /// Detach a process from a namespace
    fn detach_process(&self, ns_id: &NamespaceId, pid: Pid) -> NamespaceResult<()>;

    /// Execute a command in a namespace
    fn exec_in_namespace(
        &self,
        ns_id: &NamespaceId,
        command: &str,
        args: &[String],
    ) -> NamespaceResult<u32>;
}
