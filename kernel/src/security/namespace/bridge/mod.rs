/*!
 * Network Bridge Management
 * Creates and manages bridges for inter-namespace communication
 */

mod fallback;
mod linux;
mod macos;

use super::types::*;
use std::net::IpAddr;

#[cfg(target_os = "linux")]
pub use linux::LinuxBridgeManager as PlatformBridgeManager;

#[cfg(target_os = "macos")]
pub use macos::MacosBridgeManager as PlatformBridgeManager;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub use fallback::FallbackBridgeManager as PlatformBridgeManager;

/// Unified bridge manager that delegates to platform-specific implementation
pub struct BridgeManager {
    platform: PlatformBridgeManager,
}

impl BridgeManager {
    pub fn new() -> Self {
        Self {
            platform: PlatformBridgeManager::new(),
        }
    }

    pub async fn init(&mut self) -> NamespaceResult<()> {
        self.platform.init().await
    }

    pub async fn create_bridge(&self, bridge_name: &str) -> NamespaceResult<()> {
        self.platform.create_bridge(bridge_name).await
    }

    pub async fn delete_bridge(&self, bridge_name: &str) -> NamespaceResult<()> {
        self.platform.delete_bridge(bridge_name).await
    }

    pub async fn attach_interface(
        &self,
        bridge_name: &str,
        iface_name: &str,
    ) -> NamespaceResult<()> {
        self.platform
            .attach_interface(bridge_name, iface_name)
            .await
    }

    pub async fn detach_interface(&self, iface_name: &str) -> NamespaceResult<()> {
        self.platform.detach_interface(iface_name).await
    }

    pub async fn set_bridge_ip(
        &self,
        bridge_name: &str,
        ip_addr: IpAddr,
        prefix_len: u8,
    ) -> NamespaceResult<()> {
        self.platform
            .set_bridge_ip(bridge_name, ip_addr, prefix_len)
            .await
    }

    pub async fn enable_forwarding(&self, bridge_name: &str) -> NamespaceResult<()> {
        self.platform.enable_forwarding(bridge_name).await
    }
}

impl Default for BridgeManager {
    fn default() -> Self {
        Self::new()
    }
}
