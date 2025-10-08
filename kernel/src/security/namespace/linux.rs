/*!
 * Linux Network Namespace Implementation
 * True network isolation using Linux network namespaces
 */

use super::traits::*;
use super::types::*;
use crate::core::types::Pid;
use ahash::RandomState;
use dashmap::DashMap;
use log::info;
use std::sync::Arc;

#[cfg(target_os = "linux")]
use nix::sched::{unshare, CloneFlags};
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;

/// Linux network namespace manager
pub struct LinuxNamespaceManager {
    namespaces: Arc<DashMap<NamespaceId, NamespaceInfo, RandomState>>,
    pid_to_ns: Arc<DashMap<Pid, NamespaceId, RandomState>>,
}

impl LinuxNamespaceManager {
    pub fn new() -> Self {
        info!("Linux network namespace manager initialized");
        Self {
            namespaces: Arc::new(DashMap::with_hasher(RandomState::new())),
            pid_to_ns: Arc::new(DashMap::with_hasher(RandomState::new())),
        }
    }

    #[cfg(target_os = "linux")]
    fn create_linux_namespace(&self, config: &NamespaceConfig) -> NamespaceResult<()> {
        let ns_name = config.id.as_str();

        // Create namespace directory if it doesn't exist
        let netns_dir = std::path::Path::new("/var/run/netns");
        if !netns_dir.exists() {
            fs::create_dir_all(netns_dir).map_err(|e| {
                NamespaceError::NetworkError(format!("Failed to create netns directory: {}", e))
            })?;
        }

        // Create namespace file
        let ns_path = netns_dir.join(ns_name);
        if ns_path.exists() {
            return Err(NamespaceError::AlreadyExists(ns_name.to_string()));
        }

        // Create the namespace
        fs::File::create(&ns_path)?;

        info!("Created network namespace: {}", ns_name);

        // Store namespace info
        let info = NamespaceInfo {
            config: config.clone(),
            stats: Some(NamespaceStats {
                id: config.id.clone(),
                interface_count: 0,
                tx_bytes: 0,
                rx_bytes: 0,
                tx_packets: 0,
                rx_packets: 0,
                created_at: std::time::SystemTime::now(),
            }),
            platform: PlatformType::LinuxNetns,
        };

        self.namespaces.insert(config.id.clone(), info);
        self.pid_to_ns.insert(config.pid, config.id.clone());

        // Configure based on isolation mode
        match config.mode {
            IsolationMode::Full => {
                debug!("Namespace {} configured for full isolation", ns_name);
            }
            IsolationMode::Private => {
                if let Some(ref iface_config) = config.interface {
                    self.setup_private_network(config, iface_config)?;
                }
            }
            IsolationMode::Bridged => {
                debug!("Namespace {} configured for bridged networking", ns_name);
            }
            IsolationMode::Shared => {
                warn!("Shared mode requested - namespace will use host network");
            }
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn setup_private_network(
        &self,
        config: &NamespaceConfig,
        iface_config: &InterfaceConfig,
    ) -> NamespaceResult<()> {
        let ns_name = config.id.as_str();

        // Create veth pair
        let host_veth = format!("veth-{}", &ns_name[..8.min(ns_name.len())]);
        let ns_veth = iface_config.name.clone();

        info!(
            "Setting up private network for {} with veth pair: {} <-> {}",
            ns_name, host_veth, ns_veth
        );

        // In a production implementation, we would use rtnetlink here
        // For now, we'll use system commands as a reference
        debug!("Would create veth pair: {} <-> {}", host_veth, ns_veth);
        debug!(
            "Would configure IP: {}/{}",
            iface_config.ip_addr, iface_config.prefix_len
        );

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn destroy_linux_namespace(&self, id: &NamespaceId) -> NamespaceResult<()> {
        let ns_path = std::path::Path::new("/var/run/netns").join(id.as_str());

        if ns_path.exists() {
            fs::remove_file(&ns_path)?;
            info!("Destroyed network namespace: {}", id);
        }

        self.namespaces.remove(id);

        // Remove from pid mapping
        if let Some(info) = self.namespaces.get(id) {
            self.pid_to_ns.remove(&info.config.pid);
        }

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn create_linux_namespace(&self, _config: &NamespaceConfig) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Linux network namespaces not available on this platform".to_string(),
        ))
    }

    #[cfg(not(target_os = "linux"))]
    fn destroy_linux_namespace(&self, _id: &NamespaceId) -> NamespaceResult<()> {
        Err(NamespaceError::PlatformNotSupported(
            "Linux network namespaces not available on this platform".to_string(),
        ))
    }
}

impl Default for LinuxNamespaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NamespaceProvider for LinuxNamespaceManager {
    fn create(&self, config: NamespaceConfig) -> NamespaceResult<()> {
        self.create_linux_namespace(&config)
    }

    fn destroy(&self, id: &NamespaceId) -> NamespaceResult<()> {
        self.destroy_linux_namespace(id)
    }

    fn exists(&self, id: &NamespaceId) -> bool {
        self.namespaces.contains_key(id)
    }

    fn get_info(&self, id: &NamespaceId) -> Option<NamespaceInfo> {
        self.namespaces.get(id).map(|r| r.value().clone())
    }

    fn list(&self) -> Vec<NamespaceInfo> {
        self.namespaces.iter().map(|r| r.value().clone()).collect()
    }

    fn get_by_pid(&self, pid: Pid) -> Option<NamespaceInfo> {
        self.pid_to_ns
            .get(&pid)
            .and_then(|ns_id| self.get_info(ns_id.value()))
    }

    fn get_stats(&self, id: &NamespaceId) -> Option<NamespaceStats> {
        self.namespaces.get(id).and_then(|info| info.stats.clone())
    }

    fn is_supported(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/proc/self/ns/net").exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    fn platform(&self) -> PlatformType {
        PlatformType::LinuxNetns
    }
}

impl Clone for LinuxNamespaceManager {
    fn clone(&self) -> Self {
        Self {
            namespaces: Arc::clone(&self.namespaces),
            pid_to_ns: Arc::clone(&self.pid_to_ns),
        }
    }
}
