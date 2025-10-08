/*!
 * CPU Scheduler
 * Manages process scheduling with multiple policies and preemption
 */

use super::atomic_stats::AtomicSchedulerStats;
use super::types::SchedulingPolicy;
use crate::core::types::Pid;
use crate::monitoring::Collector;
use dashmap::DashMap;
use log::info;
use parking_lot::RwLock;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

mod entry;
mod operations;
mod policy;
mod stats;

use entry::{Entry, FairEntry};

/// Location of a process in the scheduler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueueLocation {
    Current,
    RoundRobin,
    Priority,
    Fair,
}

/// CPU Scheduler
///
/// # Performance
/// - Cache-line aligned for optimal performance in high-frequency scheduling operations
/// - Lock-free atomic stats for zero-contention monitoring in hot scheduling paths
#[repr(C, align(64))]
pub struct Scheduler {
    policy: Arc<RwLock<SchedulingPolicy>>,
    quantum: Arc<RwLock<Duration>>,

    // Round-robin queue
    rr_queue: Arc<RwLock<VecDeque<Entry>>>,

    // Priority queue (max-heap by priority)
    priority_queue: Arc<RwLock<BinaryHeap<Entry>>>,

    // Fair queue (min-heap by vruntime) - O(log n) operations
    fair_queue: Arc<RwLock<BinaryHeap<FairEntry>>>,

    // Current running process
    current: Arc<RwLock<Option<Entry>>>,

    // Process location index for O(1) lookup
    process_locations: Arc<DashMap<Pid, QueueLocation>>,

    // Statistics - lock-free atomics for hot path updates
    stats: Arc<AtomicSchedulerStats>,

    // Observability collector for event streaming
    collector: Option<Arc<Collector>>,
}

impl Scheduler {
    /// Create new scheduler with policy
    pub fn new(policy: SchedulingPolicy) -> Self {
        Self::with_quantum(policy, Duration::from_millis(10))
    }

    /// Create scheduler with custom quantum
    pub fn with_quantum(policy: SchedulingPolicy, quantum: Duration) -> Self {
        info!(
            "Scheduler initialized with lock-free atomic stats: policy={:?}, quantum={:?}",
            policy, quantum
        );

        Self {
            policy: Arc::new(RwLock::new(policy)),
            quantum: Arc::new(RwLock::new(quantum)),
            rr_queue: Arc::new(RwLock::new(VecDeque::new())),
            priority_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            fair_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            current: Arc::new(RwLock::new(None)),
            process_locations: Arc::new(DashMap::new()),
            stats: Arc::new(AtomicSchedulerStats::new(policy, quantum)),
            collector: None,
        }
    }

    /// Add observability collector
    pub fn with_collector(mut self, collector: Arc<Collector>) -> Self {
        self.collector = Some(collector);
        self
    }

    /// Set collector after construction
    pub fn set_collector(&mut self, collector: Arc<Collector>) {
        self.collector = Some(collector);
    }
}

impl Clone for Scheduler {
    fn clone(&self) -> Self {
        Self {
            policy: Arc::clone(&self.policy),
            quantum: Arc::clone(&self.quantum),
            rr_queue: Arc::clone(&self.rr_queue),
            priority_queue: Arc::clone(&self.priority_queue),
            fair_queue: Arc::clone(&self.fair_queue),
            current: Arc::clone(&self.current),
            process_locations: Arc::clone(&self.process_locations),
            stats: Arc::clone(&self.stats),
            collector: self.collector.as_ref().map(Arc::clone),
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new(SchedulingPolicy::Fair)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_round_robin_basic() {
        let scheduler = Scheduler::new(SchedulingPolicy::RoundRobin);

        scheduler.add(1, 5);
        scheduler.add(2, 5);
        scheduler.add(3, 5);

        assert_eq!(scheduler.len(), 3);

        // Should schedule in FIFO order
        assert_eq!(scheduler.schedule(), Some(1));
        assert_eq!(scheduler.current(), Some(1));
    }

    #[test]
    fn test_priority_scheduling() {
        let scheduler = Scheduler::new(SchedulingPolicy::Priority);

        scheduler.add(1, 3); // Low priority
        scheduler.add(2, 8); // High priority
        scheduler.add(3, 5); // Medium priority

        // Should schedule highest priority first
        assert_eq!(scheduler.schedule(), Some(2));
    }

    #[test]
    fn test_remove_process() {
        let scheduler = Scheduler::new(SchedulingPolicy::RoundRobin);

        scheduler.add(1, 5);
        scheduler.add(2, 5);
        assert_eq!(scheduler.len(), 2);

        assert!(scheduler.remove(1));
        assert_eq!(scheduler.len(), 1);

        assert!(!scheduler.remove(999)); // Non-existent
    }

    #[test]
    fn test_yield_process() {
        let scheduler = Scheduler::new(SchedulingPolicy::RoundRobin);

        scheduler.add(1, 5);
        scheduler.add(2, 5);

        assert_eq!(scheduler.schedule(), Some(1));
        assert_eq!(scheduler.yield_process(), Some(2));
        assert_eq!(scheduler.current(), Some(2));
    }

    #[test]
    fn test_empty_scheduler() {
        let scheduler = Scheduler::new(SchedulingPolicy::RoundRobin);
        assert!(scheduler.is_empty());
        assert_eq!(scheduler.schedule(), None);
    }

    #[test]
    fn test_statistics() {
        let scheduler = Scheduler::new(SchedulingPolicy::Priority);

        scheduler.add(1, 5);
        scheduler.add(2, 3);

        scheduler.schedule();
        scheduler.schedule();

        let stats = scheduler.stats();
        assert!(stats.total_scheduled > 0);
        assert_eq!(stats.policy, SchedulingPolicy::Priority);
    }

    #[test]
    fn test_policy_change() {
        let scheduler = Scheduler::new(SchedulingPolicy::RoundRobin);

        scheduler.add(1, 5);
        scheduler.add(2, 5);

        scheduler.set_policy(SchedulingPolicy::Priority);
        assert_eq!(scheduler.len(), 2); // Processes should be requeued
    }

    #[test]
    fn test_preemption_with_quantum() {
        let scheduler =
            Scheduler::with_quantum(SchedulingPolicy::RoundRobin, Duration::from_millis(10));

        scheduler.add(1, 5);
        scheduler.add(2, 5);

        // Schedule first process
        assert_eq!(scheduler.schedule(), Some(1));

        // Wait for quantum to expire
        thread::sleep(Duration::from_millis(15));

        // Next schedule should preempt and switch
        let next = scheduler.schedule();
        assert_eq!(next, Some(2));
    }
}
