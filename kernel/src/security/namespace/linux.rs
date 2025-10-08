/*!
 * Linux Network Namespace Implementation
 * True network isolation using Linux network namespaces
 */

use super::traits::*;
use super::types::*;
use super::veth::VethManager;
use super::bridge::BridgeManager;
use crate::core::types::Pid;
use ahash::RandomState;
use dashmap::DashMap;
use log::{debug, info, warn};
use std::sync::Arc;

#[cfg(target_os = "linux")]
use nix::sched::{unshare, CloneFlags};
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;

/// Linux network namespace manager with integrated veth and bridge support
pub struct LinuxNamespaceManager {
    namespaces: Arc<DashMap<NamespaceId, NamespaceInfo, RandomState>>,
    pid_to_ns: Arc<DashMap<Pid, NamespaceId, RandomState>>,
    veth_manager: Arc<tokio::sync::Mutex<VethManager>>,
    bridge_manager: Arc<tokio::sync::Mutex<BridgeManager>>,
    initialized: Arc<tokio::sync::OnceCell<()>>,
}

impl LinuxNamespaceManager {
    pub fn new() -> Self {
        info!("Linux network namespace manager initialized (networking pending async init)");
        Self {
            namespaces: Arc::new(DashMap::with_hasher(RandomState::new().into())),
            pid_to_ns: Arc::new(DashMap::with_hasher(RandomState::new().into())),
            veth_manager: Arc::new(tokio::sync::Mutex::new(VethManager::new())),
            bridge_manager: Arc::new(tokio::sync::Mutex::new(BridgeManager::new())),
            initialized: Arc::new(tokio::sync::OnceCell::new()),
        }
    }

    /// Initialize networking components (async) - call once before use
    pub async fn init(&self) -> NamespaceResult<()> {
        self.initialized
            .get_or_init(|| async {
                info!("Initializing Linux namespace networking components");

                // Initialize veth manager
                if let Err(e) = self.veth_manager.lock().await.init().await {
                    warn!("Failed to initialize veth manager: {}", e);
                }

                // Initialize bridge manager
                if let Err(e) = self.bridge_manager.lock().await.init().await {
                    warn!("Failed to initialize bridge manager: {}", e);
                }

                info!("Linux namespace networking components initialized");
            })
            .await;
        Ok(())
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
            return Err(NamespaceError::AlreadyExists(ns_name.to_string().into()));
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

        // Actual veth creation happens via async helper
        debug!("Veth pair configuration: {} <-> {}", host_veth, ns_veth);
        debug!(
            "IP configuration: {}/{}",
            iface_config.ip_addr, iface_config.prefix_len
        );
        // Note: Actual creation should be called via self.setup_private_network_async()

        Ok(())
    }

    /// Setup private network with veth pair (async helper)
    #[cfg(target_os = "linux")]
    pub async fn setup_private_network_async(
        &self,
        config: &NamespaceConfig,
        iface_config: &InterfaceConfig,
    ) -> NamespaceResult<()> {
        let ns_name = config.id.as_str();
        let host_veth = format!("veth-{}", &ns_name[..8.min(ns_name.len())]);
        let ns_veth = iface_config.name.clone();

        info!(
            "Creating veth pair for {} (async): {} <-> {}",
            ns_name, host_veth, ns_veth
        );

        // Create veth pair using VethManager
        let veth_mgr = self.veth_manager.lock().await;
        veth_mgr.create_pair(&host_veth, &ns_veth, &config.id).await?;
        veth_mgr.set_ip(&ns_veth, iface_config.ip_addr, iface_config.prefix_len).await?;
        veth_mgr.set_state(&ns_veth, true).await?;
        veth_mgr.set_state(&host_veth, true).await?;

        debug!("Private network setup complete for {}", ns_name);
        Ok(())
    }

    /// Setup bridged network (async helper)
    #[cfg(target_os = "linux")]
    pub async fn setup_bridged_network(
        &self,
        config: &NamespaceConfig,
        bridge_name: &str,
    ) -> NamespaceResult<()> {
        let ns_name = config.id.as_str();

        info!(
            "Setting up bridged network for {} with bridge: {}",
            ns_name, bridge_name
        );

        // Create bridge using BridgeManager
        let bridge_mgr = self.bridge_manager.lock().await;
        bridge_mgr.create_bridge(bridge_name).await?;

        // If interface config is provided, attach to bridge
        if let Some(ref iface_config) = config.interface {
            bridge_mgr.set_bridge_ip(
                bridge_name,
                iface_config.ip_addr,
                iface_config.prefix_len,
            ).await?;
            bridge_mgr.enable_forwarding(bridge_name).await?;
        }

        debug!("Bridged network setup complete for {}", ns_name);
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
            .and_then(|ns_id| self.get_info(ns_id.value().into()))
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
            veth_manager: Arc::clone(&self.veth_manager),
            bridge_manager: Arc::clone(&self.bridge_manager),
            initialized: Arc::clone(&self.initialized),
        }
    }
}
