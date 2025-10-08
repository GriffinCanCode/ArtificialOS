/*!
 * Resource Budget Tracking
 * Per-process resource quotas and usage monitoring
 */

use crate::core::types::Pid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resource budget limits per process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub max_memory_bytes: Option<usize>,
    pub max_file_descriptors: Option<usize>,
    pub max_sockets: Option<usize>,
    pub max_mappings: Option<usize>,
    pub max_ipc_queues: Option<usize>,
    pub max_async_tasks: Option<usize>,
}

impl ResourceBudget {
    /// Create unlimited budget
    pub fn unlimited() -> Self {
        Self {
            max_memory_bytes: None,
            max_file_descriptors: None,
            max_sockets: None,
            max_mappings: None,
            max_ipc_queues: None,
            max_async_tasks: None,
        }
    }

    /// Create standard budget for typical processes
    pub fn standard() -> Self {
        use crate::core::limits::*;
        Self {
            max_memory_bytes: Some(STANDARD_PROCESS_MEMORY),
            max_file_descriptors: Some(STANDARD_MAX_FILE_DESCRIPTORS),
            max_sockets: Some(MAX_SOCKETS),
            max_mappings: Some(500),
            max_ipc_queues: Some(50),
            max_async_tasks: Some(1000),
        }
    }

    /// Create restricted budget for low-priority processes
    pub fn restricted() -> Self {
        use crate::core::limits::*;
        Self {
            max_memory_bytes: Some(RESTRICTED_PROCESS_MEMORY),
            max_file_descriptors: Some(RESTRICTED_MAX_FILE_DESCRIPTORS),
            max_sockets: Some(20),
            max_mappings: Some(MAX_MEMORY_MAPPINGS),
            max_ipc_queues: Some(10),
            max_async_tasks: Some(MAX_ASYNC_TASKS),
        }
    }

    /// Check if usage exceeds budget
    pub fn check_limit(&self, resource_type: &str, current: usize) -> bool {
        match resource_type {
            "memory" => self.max_memory_bytes.map_or(true, |max| current < max),
            "file_descriptors" => self.max_file_descriptors.map_or(true, |max| current < max),
            "sockets" => self.max_sockets.map_or(true, |max| current < max),
            "mappings" => self.max_mappings.map_or(true, |max| current < max),
            "ipc" => self.max_ipc_queues.map_or(true, |max| current < max),
            "async_tasks" => self.max_async_tasks.map_or(true, |max| current < max),
            _ => true, // Unknown resource types are unlimited
        }
    }
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self::standard()
    }
}

/// Current resource usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub memory_bytes: usize,
    pub file_descriptors: usize,
    pub sockets: usize,
    pub mappings: usize,
    pub ipc_queues: usize,
    pub async_tasks: usize,
}

impl ResourceUsage {
    /// Create from cleanup stats
    pub fn from_cleanup_stats(by_type: &HashMap<String, usize>) -> Self {
        Self {
            memory_bytes: *by_type.get("memory").unwrap_or(&0),
            file_descriptors: *by_type.get("file_descriptors").unwrap_or(&0),
            sockets: *by_type.get("sockets").unwrap_or(&0),
            mappings: *by_type.get("mappings").unwrap_or(&0),
            ipc_queues: *by_type.get("ipc").unwrap_or(&0),
            async_tasks: *by_type.get("async_tasks").unwrap_or(&0),
        }
    }

    /// Calculate usage percentage against budget
    pub fn usage_percent(&self, budget: &ResourceBudget) -> HashMap<String, f64> {
        let mut percentages = HashMap::new();

        if let Some(max) = budget.max_memory_bytes {
            percentages.insert(
                "memory".into(),
                (self.memory_bytes as f64 / max as f64) * 100.0,
            );
        }

        if let Some(max) = budget.max_file_descriptors {
            percentages.insert(
                "file_descriptors".into(),
                (self.file_descriptors as f64 / max as f64) * 100.0,
            );
        }

        if let Some(max) = budget.max_sockets {
            percentages.insert("sockets".into(), (self.sockets as f64 / max as f64) * 100.0);
        }

        if let Some(max) = budget.max_mappings {
            percentages.insert(
                "mappings".into(),
                (self.mappings as f64 / max as f64) * 100.0,
            );
        }

        if let Some(max) = budget.max_ipc_queues {
            percentages.insert("ipc".into(), (self.ipc_queues as f64 / max as f64) * 100.0);
        }

        if let Some(max) = budget.max_async_tasks {
            percentages.insert(
                "async_tasks".into(),
                (self.async_tasks as f64 / max as f64) * 100.0,
            );
        }

        percentages
    }

    /// Check if any resource is near limit (>80%)
    pub fn near_limit(&self, budget: &ResourceBudget) -> Vec<String> {
        self.usage_percent(budget)
            .iter()
            .filter(|(_, pct)| **pct > 80.0)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

/// Per-process resource tracker
#[derive(Debug)]
pub struct ResourceTracker {
    budgets: HashMap<Pid, ResourceBudget>,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            budgets: HashMap::new(),
        }
    }

    /// Set budget for a process
    pub fn set_budget(&mut self, pid: Pid, budget: ResourceBudget) {
        self.budgets.insert(pid, budget);
    }

    /// Get budget for a process
    pub fn get_budget(&self, pid: Pid) -> ResourceBudget {
        self.budgets
            .get(&pid)
            .cloned()
            .unwrap_or_else(ResourceBudget::standard)
    }

    /// Remove budget tracking for terminated process
    pub fn remove(&mut self, pid: Pid) {
        self.budgets.remove(&pid);
    }
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_creation() {
        let unlimited = ResourceBudget::unlimited();
        assert!(unlimited.max_memory_bytes.is_none());

        let standard = ResourceBudget::standard();
        assert_eq!(standard.max_memory_bytes, Some(1024 * 1024 * 1024));

        let restricted = ResourceBudget::restricted();
        assert_eq!(restricted.max_memory_bytes, Some(256 * 1024 * 1024));
    }

    #[test]
    fn test_budget_limits() {
        let budget = ResourceBudget::standard();

        // Under limit
        assert!(budget.check_limit("memory", 500 * 1024 * 1024));

        // Over limit
        assert!(!budget.check_limit("memory", 2 * 1024 * 1024 * 1024));
    }

    #[test]
    fn test_usage_percentages() {
        let budget = ResourceBudget::standard();
        let mut usage = ResourceUsage::default();
        usage.memory_bytes = 512 * 1024 * 1024; // 50% of 1GB

        let percentages = usage.usage_percent(&budget);
        assert_eq!(percentages.get("memory"), Some(&50.0));
    }

    #[test]
    fn test_near_limit_detection() {
        let budget = ResourceBudget::standard();
        let mut usage = ResourceUsage::default();
        usage.file_descriptors = 900; // 90% of 1024

        let near_limits = usage.near_limit(&budget);
        assert!(near_limits.contains(&"file_descriptors".to_string().into()));
    }
}
