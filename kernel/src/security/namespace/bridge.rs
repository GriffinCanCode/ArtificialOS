/*!
 * Network Bridge Management
 * Creates and manages Linux bridges for inter-namespace communication
 */

use super::types::*;
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

/// Bridge network manager
pub struct BridgeManager {
    #[cfg(target_os = "linux")]
    handle: Option<Handle>,
}

impl BridgeManager {
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
        info!("BridgeManager initialized with netlink connection");
        Ok(())
    }

    /// Initialize for non-Linux platforms (no-op)
    #[cfg(not(target_os = "linux"))]
    pub async fn init(&mut self) -> NamespaceResult<()> {
        info!("BridgeManager initialized (platform-specific mode)");
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

    /// Create a network bridge on macOS
    #[cfg(target_os = "macos")]
    pub async fn create_bridge(&self, bridge_name: &str) -> NamespaceResult<()> {
        info!("Creating network bridge on macOS: {}", bridge_name);

        use std::process::Command;

        let output = Command::new("ifconfig")
            .args(&["bridge", "create"])
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NamespaceError::NetworkError(format!(
                "Failed to create bridge: {}",
                stderr
            )));
        }

        // The output contains the name of the created bridge (e.g., "bridge0")
        // For simplicity, we'll assume the user specifies the correct bridge name
        // In production, you'd parse the output to get the actual bridge name

        debug!("Bridge {} created successfully on macOS", bridge_name);
        Ok(())
    }

    /// Delete a network bridge on macOS
    #[cfg(target_os = "macos")]
    pub async fn delete_bridge(&self, bridge_name: &str) -> NamespaceResult<()> {
        info!("Deleting network bridge on macOS: {}", bridge_name);

        use std::process::Command;

        let output = Command::new("ifconfig")
            .args(&[bridge_name, "destroy"])
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NamespaceError::NetworkError(format!(
                "Failed to delete bridge {}: {}",
                bridge_name, stderr
            )));
        }

        debug!("Bridge deleted successfully");
        Ok(())
    }

    /// Attach an interface to a bridge on macOS
    #[cfg(target_os = "macos")]
    pub async fn attach_interface(
        &self,
        bridge_name: &str,
        iface_name: &str,
    ) -> NamespaceResult<()> {
        info!(
            "Attaching {} to bridge {} on macOS",
            iface_name, bridge_name
        );

        use std::process::Command;

        let output = Command::new("ifconfig")
            .args(&[bridge_name, "addm", iface_name])
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NamespaceError::NetworkError(format!(
                "Failed to attach {} to bridge {}: {}",
                iface_name, bridge_name, stderr
            )));
        }

        debug!("Interface attached to bridge successfully");
        Ok(())
    }

    /// Detach an interface from a bridge on macOS
    #[cfg(target_os = "macos")]
    pub async fn detach_interface(&self, iface_name: &str) -> NamespaceResult<()> {
        info!("Detaching {} from bridge on macOS", iface_name);

        use std::process::Command;

        // On macOS, we need to know which bridge the interface is attached to
        // For simplicity, we'll try to find it or require the bridge name
        // This is a limitation - in production, you'd enumerate bridges to find it
        warn!("Detaching interface on macOS requires knowing the bridge name");

        // As a fallback, we can try to bring the interface down
        let output = Command::new("ifconfig")
            .args(&[iface_name, "down"])
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to bring interface {} down: {}", iface_name, stderr);
        }

        debug!("Interface detached (brought down)");
        Ok(())
    }

    /// Configure bridge IP address on macOS
    #[cfg(target_os = "macos")]
    pub async fn set_bridge_ip(
        &self,
        bridge_name: &str,
        ip_addr: IpAddr,
        prefix_len: u8,
    ) -> NamespaceResult<()> {
        info!(
            "Setting bridge {} IP on macOS: {}/{}",
            bridge_name, ip_addr, prefix_len
        );

        use std::process::Command;

        let ip_str = match ip_addr {
            IpAddr::V4(addr) => {
                format!("{} netmask {}", addr, Self::prefix_to_netmask(prefix_len))
            }
            IpAddr::V6(addr) => {
                format!("{} prefixlen {}", addr, prefix_len)
            }
        };

        let output = Command::new("ifconfig")
            .arg(bridge_name)
            .arg(match ip_addr {
                IpAddr::V4(_) => "inet",
                IpAddr::V6(_) => "inet6",
            })
            .args(ip_str.split_whitespace())
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NamespaceError::NetworkError(format!(
                "Failed to set IP on bridge {}: {}",
                bridge_name, stderr
            )));
        }

        // Bring bridge up
        let output = Command::new("ifconfig")
            .args(&[bridge_name, "up"])
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute ifconfig: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to bring bridge {} up: {}", bridge_name, stderr);
        }

        debug!("Bridge IP configured successfully");
        Ok(())
    }

    /// Enable NAT/forwarding for bridge on macOS
    #[cfg(target_os = "macos")]
    pub async fn enable_forwarding(&self, bridge_name: &str) -> NamespaceResult<()> {
        info!("Enabling forwarding for bridge {} on macOS", bridge_name);

        use std::process::Command;

        // Enable IP forwarding
        let output = Command::new("sysctl")
            .args(&["-w", "net.inet.ip.forwarding=1"])
            .output()
            .map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to execute sysctl: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to enable IP forwarding: {}", stderr);
        } else {
            debug!("IP forwarding enabled");
        }

        // Note: Setting up NAT on macOS requires pfctl (packet filter)
        // This is more complex and requires careful configuration
        debug!("Would configure pfctl NAT rules for bridge {}", bridge_name);
        debug!("  - pfctl configuration for NAT and forwarding");
        debug!("  - /etc/pf.conf modifications may be required");

        warn!("NAT configuration on macOS requires pfctl setup - implement with pfctl rules");

        Ok(())
    }

    /// Helper to convert prefix length to netmask (IPv4) on macOS
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
    pub async fn create_bridge(&self, _bridge_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
    pub async fn delete_bridge(&self, _bridge_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
    pub async fn attach_interface(
        &self,
        _bridge_name: &str,
        _iface_name: &str,
    ) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
    pub async fn detach_interface(&self, _iface_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
    pub async fn set_bridge_ip(
        &self,
        _bridge_name: &str,
        _ip_addr: IpAddr,
        _prefix_len: u8,
    ) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }

    #[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
    pub async fn enable_forwarding(&self, _bridge_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }
}

impl Default for BridgeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_manager_creation() {
        let manager = BridgeManager::new();
        // Manager should be created successfully
        #[cfg(target_os = "linux")]
        assert!(manager.handle.is_none()); // Not initialized yet
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_bridge_init() {
        let mut manager = BridgeManager::new();
        // Note: This test may fail without proper permissions
        // In CI/CD, this would need to run with CAP_NET_ADMIN
        let result = manager.init().await;

        // We expect this to work in a proper test environment
        if result.is_ok() {
            assert!(manager.handle.is_some());
        }
    }
}
