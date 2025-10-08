/*!
 * eBPF Manager
 * Platform-aware orchestration of eBPF-based syscall filtering and monitoring
 */

use super::linux::LinuxEbpfProvider;
use super::macos::MacOSEbpfProvider;
use super::simulation::SimulationEbpfProvider;
use super::traits::*;
use super::types::*;
use crate::core::types::Pid;
use log::info;

/// Unified eBPF manager that selects the appropriate platform implementation
#[derive(Clone)]
pub struct EbpfManagerImpl {
    provider: EbpfProviderImpl,
}

/// Platform-specific provider implementations
#[derive(Clone)]
enum EbpfProviderImpl {
    #[allow(dead_code)]
    Linux(LinuxEbpfProvider),
    MacOS(MacOSEbpfProvider),
    Simulation(SimulationEbpfProvider),
}

impl EbpfManagerImpl {
    /// Create a new eBPF manager, auto-detecting the best implementation
    pub fn new() -> Self {
        let provider = Self::select_provider();
        let platform_name = match &provider {
            EbpfProviderImpl::Linux(_) => "Linux (eBPF)",
            EbpfProviderImpl::MacOS(_) => "macOS (DTrace)",
            EbpfProviderImpl::Simulation(_) => "Simulation",
        };
        info!("eBPF manager initialized using: {}", platform_name);
        Self { provider }
    }

    /// Select the best available provider for the current platform
    fn select_provider() -> EbpfProviderImpl {
        #[cfg(target_os = "linux")]
        {
            let linux_provider = LinuxEbpfProvider::new();
            if linux_provider.is_supported() {
                return EbpfProviderImpl::Linux(linux_provider);
            }
        }

        #[cfg(target_os = "macos")]
        {
            let macos_provider = MacOSEbpfProvider::new();
            if macos_provider.is_supported() {
                return EbpfProviderImpl::MacOS(macos_provider);
            }
        }

        // Fallback to simulation mode
        EbpfProviderImpl::Simulation(SimulationEbpfProvider::new())
    }

    /// Force simulation mode (for testing)
    pub fn with_simulation() -> Self {
        Self {
            provider: EbpfProviderImpl::Simulation(SimulationEbpfProvider::new().into()),
        }
    }

    /// Check if true eBPF is available
    pub fn has_true_ebpf(&self) -> bool {
        matches!(self.provider, EbpfProviderImpl::Linux(_))
    }
}

impl Default for EbpfManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

// Implement all traits by delegating to the selected provider

impl EbpfProvider for EbpfManagerImpl {
    fn is_supported(&self) -> bool {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.is_supported(),
            EbpfProviderImpl::MacOS(p) => p.is_supported(),
            EbpfProviderImpl::Simulation(p) => p.is_supported(),
        }
    }

    fn platform(&self) -> EbpfPlatform {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.platform(),
            EbpfProviderImpl::MacOS(p) => p.platform(),
            EbpfProviderImpl::Simulation(p) => p.platform(),
        }
    }

    fn load_program(&self, config: ProgramConfig) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.load_program(config),
            EbpfProviderImpl::MacOS(p) => p.load_program(config),
            EbpfProviderImpl::Simulation(p) => p.load_program(config),
        }
    }

    fn unload_program(&self, name: &str) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.unload_program(name),
            EbpfProviderImpl::MacOS(p) => p.unload_program(name),
            EbpfProviderImpl::Simulation(p) => p.unload_program(name),
        }
    }

    fn attach_program(&self, name: &str) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.attach_program(name),
            EbpfProviderImpl::MacOS(p) => p.attach_program(name),
            EbpfProviderImpl::Simulation(p) => p.attach_program(name),
        }
    }

    fn detach_program(&self, name: &str) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.detach_program(name),
            EbpfProviderImpl::MacOS(p) => p.detach_program(name),
            EbpfProviderImpl::Simulation(p) => p.detach_program(name),
        }
    }

    fn list_programs(&self) -> Vec<ProgramInfo> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.list_programs(),
            EbpfProviderImpl::MacOS(p) => p.list_programs(),
            EbpfProviderImpl::Simulation(p) => p.list_programs(),
        }
    }

    fn get_program_info(&self, name: &str) -> Option<ProgramInfo> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.get_program_info(name),
            EbpfProviderImpl::MacOS(p) => p.get_program_info(name),
            EbpfProviderImpl::Simulation(p) => p.get_program_info(name),
        }
    }

    fn stats(&self) -> EbpfStats {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.stats(),
            EbpfProviderImpl::MacOS(p) => p.stats(),
            EbpfProviderImpl::Simulation(p) => p.stats(),
        }
    }
}

