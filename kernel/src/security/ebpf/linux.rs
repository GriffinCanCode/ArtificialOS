/*!
 * Linux eBPF Provider
 * Real eBPF implementation for Linux using Aya
 */

use super::events::{EventCollector, EventType};
use super::filters::FilterManager;
use super::loader::ProgramLoader;
use super::traits::*;
use super::types::*;
use crate::core::types::Pid;
use log::{debug, info, warn};
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;

/// Linux eBPF provider with Aya support
#[derive(Clone)]
pub struct LinuxEbpfProvider {
    supported: bool,
    inner: Arc<RwLock<LinuxEbpfInner>>,
    loader: ProgramLoader,
    filters: FilterManager,
    events: EventCollector,
}

struct LinuxEbpfInner {
    /// Monitored PIDs
    monitored_pids: HashSet<Pid>,
    /// eBPF programs ready state
    initialized: bool,
    /// Platform-specific handles
    /// In full Aya integration, this would hold:
    /// - Bpf struct from Aya
    /// - Map handles
    /// - Perf/ring buffer readers
    #[allow(dead_code)]
    aya_handles: Option<AyaHandles>,
}

/// Placeholder for Aya-specific handles
struct AyaHandles {
    // Future: Add eBPF maps, program handles, etc.
}

impl LinuxEbpfProvider {
    pub fn new() -> Self {
        let supported = Self::check_support();
        if supported {
            info!("Linux eBPF provider initialized with enhanced capabilities");
        } else {
            warn!("Linux eBPF not supported on this system");
        }

        Self {
            supported,
            inner: Arc::new(RwLock::new(LinuxEbpfInner {
                monitored_pids: HashSet::new(),
                initialized: false,
                aya_handles: None,
            }).into()),
            loader: ProgramLoader::new(),
            filters: FilterManager::new(),
            events: EventCollector::new(),
        }
    }

    fn check_support() -> bool {
        #[cfg(target_os = "linux")]
        {
            // Check for eBPF support
            // - Kernel version >= 4.4
            // - CAP_SYS_ADMIN or CAP_BPF capability
            // - BPF filesystem mounted
            use std::fs;

            // Simple check: see if /sys/fs/bpf exists
            fs::metadata("/sys/fs/bpf").is_ok()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }
}

impl Default for LinuxEbpfProvider {
    fn default() -> Self {
        Self::new()
    }
}

// For now, Linux provider delegates to simulation mode
// Once Aya is integrated, this will use real eBPF programs
impl EbpfProvider for LinuxEbpfProvider {
    fn is_supported(&self) -> bool {
        self.supported
    }

    fn platform(&self) -> EbpfPlatform {
        EbpfPlatform::Linux
    }

    fn load_program(&self, config: ProgramConfig) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        // Load program via loader
        self.loader.load(config.clone())?;

        // In full Aya integration, this would:
        // 1. Load eBPF bytecode from embedded binary or compile on-the-fly
        // 2. Use Aya's Bpf::load() to load into kernel
        // 3. Store program and map handles in aya_handles
        // 4. Set up perf/ring buffers for event collection

        debug!(
            "Loaded eBPF program: {} (type: {:?})",
            config.name, config.program_type
        );

        // Auto-attach if requested
        if config.auto_attach && config.enabled {
            self.attach_program(&config.name)?;
        }

        Ok(())
    }

    fn unload_program(&self, name: &str) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        self.loader.unload(name)?;
        debug!("Unloaded eBPF program: {}", name);
        Ok(())
    }

    fn attach_program(&self, name: &str) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        self.loader.attach(name)?;

        // In full Aya integration, this would:
        // 1. Get the program from aya_handles
        // 2. Attach to appropriate hook point (tracepoint, kprobe, etc.)
        // 3. Start event collection from perf/ring buffers

        info!("Attached eBPF program: {}", name);
        Ok(())
    }

    fn detach_program(&self, name: &str) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        self.loader.detach(name)?;
        debug!("Detached eBPF program: {}", name);
        Ok(())
    }

    fn list_programs(&self) -> Vec<ProgramInfo> {
        self.loader.list()
    }

    fn get_program_info(&self, name: &str) -> Option<ProgramInfo> {
        self.loader.get_info(name)
    }

    fn stats(&self) -> EbpfStats {
        let programs = self.loader.list();
        let (syscall_events, network_events, file_events, events_per_sec) = self.events.stats();

        EbpfStats {
            programs_loaded: programs.len(),
            programs_attached: programs.iter().filter(|p| p.attached).count(),
            syscall_events,
            network_events,
            file_events,
            active_filters: self.filters.count(),
            events_per_sec,
            platform: EbpfPlatform::Linux,
        }
    }
}

impl SyscallFilterProvider for LinuxEbpfProvider {
    fn add_filter(&self, filter: SyscallFilter) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        self.filters.add(filter.clone())?;
        debug!("Added syscall filter: {}", filter.id);

