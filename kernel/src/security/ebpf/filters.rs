/*!
 * Syscall Filters
 * Fast syscall filtering with priority-based evaluation
 */

use super::types::*;
use crate::core::types::Pid;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Filter manager for syscall filtering
pub struct FilterManager {
    filters: Arc<RwLock<Vec<SyscallFilter>>>,
    cache: Arc<RwLock<HashMap<(Pid, u64), FilterAction>>>,
}

impl FilterManager {
    pub fn new() -> Self {
        Self {
            filters: Arc::new(RwLock::new(Vec::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a filter
    pub fn add(&self, filter: SyscallFilter) -> EbpfResult<()> {
        let mut filters = self.filters.write();

        // Check for duplicate ID
        if filters.iter().any(|f| f.id == filter.id) {
            return Err(EbpfError::InvalidFilter {
                reason: format!("Filter {} already exists", filter.id),
            });
        }

        filters.push(filter);

        // Sort by priority (highest first)
        filters.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Clear cache when filters change
        self.cache.write().clear();

        Ok(())
    }

    /// Remove a filter
    pub fn remove(&self, filter_id: &str) -> EbpfResult<()> {
        let mut filters = self.filters.write();

        let initial_len = filters.len();
        filters.retain(|f| f.id != filter_id);

        if filters.len() == initial_len {
            return Err(EbpfError::InvalidFilter {
                reason: format!("Filter {} not found", filter_id),
            });
        }

        // Clear cache when filters change
        self.cache.write().clear();

        Ok(())
    }

    /// Get all filters
    pub fn list(&self) -> Vec<SyscallFilter> {
        self.filters.read().clone()
    }

    /// Clear all filters
    pub fn clear(&self) -> EbpfResult<()> {
        self.filters.write().clear();
        self.cache.write().clear();
        Ok(())
    }

    /// Check if a syscall would be allowed
    pub fn check(&self, pid: Pid, syscall_nr: u64) -> bool {
        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(action) = cache.get(&(pid, syscall_nr)) {
                return matches!(action, FilterAction::Allow | FilterAction::Log);
            }
        }

        // Evaluate filters
        let filters = self.filters.read();
        let action = self.evaluate_filters(&filters, pid, syscall_nr);

        // Cache the result
        self.cache.write().insert((pid, syscall_nr), action);

        matches!(action, FilterAction::Allow | FilterAction::Log)
    }

    /// Evaluate filters for a syscall
    fn evaluate_filters(
        &self,
        filters: &[SyscallFilter],
        pid: Pid,
        syscall_nr: u64,
    ) -> FilterAction {
        for filter in filters {
            // Check PID match
            if let Some(filter_pid) = filter.pid {
                if filter_pid != pid {
                    continue;
                }
            }

            // Check syscall number match
            if let Some(ref syscall_nrs) = filter.syscall_nrs {
                if !syscall_nrs.contains(&syscall_nr) {
                    continue;
                }
            }

            // Filter matches - return action
            return filter.action;
        }

        // Default action is to allow
        FilterAction::Allow
    }

    /// Get filter count
    pub fn count(&self) -> usize {
        self.filters.read().len()
    }
}

impl Default for FilterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for FilterManager {
    fn clone(&self) -> Self {
        Self {
            filters: Arc::clone(&self.filters),
            cache: Arc::clone(&self.cache),
        }
    }
}