impl SyscallFilterProvider for EbpfManagerImpl {
    fn add_filter(&self, filter: SyscallFilter) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.add_filter(filter),
            EbpfProviderImpl::MacOS(p) => p.add_filter(filter),
            EbpfProviderImpl::Simulation(p) => p.add_filter(filter),
        }
    }

    fn remove_filter(&self, filter_id: &str) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.remove_filter(filter_id),
            EbpfProviderImpl::MacOS(p) => p.remove_filter(filter_id),
            EbpfProviderImpl::Simulation(p) => p.remove_filter(filter_id),
        }
    }

    fn get_filters(&self) -> Vec<SyscallFilter> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.get_filters(),
            EbpfProviderImpl::MacOS(p) => p.get_filters(),
            EbpfProviderImpl::Simulation(p) => p.get_filters(),
        }
    }

    fn clear_filters(&self) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.clear_filters(),
            EbpfProviderImpl::MacOS(p) => p.clear_filters(),
            EbpfProviderImpl::Simulation(p) => p.clear_filters(),
        }
    }

    fn check_syscall(&self, pid: Pid, syscall_nr: u64) -> bool {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.check_syscall(pid, syscall_nr),
            EbpfProviderImpl::MacOS(p) => p.check_syscall(pid, syscall_nr),
            EbpfProviderImpl::Simulation(p) => p.check_syscall(pid, syscall_nr),
        }
    }
}

impl EventMonitor for EbpfManagerImpl {
    fn subscribe_syscall(&self, callback: EventCallback) -> EbpfResult<String> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.subscribe_syscall(callback),
            EbpfProviderImpl::MacOS(p) => p.subscribe_syscall(callback),
            EbpfProviderImpl::Simulation(p) => p.subscribe_syscall(callback),
        }
    }

    fn subscribe_network(&self, callback: EventCallback) -> EbpfResult<String> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.subscribe_network(callback),
            EbpfProviderImpl::MacOS(p) => p.subscribe_network(callback),
            EbpfProviderImpl::Simulation(p) => p.subscribe_network(callback),
        }
    }

    fn subscribe_file(&self, callback: EventCallback) -> EbpfResult<String> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.subscribe_file(callback),
            EbpfProviderImpl::MacOS(p) => p.subscribe_file(callback),
            EbpfProviderImpl::Simulation(p) => p.subscribe_file(callback),
        }
    }

    fn subscribe_all(&self, callback: EventCallback) -> EbpfResult<String> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.subscribe_all(callback),
            EbpfProviderImpl::MacOS(p) => p.subscribe_all(callback),
            EbpfProviderImpl::Simulation(p) => p.subscribe_all(callback),
        }
    }

    fn unsubscribe(&self, subscription_id: &str) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.unsubscribe(subscription_id),
            EbpfProviderImpl::MacOS(p) => p.unsubscribe(subscription_id),
            EbpfProviderImpl::Simulation(p) => p.unsubscribe(subscription_id),
        }
    }

    fn get_recent_events(&self, limit: usize) -> Vec<EbpfEvent> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.get_recent_events(limit),
            EbpfProviderImpl::MacOS(p) => p.get_recent_events(limit),
            EbpfProviderImpl::Simulation(p) => p.get_recent_events(limit),
        }
    }

    fn get_events_by_pid(&self, pid: Pid, limit: usize) -> Vec<EbpfEvent> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.get_events_by_pid(pid, limit),
            EbpfProviderImpl::MacOS(p) => p.get_events_by_pid(pid, limit),
            EbpfProviderImpl::Simulation(p) => p.get_events_by_pid(pid, limit),
        }
    }
}

impl ProcessMonitor for EbpfManagerImpl {
    fn monitor_process(&self, pid: Pid) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.monitor_process(pid),
            EbpfProviderImpl::MacOS(p) => p.monitor_process(pid),
            EbpfProviderImpl::Simulation(p) => p.monitor_process(pid),
        }
    }

    fn unmonitor_process(&self, pid: Pid) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.unmonitor_process(pid),
            EbpfProviderImpl::MacOS(p) => p.unmonitor_process(pid),
            EbpfProviderImpl::Simulation(p) => p.unmonitor_process(pid),
        }
    }

    fn get_monitored_pids(&self) -> Vec<Pid> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.get_monitored_pids(),
            EbpfProviderImpl::MacOS(p) => p.get_monitored_pids(),
            EbpfProviderImpl::Simulation(p) => p.get_monitored_pids(),
        }
    }

    fn get_syscall_count(&self, pid: Pid) -> u64 {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.get_syscall_count(pid),
            EbpfProviderImpl::MacOS(p) => p.get_syscall_count(pid),
            EbpfProviderImpl::Simulation(p) => p.get_syscall_count(pid),
        }
    }

    fn get_network_activity(&self, pid: Pid) -> Vec<NetworkEvent> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.get_network_activity(pid),
            EbpfProviderImpl::MacOS(p) => p.get_network_activity(pid),
            EbpfProviderImpl::Simulation(p) => p.get_network_activity(pid),
        }
    }

    fn get_file_activity(&self, pid: Pid) -> Vec<FileEvent> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.get_file_activity(pid),
            EbpfProviderImpl::MacOS(p) => p.get_file_activity(pid),
            EbpfProviderImpl::Simulation(p) => p.get_file_activity(pid),
        }
    }
}

