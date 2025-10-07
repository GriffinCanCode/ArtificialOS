/*!
 * Simulation Network Isolation
 * Fallback implementation using capability-based restrictions
 */

use super::traits::*;
use super::types::*;
use crate::core::types::Pid;
use dashmap::DashMap;
use log::info;
use std::sync::Arc;

/// Simulation-based namespace manager
/// Provides API compatibility without true OS-level isolation
pub struct SimulationNamespaceManager {
    namespaces: Arc<DashMap<NamespaceId, NamespaceInfo>>,
    pid_to_ns: Arc<DashMap<Pid, NamespaceId>>,
}

impl SimulationNamespaceManager {
    pub fn new() -> Self {
        info!("Network isolation manager initialized (simulation mode)");
        Self {
            namespaces: Arc::new(DashMap::new()),
            pid_to_ns: Arc::new(DashMap::new()),
        }
    }
}

impl Default for SimulationNamespaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NamespaceProvider for SimulationNamespaceManager {
    fn create(&self, config: NamespaceConfig) -> NamespaceResult<()> {
        let ns_name = config.id.as_str();

        info!(
            "Creating simulated network namespace: {} for PID {}",
            ns_name, config.pid
        );

        let info = NamespaceInfo {
            config: config.clone(),
            stats: Some(NamespaceStats {
                id: config.id.clone(),
                interface_count: 1,
                tx_bytes: 0,
                rx_bytes: 0,
                tx_packets: 0,
                rx_packets: 0,
                created_at: std::time::SystemTime::now(),
            }),
            platform: PlatformType::Simulation,
        };

        self.namespaces.insert(config.id.clone(), info);
        self.pid_to_ns.insert(config.pid, config.id.clone());

        Ok(())
    }

    fn destroy(&self, id: &NamespaceId) -> NamespaceResult<()> {
        if let Some((_, info)) = self.namespaces.remove(id) {
            self.pid_to_ns.remove(&info.config.pid);
            info!("Destroyed simulated network namespace: {}", id);
        }
        Ok(())
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
        self.namespaces
            .get(id)
            .and_then(|info| info.stats.clone())
    }

    fn is_supported(&self) -> bool {
        true // Always available as fallback
    }

    fn platform(&self) -> PlatformType {
        PlatformType::Simulation
    }
}

impl Clone for SimulationNamespaceManager {
    fn clone(&self) -> Self {
        Self {
            namespaces: Arc::clone(&self.namespaces),
            pid_to_ns: Arc::clone(&self.pid_to_ns),
        }
    }
}
