/*!
 * Scheduler Entry Types
 * Internal data structures for process scheduling entries
 */

use crate::core::types::{Pid, Priority};
use std::time::{Duration, Instant};

/// Process scheduling entry
#[derive(Debug, Clone)]
pub(super) struct Entry {
    pub pid: Pid,
    pub priority: Priority,
    pub vruntime: u64, // Virtual runtime for fair scheduling (microseconds)
    pub last_scheduled: Option<Instant>,
    pub time_slice_remaining: Duration,
    pub cpu_time_micros: u64, // Total CPU time used by this process (microseconds)
}

impl Entry {
    pub fn new(pid: Pid, priority: Priority, quantum: Duration) -> Self {
        Self {
            pid,
            priority,
            vruntime: 0,
            last_scheduled: None,
            time_slice_remaining: quantum,
            cpu_time_micros: 0,
        }
    }

    /// Update virtual runtime based on actual runtime and priority
    pub fn update_vruntime(&mut self, actual_runtime: Duration) {
        // Lower priority (higher number) = slower vruntime growth = more CPU time
        let weight = Self::priority_to_weight(self.priority);
        let vruntime_delta = (actual_runtime.as_micros() as u64 * 100) / weight;
        self.vruntime += vruntime_delta;
    }

    /// Convert priority to weight (higher priority = higher weight)
    fn priority_to_weight(priority: Priority) -> u64 {
        match priority {
            0..=3 => 50,  // Low priority
            4..=7 => 100, // Normal priority
            _ => 200,     // High priority
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.pid == other.pid
    }
}

impl Eq for Entry {}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // BinaryHeap is a max-heap, so higher priority values are scheduled first
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.vruntime.cmp(&self.vruntime)) // Lower vruntime first for fairness
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Wrapper for Fair scheduling that compares by vruntime (min-heap)
#[derive(Debug, Clone)]
pub(super) struct FairEntry(pub Entry);

impl PartialEq for FairEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0.pid == other.0.pid
    }
}

impl Eq for FairEntry {}

impl Ord for FairEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare by vruntime (lower is better), then by priority (higher is better)
        other
            .0
            .vruntime
            .cmp(&self.0.vruntime)
            .then_with(|| self.0.priority.cmp(&other.0.priority))
    }
}

impl PartialOrd for FairEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
