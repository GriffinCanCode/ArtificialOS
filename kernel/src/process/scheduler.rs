/*!
 * CPU Scheduler
 * Manages process scheduling with multiple policies and preemption
 */

use super::types::{ProcessStats, SchedulerStats, SchedulingPolicy};
use crate::core::types::{Pid, Priority};
use log::info;
use parking_lot::RwLock;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Process scheduling entry
#[derive(Debug, Clone)]
struct Entry {
    pid: Pid,
    priority: Priority,
    vruntime: u64, // Virtual runtime for fair scheduling (microseconds)
    last_scheduled: Option<Instant>,
    time_slice_remaining: Duration,
    cpu_time_micros: u64, // Total CPU time used by this process (microseconds)
}

impl Entry {
    fn new(pid: Pid, priority: Priority, quantum: Duration) -> Self {
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
    fn update_vruntime(&mut self, actual_runtime: Duration) {
        // Lower priority (higher number) = slower vruntime growth = more CPU time
        let weight = Self::priority_to_weight(self.priority);
        let vruntime_delta = (actual_runtime.as_micros() as u64 * 100) / weight;
        self.vruntime += vruntime_delta;
    }

    /// Convert priority to weight (higher priority = higher weight)
    fn priority_to_weight(priority: Priority) -> u64 {
        match priority {
            0..=3 => 50,   // Low priority
            4..=7 => 100,  // Normal priority
            _ => 200,      // High priority
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
        self.priority.cmp(&other.priority)
            .then_with(|| other.vruntime.cmp(&self.vruntime)) // Lower vruntime first for fairness
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// CPU Scheduler
pub struct Scheduler {
    policy: Arc<RwLock<SchedulingPolicy>>,
    quantum: Arc<RwLock<Duration>>,

    // Round-robin queue
    rr_queue: Arc<RwLock<VecDeque<Entry>>>,

    // Priority queue (min-heap by priority and vruntime)
    priority_queue: Arc<RwLock<BinaryHeap<Entry>>>,

    // Current running process
    current: Arc<RwLock<Option<Entry>>>,

    // Statistics
    stats: Arc<RwLock<SchedulerStats>>,
}

impl Scheduler {
    /// Create new scheduler with policy
    pub fn new(policy: SchedulingPolicy) -> Self {
        Self::with_quantum(policy, Duration::from_millis(10))
    }

    /// Create scheduler with custom quantum
    pub fn with_quantum(policy: SchedulingPolicy, quantum: Duration) -> Self {
        info!("Scheduler initialized: policy={:?}, quantum={:?}", policy, quantum);

        Self {
            policy: Arc::new(RwLock::new(policy)),
            quantum: Arc::new(RwLock::new(quantum)),
            rr_queue: Arc::new(RwLock::new(VecDeque::new())),
            priority_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            current: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(SchedulerStats {
                total_scheduled: 0,
                context_switches: 0,
                preemptions: 0,
                active_processes: 0,
                policy,
                quantum_micros: quantum.as_micros() as u64,
            })),
        }
    }

    /// Add process to scheduler
    pub fn add(&self, pid: Pid, priority: Priority) {
        let quantum = *self.quantum.read();
        let policy = *self.policy.read();
        let entry = Entry::new(pid, priority, quantum);

        match policy {
            SchedulingPolicy::RoundRobin => {
                self.rr_queue.write().push_back(entry);
            }
            SchedulingPolicy::Priority | SchedulingPolicy::Fair => {
                self.priority_queue.write().push(entry);
            }
        }

        self.stats.write().active_processes += 1;
        info!("Process {} added to scheduler (priority: {})", pid, priority);
    }

    /// Remove process from scheduler
    pub fn remove(&self, pid: Pid) -> bool {
        let mut removed = false;

        let policy = *self.policy.read();
        match policy {
            SchedulingPolicy::RoundRobin => {
                let mut queue = self.rr_queue.write();
                if let Some(pos) = queue.iter().position(|e| e.pid == pid) {
                    queue.remove(pos);
                    removed = true;
                }
            }
            SchedulingPolicy::Priority | SchedulingPolicy::Fair => {
                let mut queue = self.priority_queue.write();
                let entries: Vec<Entry> = queue.drain().filter(|e| e.pid != pid).collect();
                removed = entries.len() < queue.len() + 1;
                for entry in entries {
                    queue.push(entry);
                }
            }
        }

        // Check if current process is the one being removed
        let mut current = self.current.write();
        if current.as_ref().map(|e| e.pid) == Some(pid) {
            *current = None;
            removed = true;
        }

        if removed {
            self.stats.write().active_processes = self.stats.read().active_processes.saturating_sub(1);
            info!("Process {} removed from scheduler", pid);
        }

        removed
    }

    /// Schedule next process (returns None if no processes available)
    pub fn schedule(&self) -> Option<u32> {
        let mut current = self.current.write();
        let now = Instant::now();

        // Handle current process
        if let Some(ref mut entry) = *current {
            let elapsed = entry.last_scheduled.map(|t| now.duration_since(t)).unwrap_or_default();

            // Track CPU usage
            entry.cpu_time_micros += elapsed.as_micros() as u64;

            // Update virtual runtime for fair scheduling
            let policy = *self.policy.read();
            if policy == SchedulingPolicy::Fair {
                entry.update_vruntime(elapsed);
            }

            // Check if time quantum expired
            if elapsed >= entry.time_slice_remaining {
                // Preemption needed
                let preempted_pid = entry.pid;
                let mut new_entry = entry.clone();
                *current = None;

                // Re-add to queue with reset quantum
                new_entry.time_slice_remaining = *self.quantum.read();
                new_entry.last_scheduled = None;

                match policy {
                    SchedulingPolicy::RoundRobin => {
                        self.rr_queue.write().push_back(new_entry);
                    }
                    SchedulingPolicy::Priority | SchedulingPolicy::Fair => {
                        self.priority_queue.write().push(new_entry);
                    }
                }

                let mut stats = self.stats.write();
                stats.preemptions += 1;
                stats.context_switches += 1;

                info!("Process {} preempted after {:?}", preempted_pid, elapsed);
            } else {
                // Continue current process
                return Some(entry.pid);
            }
        }

        // Select next process
        let policy = *self.policy.read();
        let next = match policy {
            SchedulingPolicy::RoundRobin => self.rr_queue.write().pop_front(),
            SchedulingPolicy::Priority => self.priority_queue.write().pop(),
            SchedulingPolicy::Fair => {
                // For Fair scheduling, select process with minimum vruntime
                let mut queue = self.priority_queue.write();
                if queue.is_empty() {
                    None
                } else {
                    // Drain all entries, find minimum vruntime, and rebuild heap
                    let mut entries: Vec<Entry> = queue.drain().collect();
                    entries.sort_by(|a, b| {
                        a.vruntime.cmp(&b.vruntime)
                            .then_with(|| b.priority.cmp(&a.priority)) // Higher priority as tiebreaker
                    });
                    let selected = entries.remove(0);
                    // Put remaining entries back
                    for entry in entries {
                        queue.push(entry);
                    }
                    Some(selected)
                }
            }
        };

        if let Some(mut entry) = next {
            let pid = entry.pid;
            entry.last_scheduled = Some(now);
            entry.time_slice_remaining = *self.quantum.read();
            *current = Some(entry);

            let mut stats = self.stats.write();
            stats.total_scheduled += 1;
            if stats.total_scheduled > 0 {
                stats.context_switches += 1;
            }

            info!("Scheduled process {} ({:?})", pid, policy);
            Some(pid)
        } else {
            None
        }
    }

    /// Yield current process (voluntary context switch)
    pub fn yield_process(&self) -> Option<u32> {
        let mut current = self.current.write();

        if let Some(entry) = current.take() {
            info!("Process {} yielded voluntarily", entry.pid);

            // Re-add to queue with full quantum
            let mut new_entry = entry;
            new_entry.time_slice_remaining = *self.quantum.read();
            new_entry.last_scheduled = None;

            let policy = *self.policy.read();
            match policy {
                SchedulingPolicy::RoundRobin => {
                    self.rr_queue.write().push_back(new_entry);
                }
                SchedulingPolicy::Priority | SchedulingPolicy::Fair => {
                    self.priority_queue.write().push(new_entry);
                }
            }

            self.stats.write().context_switches += 1;
        }

        drop(current);
        self.schedule()
    }

    /// Get current running process
    pub fn current(&self) -> Option<u32> {
        self.current.read().as_ref().map(|e| e.pid)
    }

    /// Change scheduling policy (preserves processes but requeues them)
    pub fn set_policy(&self, new_policy: SchedulingPolicy) {
        let current_policy = *self.policy.read();
        if new_policy == current_policy {
            return;
        }

        info!("Changing scheduler policy from {:?} to {:?} (requeuing all processes)", current_policy, new_policy);

        // Collect all processes from current queues
        let mut all_entries = Vec::new();

        // Collect from round-robin queue
        let mut rr_queue = self.rr_queue.write();
        all_entries.extend(rr_queue.drain(..));
        drop(rr_queue);

        // Collect from priority queue
        let mut pq = self.priority_queue.write();
        all_entries.extend(pq.drain());
        drop(pq);

        // Collect current process
        let mut current = self.current.write();
        if let Some(entry) = current.take() {
            all_entries.push(entry);
        }
        drop(current);

        // Update policy
        *self.policy.write() = new_policy;
        self.stats.write().policy = new_policy;

        // Requeue all processes under new policy
        for entry in all_entries {
            match new_policy {
                SchedulingPolicy::RoundRobin => {
                    self.rr_queue.write().push_back(entry);
                }
                SchedulingPolicy::Priority | SchedulingPolicy::Fair => {
                    self.priority_queue.write().push(entry);
                }
            }
        }

        info!("Policy change complete: {} processes requeued", self.len());
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> SchedulerStats {
        self.stats.read().clone()
    }

    /// Get per-process CPU usage statistics
    pub fn process_stats(&self, pid: Pid) -> Option<ProcessStats> {
        // Check current process
        let current = self.current.read();
        if let Some(ref entry) = *current {
            if entry.pid == pid {
                return Some(ProcessStats {
                    pid: entry.pid,
                    priority: entry.priority,
                    cpu_time_micros: entry.cpu_time_micros,
                    vruntime: entry.vruntime,
                    is_current: true,
                });
            }
        }
        drop(current);

        // Search in queues
        let policy = *self.policy.read();
        match policy {
            SchedulingPolicy::RoundRobin => {
                let queue = self.rr_queue.read();
                queue.iter().find(|e| e.pid == pid).map(|entry| ProcessStats {
                    pid: entry.pid,
                    priority: entry.priority,
                    cpu_time_micros: entry.cpu_time_micros,
                    vruntime: entry.vruntime,
                    is_current: false,
                })
            }
            SchedulingPolicy::Priority | SchedulingPolicy::Fair => {
                let queue = self.priority_queue.read();
                queue.iter().find(|e| e.pid == pid).map(|entry| ProcessStats {
                    pid: entry.pid,
                    priority: entry.priority,
                    cpu_time_micros: entry.cpu_time_micros,
                    vruntime: entry.vruntime,
                    is_current: false,
                })
            }
        }
    }

    /// Get all process statistics
    pub fn all_process_stats(&self) -> Vec<ProcessStats> {
        let mut stats = Vec::new();

        // Get current process
        let current = self.current.read();
        if let Some(ref entry) = *current {
            stats.push(ProcessStats {
                pid: entry.pid,
                priority: entry.priority,
                cpu_time_micros: entry.cpu_time_micros,
                vruntime: entry.vruntime,
                is_current: true,
            });
        }
        drop(current);

        // Get queued processes
        let policy = *self.policy.read();
        match policy {
            SchedulingPolicy::RoundRobin => {
                let queue = self.rr_queue.read();
                stats.extend(queue.iter().map(|entry| ProcessStats {
                    pid: entry.pid,
                    priority: entry.priority,
                    cpu_time_micros: entry.cpu_time_micros,
                    vruntime: entry.vruntime,
                    is_current: false,
                }));
            }
            SchedulingPolicy::Priority | SchedulingPolicy::Fair => {
                let queue = self.priority_queue.read();
                stats.extend(queue.iter().map(|entry| ProcessStats {
                    pid: entry.pid,
                    priority: entry.priority,
                    cpu_time_micros: entry.cpu_time_micros,
                    vruntime: entry.vruntime,
                    is_current: false,
                }));
            }
        }

        stats
    }

    /// Get current scheduling policy
    pub fn policy(&self) -> SchedulingPolicy {
        *self.policy.read()
    }

    /// Get number of processes in scheduler
    pub fn len(&self) -> usize {
        let policy = *self.policy.read();
        let queue_len = match policy {
            SchedulingPolicy::RoundRobin => self.rr_queue.read().len(),
            SchedulingPolicy::Priority | SchedulingPolicy::Fair => self.priority_queue.read().len(),
        };
        queue_len + if self.current.read().is_some() { 1 } else { 0 }
    }

    /// Check if scheduler is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Clone for Scheduler {
    fn clone(&self) -> Self {
        Self {
            policy: Arc::clone(&self.policy),
            quantum: Arc::clone(&self.quantum),
            rr_queue: Arc::clone(&self.rr_queue),
            priority_queue: Arc::clone(&self.priority_queue),
            current: Arc::clone(&self.current),
            stats: Arc::clone(&self.stats),
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

        scheduler.add(1, 3);  // Low priority
        scheduler.add(2, 8);  // High priority
        scheduler.add(3, 5);  // Medium priority

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
        let scheduler = Scheduler::with_quantum(SchedulingPolicy::RoundRobin, Duration::from_millis(10));

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
