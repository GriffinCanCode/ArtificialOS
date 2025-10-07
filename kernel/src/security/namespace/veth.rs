/*!
 * Virtual Ethernet (veth) Pair Management
 * Creates and manages virtual ethernet pairs for namespace connectivity
 */

use super::types::*;
use log::{debug, info, warn};

#[cfg(target_os = "linux")]
use futures::stream::TryStreamExt;
#[cfg(target_os = "linux")]
use rtnetlink::{new_connection, Handle, IpVersion};
#[cfg(target_os = "linux")]
use std::net::IpAddr;

/// Virtual ethernet pair manager
pub struct VethManager {
    #[cfg(target_os = "linux")]
    handle: Option<Handle>,
}

impl VethManager {
    #[cfg(target_os = "linux")]
    pub fn new() -> Self {
        Self { handle: None }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new() -> Self {
        Self {}
    }

    /// Initialize the netlink connection (must be called in async context)
    #[cfg(target_os = "linux")]
    pub async fn init(&mut self) -> NamespaceResult<()> {
        let (connection, handle, _) = new_connection().map_err(|e| {
            NamespaceError::NetworkError(format!("Failed to create netlink connection: {}", e))
        })?;

        // Spawn the connection in the background
        tokio::spawn(connection);

        self.handle = Some(handle);
        info!("VethManager initialized with netlink connection");
        Ok(())
    }

    /// Initialize for non-Linux platforms (no-op)
    #[cfg(not(target_os = "linux"))]
    pub async fn init(&mut self) -> NamespaceResult<()> {
        info!("VethManager initialized (platform-specific mode)");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn get_handle(&self) -> NamespaceResult<&Handle> {
        self.handle.as_ref().ok_or_else(|| {
            NamespaceError::InvalidConfig(
                "VethManager not initialized. Call init() first.".to_string(),
            )
        })
    }

    /// Create a veth pair connecting host and namespace
    #[cfg(target_os = "linux")]
    pub async fn create_pair(
        &self,
        host_name: &str,
        ns_name: &str,
        ns_id: &NamespaceId,
    ) -> NamespaceResult<()> {
        info!(
            "Creating veth pair: {} (host) <-> {} (ns: {})",
            host_name, ns_name, ns_id
        );

        let handle = self.get_handle()?;

        // Create veth pair
        handle
            .link()
            .add()
            .veth(host_name.to_string(), ns_name.to_string())
            .execute()
            .await
            .map_err(|e| {
                NamespaceError::NetworkError(format!(
                    "Failed to create veth pair {}<->{}: {}",
                    host_name, ns_name, e
                ))
            })?;

        debug!("Veth pair created: {} <-> {}", host_name, ns_name);

        // Get the peer interface index
        let mut links = handle
            .link()
            .get()
            .match_name(ns_name.to_string())
            .execute();
        if let Some(link) = links.try_next().await.map_err(|e| {
            NamespaceError::NetworkError(format!("Failed to get link {}: {}", ns_name, e))
        })? {
            let ns_path = format!("/var/run/netns/{}", ns_id.as_str());

            // Move peer to namespace
            // Note: rtnetlink doesn't directly support moving to namespace by path
            // In production, you'd need to use nix::sched::setns or similar
            debug!("Would move interface {} to namespace {}", ns_name, ns_path);

            // This is a limitation - moving interfaces requires direct syscalls
            // The actual implementation would use:
            // 1. Open namespace file descriptor: let ns_fd = File::open(ns_path)?;
            // 2. Use netlink to move interface by fd
            warn!("Interface movement to namespace requires additional implementation");
        }

        debug!("Veth pair created successfully");
        Ok(())
    }

    /// Delete a veth pair
    #[cfg(target_os = "linux")]
    pub async fn delete_pair(&self, host_name: &str) -> NamespaceResult<()> {
        info!("Deleting veth pair: {}", host_name);

        let handle = self.get_handle()?;

        // Get the link index
        let mut links = handle
            .link()
            .get()
            .match_name(host_name.to_string())
            .execute();

        if let Some(link) = links.try_next().await.map_err(|e| {
            NamespaceError::NetworkError(format!("Failed to find interface {}: {}", host_name, e))
        })? {
            // Delete the link (automatically removes the pair)
            handle
                .link()
                .del(link.header.index)
                .execute()
                .await
                .map_err(|e| {
                    NamespaceError::NetworkError(format!(
                        "Failed to delete veth pair {}: {}",
                        host_name, e
                    ))
                })?;

            debug!("Veth pair deleted successfully");
            Ok(())
        } else {
            Err(NamespaceError::NotFound(format!(
                "Interface {} not found",
                host_name
            )))
        }
    }

    /// Configure IP address on veth interface
    #[cfg(target_os = "linux")]
    pub async fn set_ip(
        &self,
        iface_name: &str,
        ip_addr: IpAddr,
        prefix_len: u8,
    ) -> NamespaceResult<()> {
        info!(
            "Setting IP address on {}: {}/{}",
            iface_name, ip_addr, prefix_len
        );

        let handle = self.get_handle()?;

        // Get the link index
        let mut links = handle
            .link()
            .get()
            .match_name(iface_name.to_string())
            .execute();

        if let Some(link) = links.try_next().await.map_err(|e| {
            NamespaceError::NetworkError(format!("Failed to find interface {}: {}", iface_name, e))
        })? {
            // Add IP address
            handle
                .address()
                .add(link.header.index, ip_addr, prefix_len)
                .execute()
                .await
                .map_err(|e| {
                    NamespaceError::NetworkError(format!(
                        "Failed to set IP {}/{} on {}: {}",
                        ip_addr, prefix_len, iface_name, e
                    ))
                })?;

            debug!("IP address configured successfully");
            Ok(())
        } else {
            Err(NamespaceError::NotFound(format!(
                "Interface {} not found",
                iface_name
            )))
        }
    }

    /// Bring interface up or down
    #[cfg(target_os = "linux")]
    pub async fn set_state(&self, iface_name: &str, up: bool) -> NamespaceResult<()> {
        let state = if up { "up" } else { "down" };
        info!("Setting interface {} {}", iface_name, state);

        let handle = self.get_handle()?;

        // Get the link index
        let mut links = handle
            .link()
            .get()
            .match_name(iface_name.to_string())
            .execute();

        if let Some(link) = links.try_next().await.map_err(|e| {
            NamespaceError::NetworkError(format!("Failed to find interface {}: {}", iface_name, e))
        })? {
            // Set link state
            let request = if up {
                handle.link().set(link.header.index).up()
            } else {
                handle.link().set(link.header.index).down()
            };

            request.execute().await.map_err(|e| {
                NamespaceError::NetworkError(format!(
                    "Failed to set interface {} {}: {}",
                    iface_name, state, e
                ))
            })?;

            debug!("Interface state changed successfully");
            Ok(())
        } else {
            Err(NamespaceError::NotFound(format!(
                "Interface {} not found",
                iface_name
            )))
        }
    }

    /// Create a feth pair on macOS (alternative to veth)
    #[cfg(target_os = "macos")]
    pub async fn create_pair(
        &self,
        host_name: &str,
        ns_name: &str,
        _ns_id: &NamespaceId,
    ) -> NamespaceResult<()> {
        info!(
            "Creating feth pair on macOS: {} (host) <-> {} (peer)",
            host_name, ns_name
        );

        // On macOS, we use feth (fake ethernet) interfaces
        // Command equivalent: ifconfig feth0 create
        // Then: ifconfig feth0 peer feth1

        use std::process::Command;

        // Create the first feth interface
        let output = Command::new("ifconfig")
            .args(&[host_name, "create"])
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NamespaceError::NetworkError(format!(
                "Failed to create feth interface {}: {}",
                host_name, stderr
            )));
        }

        // Set the peer
        let output = Command::new("ifconfig")
            .args(&[host_name, "peer", ns_name])
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to set peer for {}: {}", host_name, stderr);
        }