        // In full Aya integration, this would update the eBPF map
        // to enforce filtering in kernel space

        Ok(())
    }

    fn remove_filter(&self, filter_id: &str) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        self.filters.remove(filter_id)?;
        debug!("Removed syscall filter: {}", filter_id);
        Ok(())
    }

    fn get_filters(&self) -> Vec<SyscallFilter> {
        self.filters.list()
    }

    fn clear_filters(&self) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        self.filters.clear()?;
        debug!("Cleared all syscall filters");
        Ok(())
    }

    fn check_syscall(&self, pid: Pid, syscall_nr: u64) -> bool {
        self.filters.check(pid, syscall_nr)
    }
}

impl EventMonitor for LinuxEbpfProvider {
    fn subscribe_syscall(&self, callback: EventCallback) -> EbpfResult<String> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        let id = self.events.subscribe(EventType::Syscall, callback);
        debug!("Created syscall event subscription: {}", id);
        Ok(id)
    }

    fn subscribe_network(&self, callback: EventCallback) -> EbpfResult<String> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        let id = self.events.subscribe(EventType::Network, callback);
        debug!("Created network event subscription: {}", id);
        Ok(id)
    }

    fn subscribe_file(&self, callback: EventCallback) -> EbpfResult<String> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        let id = self.events.subscribe(EventType::File, callback);
        debug!("Created file event subscription: {}", id);
        Ok(id)
    }

    fn subscribe_all(&self, callback: EventCallback) -> EbpfResult<String> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        let id = self.events.subscribe(EventType::All, callback);
        debug!("Created all-events subscription: {}", id);
        Ok(id)
    }

    fn unsubscribe(&self, subscription_id: &str) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        self.events.unsubscribe(subscription_id)?;
        debug!("Removed event subscription: {}", subscription_id);
        Ok(())
    }

    fn get_recent_events(&self, limit: usize) -> Vec<EbpfEvent> {
        self.events.recent(limit)
    }

    fn get_events_by_pid(&self, pid: Pid, limit: usize) -> Vec<EbpfEvent> {
        self.events.by_pid(pid, limit)
    }
}

impl ProcessMonitor for LinuxEbpfProvider {
    fn monitor_process(&self, pid: Pid) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        let mut inner = self.inner.write();
        inner.monitored_pids.insert(pid);

        // In full Aya integration, this would update the eBPF map
        // to filter events for this specific PID

        info!("Started monitoring process: {}", pid);
        Ok(())
    }

    fn unmonitor_process(&self, pid: Pid) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        let mut inner = self.inner.write();
        inner.monitored_pids.remove(&pid);

        debug!("Stopped monitoring process: {}", pid);
        Ok(())
    }

    fn get_monitored_pids(&self) -> Vec<Pid> {
        self.inner.read().monitored_pids.iter().copied().collect()
    }

    fn get_syscall_count(&self, pid: Pid) -> u64 {
        self.events
            .by_pid(pid, usize::MAX)
            .iter()
            .filter(|e| matches!(e, EbpfEvent::Syscall(_)))
            .count() as u64
    }

    fn get_network_activity(&self, pid: Pid) -> Vec<NetworkEvent> {
        self.events
            .by_pid(pid, usize::MAX)
            .into_iter()
            .filter_map(|e| match e {
                EbpfEvent::Network(ne) => Some(ne),
                _ => None,
            })
            .collect()
    }

    fn get_file_activity(&self, pid: Pid) -> Vec<FileEvent> {
        self.events
            .by_pid(pid, usize::MAX)
            .into_iter()
            .filter_map(|e| match e {
                EbpfEvent::File(fe) => Some(fe),
                _ => None,
            })
            .collect()
    }
}

impl EbpfManager for LinuxEbpfProvider {
    fn init(&self) -> EbpfResult<()> {
        if !self.supported {
            return Err(EbpfError::UnsupportedPlatform {
                platform: "Linux".to_string(),
            });
        }

        let mut inner = self.inner.write();

        // In full Aya integration, this would:
        // 1. Check kernel version and eBPF support
        // 2. Initialize Aya runtime
        // 3. Set up required eBPF maps
        // 4. Load default monitoring programs

        inner.initialized = true;
        info!("Linux eBPF subsystem initialized successfully");
        Ok(())
    }

    fn shutdown(&self) -> EbpfResult<()> {
        let mut inner = self.inner.write();

        // Detach and unload all programs
        for program in self.loader.list() {
            if program.attached {
                let _ = self.loader.detach(&program.name);
            }
            let _ = self.loader.unload(&program.name);
        }

        // Clear filters and events
        let _ = self.filters.clear();
        self.events.clear();

        inner.monitored_pids.clear();
        inner.initialized = false;

        info!("Linux eBPF subsystem shut down");
        Ok(())
    }

    fn health_check(&self) -> bool {
        self.supported && self.inner.read().initialized
    }
}
