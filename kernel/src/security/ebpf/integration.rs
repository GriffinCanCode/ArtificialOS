/*!
 * eBPF Integration
 * Integration helpers for eBPF with existing monitoring and security systems
 */

use super::manager::EbpfManagerImpl;
use super::traits::*;
use super::types::*;
use crate::core::types::Pid;
use crate::monitoring::MetricsCollector;
use crate::security::sandbox::SandboxManager;
use crate::security::types::Capability;
use log::{debug, info, warn};
use std::sync::Arc;

/// Integrated eBPF monitor that works with existing systems
pub struct IntegratedEbpfMonitor {
    ebpf: EbpfManagerImpl,
    metrics: Option<Arc<MetricsCollector>>,
    sandbox: Option<SandboxManager>,
}

impl IntegratedEbpfMonitor {
    /// Create a new integrated monitor
    pub fn new(ebpf: EbpfManagerImpl) -> Self {
        Self {
            ebpf,
            metrics: None,
            sandbox: None,
        }
    }

    /// Add metrics collector integration
    pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Add sandbox manager integration
    pub fn with_sandbox(mut self, sandbox: SandboxManager) -> Self {
        self.sandbox = Some(sandbox);
        self
    }

    /// Initialize the integrated monitor
    pub fn init(&self) -> EbpfResult<()> {
        info!("Initializing integrated eBPF monitor");

        // Initialize eBPF
        self.ebpf.init()?;

        // Subscribe to events for metrics
        if let Some(ref metrics) = self.metrics {
            let metrics_clone = Arc::clone(metrics);
            self.ebpf.subscribe_all(Box::new(move |event| {
                Self::handle_event_metrics(&metrics_clone, &event);
            }))?;
            info!("eBPF metrics integration enabled");
        }

        // Load default monitoring programs
        self.load_default_programs()?;

        info!("Integrated eBPF monitor initialized");
        Ok(())
    }

    /// Shutdown the monitor
    pub fn shutdown(&self) -> EbpfResult<()> {
        info!("Shutting down integrated eBPF monitor");
        self.ebpf.shutdown()
    }

    /// Load default monitoring programs
    fn load_default_programs(&self) -> EbpfResult<()> {
        let programs = vec![
            ProgramConfig {
                name: "syscall_entry".to_string(),
                program_type: ProgramType::SyscallEntry,
                auto_attach: true,
                enabled: true,
            },
            ProgramConfig {
                name: "syscall_exit".to_string(),
                program_type: ProgramType::SyscallExit,
                auto_attach: true,
                enabled: true,
            },
            ProgramConfig {
                name: "network_socket".to_string(),
                program_type: ProgramType::NetworkSocket,
                auto_attach: true,
                enabled: true,
            },
            ProgramConfig {
                name: "file_ops".to_string(),
                program_type: ProgramType::FileOps,
                auto_attach: true,
                enabled: true,
            },
        ];

        for config in programs {
            if let Err(e) = self.ebpf.load_program(config.clone()) {
                warn!("Failed to load program {}: {:?}", config.name, e);
            } else {
                debug!("Loaded eBPF program: {}", config.name);
            }
        }

        Ok(())
    }

    /// Handle event and update metrics
    fn handle_event_metrics(metrics: &MetricsCollector, event: &EbpfEvent) {
        match event {
            EbpfEvent::Syscall(e) => {
                metrics.inc_counter("ebpf.syscalls.total", 1.0);
                if let Some(name) = &e.name {
                    metrics.inc_counter(&format!("ebpf.syscalls.{}", name), 1.0);
                }
            }
            EbpfEvent::Network(e) => {
                metrics.inc_counter("ebpf.network.total", 1.0);
                metrics.inc_counter(&format!("ebpf.network.{:?}", e.event_type), 1.0);
            }
            EbpfEvent::File(e) => {
                metrics.inc_counter("ebpf.files.total", 1.0);
                metrics.inc_counter(&format!("ebpf.files.{:?}", e.operation), 1.0);
            }
        }
    }

