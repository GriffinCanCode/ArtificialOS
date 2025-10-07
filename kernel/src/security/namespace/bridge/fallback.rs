/*!
 * Fallback Bridge Implementation
 * For platforms that don't support network bridges
 */

use super::super::types::*;
use log::info;
use std::net::IpAddr;

/// Fallback bridge manager for unsupported platforms
pub struct FallbackBridgeManager {}

impl FallbackBridgeManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn init(&mut self) -> NamespaceResult<()> {
        info!("FallbackBridgeManager initialized (platform-specific mode)");
        Ok(())
    }

    pub async fn create_bridge(&self, _bridge_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }

    pub async fn delete_bridge(&self, _bridge_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }

    pub async fn attach_interface(
        &self,
        _bridge_name: &str,
        _iface_name: &str,
    ) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }

    pub async fn detach_interface(&self, _iface_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }

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

    pub async fn enable_forwarding(&self, _bridge_name: &str) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Network bridges not supported on this platform".to_string(),
        ))
    }
}

impl Default for FallbackBridgeManager {
    fn default() -> Self {
        Self::new()
    }
}
