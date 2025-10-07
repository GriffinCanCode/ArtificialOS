/*!
 * Simulation eBPF Provider
 * In-memory simulation for testing and unsupported platforms
 */

use super::traits::*;
use super::types::*;
use crate::core::types::Pid;
use dashmap::DashMap;
use ahash::RandomState;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Simulation-based eBPF provider
#[derive(Clone)]
pub struct SimulationEbpfProvider {
    programs: Arc<DashMap<String, ProgramInfo, RandomState>>,
    filters: Arc<RwLock<Vec<SyscallFilter>>>,
    events: Arc<RwLock<VecDeque<EbpfEvent>>>,
    monitored_pids: Arc<DashMap<Pid, u64, RandomState>>,
    event_count: Arc<RwLock<(u64, u64, u64)>>, // (syscall, network, file)
}

impl SimulationEbpfProvider {
    pub fn new() -> Self {
        Self {
            programs: Arc::new(DashMap::with_hasher(RandomState::new())),
            filters: Arc::new(RwLock::new(Vec::new())),
            events: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            monitored_pids: Arc::new(DashMap::with_hasher(RandomState::new())),
            event_count: Arc::new(RwLock::new((0, 0, 0))),
        }
    }

    fn add_event(&self, event: EbpfEvent) {
        let mut events = self.events.write();
        if events.len() >= 1000 {
            events.pop_front();
        }
        events.push_back(event.clone());

        let mut counts = self.event_count.write();
        match event {
            EbpfEvent::Syscall(_) => counts.0 += 1,
            EbpfEvent::Network(_) => counts.1 += 1,
            EbpfEvent::File(_) => counts.2 += 1,
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

impl Default for SimulationEbpfProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EbpfProvider for SimulationEbpfProvider {
    fn is_supported(&self) -> bool {
        true
    }

    fn platform(&self) -> EbpfPlatform {
        EbpfPlatform::Simulation
    }

    fn load_program(&self, config: ProgramConfig) -> EbpfResult<()> {
        let info = ProgramInfo {
            name: config.name.clone(),
            program_type: config.program_type,
            attached: config.auto_attach,
            events_captured: 0,
            created_at: Self::current_timestamp(),
        };
        self.programs.insert(config.name, info);
        Ok(())
    }

    fn unload_program(&self, name: &str) -> EbpfResult<()> {
        self.programs
            .remove(name)
            .ok_or_else(|| EbpfError::ProgramNotFound {
                name: name.to_string(),
            })?;
        Ok(())
    }

    fn attach_program(&self, name: &str) -> EbpfResult<()> {
        if let Some(mut program) = self.programs.get_mut(name) {
            program.attached = true;
            Ok(())
        } else {
            Err(EbpfError::ProgramNotFound {
                name: name.to_string(),
            })
        }
    }

    fn detach_program(&self, name: &str) -> EbpfResult<()> {
        if let Some(mut program) = self.programs.get_mut(name) {
            program.attached = false;
            Ok(())
        } else {
            Err(EbpfError::ProgramNotFound {
                name: name.to_string(),
            })
        }
    }

    fn list_programs(&self) -> Vec<ProgramInfo> {
        self.programs
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    fn get_program_info(&self, name: &str) -> Option<ProgramInfo> {
        self.programs.get(name).map(|entry| entry.value().clone())
    }

    fn stats(&self) -> EbpfStats {
        let counts = self.event_count.read();
        let events = self.events.read();

        EbpfStats {
            programs_loaded: self.programs.len(),
            programs_attached: self
                .programs
                .iter()
                .filter(|entry| entry.value().attached)
                .count(),
            syscall_events: counts.0,
            network_events: counts.1,
            file_events: counts.2,
            active_filters: self.filters.read().len(),
            events_per_sec: 0.0, // Simulation doesn't track rate
            platform: EbpfPlatform::Simulation,
        }
    }
}

impl SyscallFilterProvider for SimulationEbpfProvider {
    fn add_filter(&self, filter: SyscallFilter) -> EbpfResult<()> {
        let mut filters = self.filters.write();
        filters.push(filter);
        filters.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(())
    }

    fn remove_filter(&self, filter_id: &str) -> EbpfResult<()> {
        let mut filters = self.filters.write();
        filters.retain(|f| f.id != filter_id);
        Ok(())
    }

    fn get_filters(&self) -> Vec<SyscallFilter> {
        self.filters.read().clone()
    }

    fn clear_filters(&self) -> EbpfResult<()> {
        self.filters.write().clear();
        Ok(())
    }

    fn check_syscall(&self, pid: Pid, syscall_nr: u64) -> bool {
        let filters = self.filters.read();

        for filter in filters.iter() {
            // Check if filter applies to this PID
            if let Some(filter_pid) = filter.pid {
                if filter_pid != pid {
                    continue;
                }
            }

            // Check if filter applies to this syscall
            if let Some(ref syscalls) = filter.syscall_nrs {
                if !syscalls.contains(&syscall_nr) {
                    continue;
                }
            }

            // Filter matches, apply action
            return match filter.action {
                FilterAction::Allow | FilterAction::Log | FilterAction::RateLimit { .. } => true,
                FilterAction::Deny => false,
            };
        }

        // Default: allow if no filter matches
        true
    }
}

impl EventMonitor for SimulationEbpfProvider {
    fn subscribe_syscall(&self, _callback: EventCallback) -> EbpfResult<String> {
        // Note: In simulation mode, callbacks are not actually invoked
        Ok(format!("sim_syscall_{}", Self::current_timestamp()))
    }

    fn subscribe_network(&self, _callback: EventCallback) -> EbpfResult<String> {
        Ok(format!("sim_network_{}", Self::current_timestamp()))
    }

    fn subscribe_file(&self, _callback: EventCallback) -> EbpfResult<String> {
        Ok(format!("sim_file_{}", Self::current_timestamp()))
    }

    fn subscribe_all(&self, _callback: EventCallback) -> EbpfResult<String> {
        Ok(format!("sim_all_{}", Self::current_timestamp()))
    }

    fn unsubscribe(&self, _subscription_id: &str) -> EbpfResult<()> {
        Ok(())
    }

    fn get_recent_events(&self, limit: usize) -> Vec<EbpfEvent> {
        let _events = self.events.read();
        // In simulation mode, return empty list (no actual events captured)
        Vec::with_capacity(std::cmp::min(limit, 0))
    }

    fn get_events_by_pid(&self, pid: Pid, limit: usize) -> Vec<EbpfEvent> {
        let events = self.events.read();
        events
            .iter()
            .rev()
            .filter(|e| e.pid() == pid)
            .take(limit)
            .cloned()
            .collect()
    }
}

impl ProcessMonitor for SimulationEbpfProvider {
    fn monitor_process(&self, pid: Pid) -> EbpfResult<()> {
        self.monitored_pids.insert(pid, 0);
        Ok(())
    }

    fn unmonitor_process(&self, pid: Pid) -> EbpfResult<()> {
        self.monitored_pids.remove(&pid);
        Ok(())
    }

    fn get_monitored_pids(&self) -> Vec<Pid> {
        self.monitored_pids
            .iter()
            .map(|entry| *entry.key())
            .collect()
    }

    fn get_syscall_count(&self, pid: Pid) -> u64 {
        self.monitored_pids.get(&pid).map(|v| *v).unwrap_or(0)
    }

    fn get_network_activity(&self, pid: Pid) -> Vec<NetworkEvent> {
        let events = self.events.read();
        events
            .iter()
            .filter_map(|e| match e {
                EbpfEvent::Network(ne) if ne.pid == pid => Some(ne.clone()),
                _ => None,
            })
            .collect()
    }

    fn get_file_activity(&self, pid: Pid) -> Vec<FileEvent> {
        let events = self.events.read();
        events
            .iter()
            .filter_map(|e| match e {
                EbpfEvent::File(fe) if fe.pid == pid => Some(fe.clone()),
                _ => None,
            })
            .collect()
    }
}

impl EbpfManager for SimulationEbpfProvider {
    fn init(&self) -> EbpfResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> EbpfResult<()> {
        self.programs.clear();
        self.filters.write().clear();
        self.events.write().clear();
        self.monitored_pids.clear();
        Ok(())
    }

    fn health_check(&self) -> bool {
        true
    }
}
