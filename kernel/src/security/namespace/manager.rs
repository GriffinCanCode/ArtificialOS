/*!
 * Network Namespace Manager
 * Platform-aware orchestration of network isolation
 */

use super::linux::LinuxNamespaceManager;
use super::macos::MacOSNamespaceManager;
use super::simulation::SimulationNamespaceManager;
use super::traits::*;
use super::types::*;
use crate::core::types::Pid;
use log::info;

/// Unified namespace manager that selects the appropriate platform implementation
#[derive(Clone)]
pub struct NamespaceManager {
    provider: NamespaceProviderImpl,
}

/// Platform-specific provider implementations
#[derive(Clone)]
enum NamespaceProviderImpl {
    #[allow(dead_code)]
    Linux(LinuxNamespaceManager),
    MacOS(MacOSNamespaceManager),
    Simulation(SimulationNamespaceManager),
}

impl NamespaceManager {
    /// Create a new namespace manager, auto-detecting the best implementation
    pub fn new() -> Self {
        let provider = Self::select_provider();
        let platform_name = match &provider {
            NamespaceProviderImpl::Linux(_) => "Linux (network namespaces)",
            NamespaceProviderImpl::MacOS(_) => "macOS (packet filters)",
            NamespaceProviderImpl::Simulation(_) => "Simulation",
        };
        info!(
            "Network namespace manager initialized using: {}",
            platform_name
        );
        Self { provider }
    }

    /// Select the best available provider for the current platform
    fn select_provider() -> NamespaceProviderImpl {
        #[cfg(target_os = "linux")]
        {
            let linux_mgr = LinuxNamespaceManager::new();
            if linux_mgr.is_supported() {
                return NamespaceProviderImpl::Linux(linux_mgr);
            }
        }

        #[cfg(target_os = "macos")]
        {
            let macos_mgr = MacOSNamespaceManager::new();
            if macos_mgr.is_supported() {
                return NamespaceProviderImpl::MacOS(macos_mgr);
            }
        }

        // Fallback to simulation mode
        NamespaceProviderImpl::Simulation(SimulationNamespaceManager::new())
    }

    /// Force a specific platform implementation
    #[cfg(test)]
    pub fn with_simulation() -> Self {
        Self {
            provider: NamespaceProviderImpl::Simulation(SimulationNamespaceManager::new()),
        }
    }

    /// Get the current platform type
    pub fn platform(&self) -> PlatformType {
        match &self.provider {
            NamespaceProviderImpl::Linux(m) => m.platform(),
            NamespaceProviderImpl::MacOS(m) => m.platform(),
            NamespaceProviderImpl::Simulation(m) => m.platform(),
        }
    }

    /// Check if true OS-level isolation is available
    pub fn has_true_isolation(&self) -> bool {
        matches!(
            self.provider,
            NamespaceProviderImpl::Linux(_) | NamespaceProviderImpl::MacOS(_)
        )
    }

    /// Create a new network namespace
    pub fn create(&self, config: NamespaceConfig) -> NamespaceResult<()> {
        match &self.provider {
            NamespaceProviderImpl::Linux(m) => m.create(config),
            NamespaceProviderImpl::MacOS(m) => m.create(config),
            NamespaceProviderImpl::Simulation(m) => m.create(config),
        }
    }

    /// Destroy a network namespace
    pub fn destroy(&self, id: &NamespaceId) -> NamespaceResult<()> {
        match &self.provider {
            NamespaceProviderImpl::Linux(m) => m.destroy(id),
            NamespaceProviderImpl::MacOS(m) => m.destroy(id),
            NamespaceProviderImpl::Simulation(m) => m.destroy(id),
        }
    }

    /// Check if a namespace exists
    pub fn exists(&self, id: &NamespaceId) -> bool {
        match &self.provider {
            NamespaceProviderImpl::Linux(m) => m.exists(id),
            NamespaceProviderImpl::MacOS(m) => m.exists(id),
            NamespaceProviderImpl::Simulation(m) => m.exists(id),
        }
    }

    /// Get namespace information
    pub fn get_info(&self, id: &NamespaceId) -> Option<NamespaceInfo> {
        match &self.provider {
            NamespaceProviderImpl::Linux(m) => m.get_info(id),
            NamespaceProviderImpl::MacOS(m) => m.get_info(id),
            NamespaceProviderImpl::Simulation(m) => m.get_info(id),
        }
    }

    /// List all namespaces
    pub fn list(&self) -> Vec<NamespaceInfo> {
        match &self.provider {
            NamespaceProviderImpl::Linux(m) => m.list(),
            NamespaceProviderImpl::MacOS(m) => m.list(),
            NamespaceProviderImpl::Simulation(m) => m.list(),
        }
    }

    /// Get namespace for a process
    pub fn get_by_pid(&self, pid: Pid) -> Option<NamespaceInfo> {
        match &self.provider {
            NamespaceProviderImpl::Linux(m) => m.get_by_pid(pid),
            NamespaceProviderImpl::MacOS(m) => m.get_by_pid(pid),
            NamespaceProviderImpl::Simulation(m) => m.get_by_pid(pid),
        }
    }

    /// Get namespace statistics
    pub fn get_stats(&self, id: &NamespaceId) -> Option<NamespaceStats> {
        match &self.provider {
            NamespaceProviderImpl::Linux(m) => m.get_stats(id),
            NamespaceProviderImpl::MacOS(m) => m.get_stats(id),
            NamespaceProviderImpl::Simulation(m) => m.get_stats(id),
        }
    }

    /// Get total namespace count
    pub fn count(&self) -> usize {
        self.list().len()
    }
}

impl Default for NamespaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = NamespaceManager::new();
        assert!(manager.platform() != PlatformType::WindowsWFP);
    }

    #[test]
    fn test_simulation_mode() {
        let manager = NamespaceManager::with_simulation();
        assert_eq!(manager.platform(), PlatformType::Simulation);
    }

    #[test]
    fn test_create_and_destroy() {
        let manager = NamespaceManager::with_simulation();
        let config = NamespaceConfig::full_isolation(1);

        assert!(manager.create(config.clone()).is_ok());
        assert!(manager.exists(&config.id));
        assert!(manager.destroy(&config.id).is_ok());
        assert!(!manager.exists(&config.id));
    }

    #[test]
    fn test_list_namespaces() {
        let manager = NamespaceManager::with_simulation();
        let config1 = NamespaceConfig::full_isolation(1);
        let config2 = NamespaceConfig::private_network(2);

        manager.create(config1.clone()).unwrap();
        manager.create(config2.clone()).unwrap();

        let namespaces = manager.list();
        assert_eq!(namespaces.len(), 2);

        manager.destroy(&config1.id).unwrap();
        manager.destroy(&config2.id).unwrap();
    }

    #[test]
    fn test_get_by_pid() {
        let manager = NamespaceManager::with_simulation();
        let config = NamespaceConfig::full_isolation(42);

        manager.create(config.clone()).unwrap();

        let info = manager.get_by_pid(42);
        assert!(info.is_some());
        assert_eq!(info.unwrap().config.pid, 42);

        manager.destroy(&config.id).unwrap();
    }
}