    /// Sync sandbox policies to eBPF filters
    pub fn sync_sandbox_policies(&self, pid: Pid) -> EbpfResult<()> {
        if let Some(ref sandbox) = self.sandbox {
            // Check file write capability
            if !sandbox.check_permission(pid, &Capability::WriteFile(None)) {
                // Block write syscall
                self.ebpf.add_filter(SyscallFilter {
                    id: format!("sandbox_block_write_{}", pid),
                    pid: Some(pid),
                    syscall_nrs: Some(vec![1]), // write
                    action: FilterAction::Deny,
                    priority: 1000,
                })?;
                debug!("Added eBPF filter to block write for PID {}", pid);
            }

            // Check network capability (simplified check)
            use crate::security::NetworkRule;
            let network_cap = Capability::NetworkAccess(NetworkRule::AllowAll);
            if !sandbox.check_permission(pid, &network_cap) {
                // Block network syscalls
                self.ebpf.add_filter(SyscallFilter {
                    id: format!("sandbox_block_network_{}", pid),
                    pid: Some(pid),
                    syscall_nrs: Some(vec![41, 42, 43, 44, 45]), // socket, connect, accept, sendto, recvfrom
                    action: FilterAction::Deny,
                    priority: 1000,
                })?;
                debug!("Added eBPF filter to block network for PID {}", pid);
            }
        }

        Ok(())
    }

    /// Start monitoring a process with full integration
    pub fn monitor_process(&self, pid: Pid) -> EbpfResult<()> {
        // Start eBPF monitoring
        self.ebpf.monitor_process(pid)?;

        // Sync sandbox policies
        self.sync_sandbox_policies(pid)?;

        info!("Started integrated monitoring for PID {}", pid);
        Ok(())
    }

    /// Stop monitoring a process
    pub fn unmonitor_process(&self, pid: Pid) -> EbpfResult<()> {
        // Remove filters
        self.ebpf
            .remove_filter(&format!("sandbox_block_write_{}", pid))
            .ok();
        self.ebpf
            .remove_filter(&format!("sandbox_block_network_{}", pid))
            .ok();

        // Stop eBPF monitoring
        self.ebpf.unmonitor_process(pid)?;

        info!("Stopped integrated monitoring for PID {}", pid);
        Ok(())
    }

    /// Get comprehensive process statistics
    pub fn get_process_stats(&self, pid: Pid) -> ProcessEbpfStats {
        ProcessEbpfStats {
            pid,
            syscall_count: self.ebpf.get_syscall_count(pid),
            network_events: self.ebpf.get_network_activity(pid).len() as u64,
            file_events: self.ebpf.get_file_activity(pid).len() as u64,
            active_filters: self
                .ebpf
                .get_filters()
                .iter()
                .filter(|f| f.pid == Some(pid))
                .count(),
        }
    }

    /// Get the underlying eBPF manager
    pub fn ebpf(&self) -> &EbpfManagerImpl {
        &self.ebpf
    }

    /// Get overall statistics
    pub fn stats(&self) -> EbpfStats {
        self.ebpf.stats()
    }
}

/// Process-specific eBPF statistics
#[derive(Debug, Clone)]
pub struct ProcessEbpfStats {
    pub pid: Pid,
    pub syscall_count: u64,
    pub network_events: u64,
    pub file_events: u64,
    pub active_filters: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrated_monitor_creation() {
        let ebpf = EbpfManagerImpl::with_simulation();
        let monitor = IntegratedEbpfMonitor::new(ebpf);
        assert!(monitor.init().is_ok());
        assert!(monitor.shutdown().is_ok());
    }

    #[test]
    fn test_monitor_with_metrics() {
        let ebpf = EbpfManagerImpl::with_simulation();
        let metrics = Arc::new(MetricsCollector::new());
        let monitor = IntegratedEbpfMonitor::new(ebpf).with_metrics(metrics);

        assert!(monitor.init().is_ok());
        assert!(monitor.shutdown().is_ok());
    }

    #[test]
    fn test_process_monitoring() {
        let ebpf = EbpfManagerImpl::with_simulation();
        let monitor = IntegratedEbpfMonitor::new(ebpf);
        monitor.init().unwrap();

        assert!(monitor.monitor_process(123).is_ok());

        let stats = monitor.get_process_stats(123);
        assert_eq!(stats.pid, 123);

        assert!(monitor.unmonitor_process(123).is_ok());
        monitor.shutdown().unwrap();
    }

    #[test]
    fn test_sandbox_sync() {
        let ebpf = EbpfManagerImpl::with_simulation();
        let sandbox = SandboxManager::new();
        let monitor = IntegratedEbpfMonitor::new(ebpf).with_sandbox(sandbox);

        monitor.init().unwrap();

        // This should add filters based on sandbox policies
        assert!(monitor.sync_sandbox_policies(123).is_ok());

        monitor.shutdown().unwrap();
    }
}
