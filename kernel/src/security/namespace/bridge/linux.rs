/*!
 * Linux Network Bridge Implementation
 * Uses rtnetlink for bridge management
 */

use super::super::types::*;
use log::{debug, info, warn};
use std::net::IpAddr;

#[cfg(target_os = "linux")]
use futures::stream::TryStreamExt;
#[cfg(target_os = "linux")]
use rtnetlink::{new_connection, Handle};
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::io::Write;

/// Bridge network manager for Linux
pub struct LinuxBridgeManager {
    #[cfg(target_os = "linux")]
    handle: Option<Handle>,
}

impl LinuxBridgeManager {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "linux")]
            handle: None,
        }
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
        info!("LinuxBridgeManager initialized with netlink connection");
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn init(&mut self) -> NamespaceResult<()> {
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn get_handle(&self) -> NamespaceResult<&Handle> {
        self.handle.as_ref().ok_or_else(|| {
            NamespaceError::InvalidConfig(
                "BridgeManager not initialized. Call init() first.".to_string(),
            )
        })
    }

    /// Create a network bridge
    #[cfg(target_os = "linux")]
    pub async fn create_bridge(&self, bridge_name: &str) -> NamespaceResult<()> {
        info!("Creating network bridge: {}", bridge_name);

        let handle = self.get_handle()?;

        // Create bridge interface
        handle
            .link()
            .add()
            .bridge(bridge_name.to_string())
            .execute()
            .await
            .map_err(|e| {
                NamespaceError::NetworkError(format!(
                    "Failed to create bridge {}: {}",
                    bridge_name, e
                ))
            })?;

        debug!("Bridge {} created successfully", bridge_name);
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn create_bridge(&self, _bridge_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Linux bridge operations not supported on this platform".to_string(),
        ))
    }

    /// Delete a network bridge
    #[cfg(target_os = "linux")]
    pub async fn delete_bridge(&self, bridge_name: &str) -> NamespaceResult<()> {
        info!("Deleting network bridge: {}", bridge_name);

        let handle = self.get_handle()?;

        // Get the bridge index
        let mut links = handle
            .link()
            .get()
            .match_name(bridge_name.to_string())
            .execute();

        if let Some(link) = links.try_next().await.map_err(|e| {
            NamespaceError::NetworkError(format!("Failed to find bridge {}: {}", bridge_name, e))
        })? {
            // Delete the bridge
            handle
                .link()
                .del(link.header.index)
                .execute()
                .await
                .map_err(|e| {
                    NamespaceError::NetworkError(format!(
                        "Failed to delete bridge {}: {}",
                        bridge_name, e
                    ))
                })?;

            debug!("Bridge deleted successfully");
            Ok(())
        } else {
            Err(NamespaceError::NotFound(format!(
                "Bridge {} not found",
                bridge_name
            )))
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn delete_bridge(&self, _bridge_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Linux bridge operations not supported on this platform".to_string(),
        ))
    }

    /// Attach an interface to a bridge
    #[cfg(target_os = "linux")]
    pub async fn attach_interface(
        &self,
        bridge_name: &str,
        iface_name: &str,
    ) -> NamespaceResult<()> {
        info!("Attaching {} to bridge {}", iface_name, bridge_name);

        let handle = self.get_handle()?;

        // Get the bridge index
        let mut bridge_links = handle
            .link()
            .get()
            .match_name(bridge_name.to_string())
            .execute();
        let bridge_link = bridge_links
            .try_next()
            .await
            .map_err(|e| {
                NamespaceError::NetworkError(format!(
                    "Failed to find bridge {}: {}",
                    bridge_name, e
                ))
            })?
            .ok_or_else(|| NamespaceError::NotFound(format!("Bridge {} not found", bridge_name)))?;

        // Get the interface index
        let mut iface_links = handle
            .link()
            .get()
            .match_name(iface_name.to_string())
            .execute();
        let iface_link = iface_links
            .try_next()
            .await
            .map_err(|e| {
                NamespaceError::NetworkError(format!(
                    "Failed to find interface {}: {}",
                    iface_name, e
                ))
            })?
            .ok_or_else(|| {
                NamespaceError::NotFound(format!("Interface {} not found", iface_name))
            })?;

        // Set the interface's master to the bridge
        handle
            .link()
            .set(iface_link.header.index)
            .master(bridge_link.header.index)
            .execute()
            .await
            .map_err(|e| {
                NamespaceError::NetworkError(format!(
                    "Failed to attach {} to bridge {}: {}",
                    iface_name, bridge_name, e
                ))
            })?;

        debug!("Interface attached to bridge successfully");
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn attach_interface(
        &self,
        _bridge_name: &str,
        _iface_name: &str,
    ) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Linux bridge operations not supported on this platform".to_string(),
        ))
    }

    /// Detach an interface from a bridge
    #[cfg(target_os = "linux")]
    pub async fn detach_interface(&self, iface_name: &str) -> NamespaceResult<()> {
        info!("Detaching {} from bridge", iface_name);

        let handle = self.get_handle()?;

        // Get the interface index
        let mut links = handle
            .link()
            .get()
            .match_name(iface_name.to_string())
            .execute();

        if let Some(link) = links.try_next().await.map_err(|e| {
            NamespaceError::NetworkError(format!("Failed to find interface {}: {}", iface_name, e))
        })? {
            // Remove the master (set to no master)
            handle
                .link()
                .set(link.header.index)
                .nomaster()
                .execute()
                .await
                .map_err(|e| {
                    NamespaceError::NetworkError(format!(
                        "Failed to detach {} from bridge: {}",
                        iface_name, e
                    ))
                })?;

            debug!("Interface detached from bridge successfully");
            Ok(())
        } else {
            Err(NamespaceError::NotFound(format!(
                "Interface {} not found",
                iface_name
            )))
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn detach_interface(&self, _iface_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Linux bridge operations not supported on this platform".to_string(),
        ))
    }

    /// Configure bridge IP address
    #[cfg(target_os = "linux")]
    pub async fn set_bridge_ip(
        &self,
        bridge_name: &str,
        ip_addr: IpAddr,
        prefix_len: u8,
    ) -> NamespaceResult<()> {
        info!(
            "Setting bridge {} IP: {}/{}",
            bridge_name, ip_addr, prefix_len
        );

        let handle = self.get_handle()?;

        // Get the bridge index
        let mut links = handle
            .link()
            .get()
            .match_name(bridge_name.to_string())
            .execute();

        if let Some(link) = links.try_next().await.map_err(|e| {
            NamespaceError::NetworkError(format!("Failed to find bridge {}: {}", bridge_name, e))
        })? {
            // Add IP address to bridge
            handle
                .address()
                .add(link.header.index, ip_addr, prefix_len)
                .execute()
                .await
                .map_err(|e| {
                    NamespaceError::NetworkError(format!(
                        "Failed to set IP {}/{} on bridge {}: {}",
                        ip_addr, prefix_len, bridge_name, e
                    ))
                })?;

            // Bring bridge up
            handle
                .link()
                .set(link.header.index)
                .up()
                .execute()
                .await
                .map_err(|e| {
                    NamespaceError::NetworkError(format!(
                        "Failed to bring bridge {} up: {}",
                        bridge_name, e
                    ))
                })?;

            debug!("Bridge IP configured successfully");
            Ok(())
        } else {
            Err(NamespaceError::NotFound(format!(
                "Bridge {} not found",
                bridge_name
            )))
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn set_bridge_ip(
        &self,
        _bridge_name: &str,
        _ip_addr: IpAddr,
        _prefix_len: u8,
    ) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Linux bridge operations not supported on this platform".to_string(),
        ))
    }

    /// Enable NAT/forwarding for bridge
    #[cfg(target_os = "linux")]
    pub async fn enable_forwarding(&self, bridge_name: &str) -> NamespaceResult<()> {
        info!("Enabling forwarding for bridge {}", bridge_name);

        // Enable IP forwarding
        let ipv4_forward_path = "/proc/sys/net/ipv4/ip_forward";
        if let Ok(mut file) = fs::OpenOptions::new().write(true).open(ipv4_forward_path) {
            if let Err(e) = file.write_all(b"1") {
                warn!("Failed to enable IPv4 forwarding: {}", e);
            } else {
                debug!("IPv4 forwarding enabled");
            }
        } else {
            warn!(
                "Unable to open {} for writing. May need elevated privileges.",
                ipv4_forward_path
            );
        }

        // Note: Setting up iptables rules requires external command execution
        // or using a library like iptables-rs. For now, we'll log what should be done.
        debug!(
            "Would configure iptables NAT rules for bridge {}",
            bridge_name
        );
        debug!("  - iptables -t nat -A POSTROUTING -s <bridge_subnet> -j MASQUERADE");
        debug!("  - iptables -A FORWARD -i {} -j ACCEPT", bridge_name);
        debug!("  - iptables -A FORWARD -o {} -j ACCEPT", bridge_name);

        warn!("NAT configuration requires iptables setup - implement with iptables-rs or nftables");

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn enable_forwarding(&self, _bridge_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Linux bridge operations not supported on this platform".to_string(),
        ))
    }
}

impl Default for LinuxBridgeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_manager_creation() {
        let manager = LinuxBridgeManager::new();
        // Manager should be created successfully
        #[cfg(target_os = "linux")]
        assert!(manager.handle.is_none()); // Not initialized yet
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_bridge_init() {
        let mut manager = LinuxBridgeManager::new();
        // Note: This test may fail without proper permissions
        // In CI/CD, this would need to run with CAP_NET_ADMIN
        let result = manager.init().await;

        // We expect this to work in a proper test environment
        if result.is_ok() {
            assert!(manager.handle.is_some());
        }
    }
}