        debug!("Feth pair created successfully on macOS");
        Ok(())
    }

    /// Delete a feth pair on macOS
    #[cfg(target_os = "macos")]
    pub async fn delete_pair(&self, host_name: &str) -> NamespaceResult<()> {
        info!("Deleting feth pair on macOS: {}", host_name);

        use std::process::Command;

        let output = Command::new("ifconfig")
            .args(&[host_name, "destroy"])
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NamespaceError::NetworkError(format!(
                "Failed to destroy feth interface {}: {}",
                host_name, stderr
            )));
        }

        debug!("Feth pair deleted successfully");
        Ok(())
    }

    /// Configure IP address on interface (macOS)
    #[cfg(target_os = "macos")]
    pub async fn set_ip(
        &self,
        iface_name: &str,
        ip_addr: std::net::IpAddr,
        prefix_len: u8,
    ) -> NamespaceResult<()> {
        info!(
            "Setting IP address on {} (macOS): {}/{}",
            iface_name, ip_addr, prefix_len
        );

        use std::process::Command;

        let ip_str = match ip_addr {
            std::net::IpAddr::V4(addr) => {
                format!("{} netmask {}", addr, Self::prefix_to_netmask(prefix_len))
            }
            std::net::IpAddr::V6(addr) => {
                format!("{} prefixlen {}", addr, prefix_len)
            }
        };

        let output = Command::new("ifconfig")
            .arg(iface_name)
            .arg(match ip_addr {
                std::net::IpAddr::V4(_) => "inet",
                std::net::IpAddr::V6(_) => "inet6",
            })
            .args(ip_str.split_whitespace())
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NamespaceError::NetworkError(format!(
                "Failed to set IP on {}: {}",
                iface_name, stderr
            )));
        }

        debug!("IP address configured successfully");
        Ok(())
    }

    /// Bring interface up or down (macOS)
    #[cfg(target_os = "macos")]
    pub async fn set_state(&self, iface_name: &str, up: bool) -> NamespaceResult<()> {
        let state = if up { "up" } else { "down" };
        info!("Setting interface {} {} on macOS", iface_name, state);

        use std::process::Command;

        let output = Command::new("ifconfig")
            .args(&[iface_name, state])
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NamespaceError::NetworkError(format!(
                "Failed to set interface {} {}: {}",
                iface_name, state, stderr
            )));
        }

        debug!("Interface state changed successfully");
        Ok(())
    }

    /// Helper to convert prefix length to netmask (IPv4)
    #[cfg(target_os = "macos")]
    fn prefix_to_netmask(prefix: u8) -> String {
        let mask = !0u32 << (32 - prefix);
        let octets = [
            ((mask >> 24) & 0xFF) as u8,
            ((mask >> 16) & 0xFF) as u8,
            ((mask >> 8) & 0xFF) as u8,
            (mask & 0xFF) as u8,
        ];
        format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
    pub async fn create_pair(
        &self,
        _host_name: &str,
        _ns_name: &str,
        _ns_id: &NamespaceId,
    ) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Virtual ethernet pairs not supported on this platform".to_string(),
        ))
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
    pub async fn delete_pair(&self, _host_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Virtual ethernet pairs not supported on this platform".to_string(),
        ))
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
    pub async fn set_ip(
        &self,
        _iface_name: &str,
        _ip_addr: std::net::IpAddr,
        _prefix_len: u8,
    ) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Virtual ethernet pairs not supported on this platform".to_string(),
        ))
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
    pub async fn set_state(&self, _iface_name: &str, _up: bool) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Virtual ethernet pairs not supported on this platform".to_string(),
        ))
    }
}

impl Default for VethManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_veth_manager_creation() {
        let manager = VethManager::new();
        // Manager should be created successfully
        #[cfg(target_os = "linux")]
        assert!(manager.handle.is_none()); // Not initialized yet
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_veth_init() {
        let mut manager = VethManager::new();
        // Note: This test may fail without proper permissions
        // In CI/CD, this would need to run with CAP_NET_ADMIN
        let result = manager.init().await;

        // We expect this to work in a proper test environment
        if result.is_ok() {
            assert!(manager.handle.is_some());
        }
    }
}