impl EbpfManager for EbpfManagerImpl {
    fn init(&self) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.init(),
            EbpfProviderImpl::MacOS(p) => p.init(),
            EbpfProviderImpl::Simulation(p) => p.init(),
        }
    }

    fn shutdown(&self) -> EbpfResult<()> {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.shutdown(),
            EbpfProviderImpl::MacOS(p) => p.shutdown(),
            EbpfProviderImpl::Simulation(p) => p.shutdown(),
        }
    }

    fn health_check(&self) -> bool {
        match &self.provider {
            EbpfProviderImpl::Linux(p) => p.health_check(),
            EbpfProviderImpl::MacOS(p) => p.health_check(),
            EbpfProviderImpl::Simulation(p) => p.health_check(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = EbpfManagerImpl::new();
        assert!(manager.is_supported());
    }

    #[test]
    fn test_simulation_mode() {
        let manager = EbpfManagerImpl::with_simulation();
        assert_eq!(manager.platform(), EbpfPlatform::Simulation);
        assert!(manager.is_supported());
    }

    #[test]
    fn test_load_and_unload_program() {
        let manager = EbpfManagerImpl::with_simulation();

        let config = ProgramConfig {
            name: "test_program".into(),
            program_type: ProgramType::SyscallEntry,
            auto_attach: false,
            enabled: true,
        };

        assert!(manager.load_program(config).is_ok());
        assert!(manager.get_program_info("test_program").is_some());
        assert!(manager.unload_program("test_program").is_ok());
        assert!(manager.get_program_info("test_program").is_none());
    }

    #[test]
    fn test_attach_detach_program() {
        let manager = EbpfManagerImpl::with_simulation();

        let config = ProgramConfig {
            name: "test_attach".into(),
            program_type: ProgramType::NetworkSocket,
            auto_attach: false,
            enabled: true,
        };

        manager.load_program(config).unwrap();
        assert!(manager.attach_program("test_attach").is_ok());

        let info = manager.get_program_info("test_attach").unwrap();
        assert!(info.attached);

        assert!(manager.detach_program("test_attach").is_ok());
        let info = manager.get_program_info("test_attach").unwrap();
        assert!(!info.attached);
    }

    #[test]
    fn test_add_remove_filter() {
        let manager = EbpfManagerImpl::with_simulation();

        let filter = SyscallFilter {
            id: "filter1".into(),
            pid: Some(123),
            syscall_nrs: Some(vec![1, 2, 3]),
            action: FilterAction::Deny,
            priority: 100,
        };

        assert!(manager.add_filter(filter).is_ok());
        assert_eq!(manager.get_filters().len(), 1);
        assert!(manager.remove_filter("filter1").is_ok());
        assert_eq!(manager.get_filters().len(), 0);
    }

    #[test]
    fn test_check_syscall() {
        let manager = EbpfManagerImpl::with_simulation();

        // No filters - should allow
        assert!(manager.check_syscall(123, 1));

        // Add deny filter
        let filter = SyscallFilter {
            id: "deny_write".into(),
            pid: Some(123),
            syscall_nrs: Some(vec![1]), // write syscall
            action: FilterAction::Deny,
            priority: 100,
        };
        manager.add_filter(filter).unwrap();

        // Should deny
        assert!(!manager.check_syscall(123, 1));
        // Different PID should allow
        assert!(manager.check_syscall(456, 1));
        // Different syscall should allow
        assert!(manager.check_syscall(123, 2));
    }

    #[test]
    fn test_process_monitoring() {
        let manager = EbpfManagerImpl::with_simulation();

        assert!(manager.monitor_process(100).is_ok());
        assert!(manager.monitor_process(200).is_ok());

        let pids = manager.get_monitored_pids();
        assert_eq!(pids.len(), 2);
        assert!(pids.contains(&100));
        assert!(pids.contains(&200));

        assert!(manager.unmonitor_process(100).is_ok());
        assert_eq!(manager.get_monitored_pids().len(), 1);
    }

    #[test]
    fn test_manager_lifecycle() {
        let manager = EbpfManagerImpl::with_simulation();

        assert!(manager.init().is_ok());
        assert!(manager.health_check());
        assert!(manager.shutdown().is_ok());
    }
}
