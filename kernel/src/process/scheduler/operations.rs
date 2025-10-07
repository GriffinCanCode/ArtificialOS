/*!
 * Scheduler Core Operations
 * Add, remove, schedule, and yield process operations
 */

use super::entry::{Entry, FairEntry};
use super::Scheduler;
use crate::core::types::{Pid, Priority};
use crate::process::types::{ProcessStats, SchedulingPolicy};
use log::info;
use std::time::Instant;

impl Scheduler {
    /// Add process to scheduler
    pub fn add(&self, pid: Pid, priority: Priority) {
        let quantum = *self.quantum.read();
        let policy = *self.policy.read();
        let mut entry = Entry::new(pid, priority, quantum);

        // For fair scheduling, initialize vruntime to min_vruntime to prevent starvation
        if policy == SchedulingPolicy::Fair {
            let mut min_vrt = u64::MAX;

            // Check current process vruntime
            if let Some(ref current_entry) = *self.current.read() {
                min_vrt = min_vrt.min(current_entry.vruntime);
            }

            // Check queued processes vruntime
            let queue = self.fair_queue.read();
            for e in queue.iter() {
                min_vrt = min_vrt.min(e.0.vruntime);
            }
            drop(queue);

            entry.vruntime = if min_vrt == u64::MAX { 0 } else { min_vrt };
        }

        let vruntime = entry.vruntime;

        match policy {
            SchedulingPolicy::RoundRobin => {
                self.rr_queue.write().push_back(entry);
            }
            SchedulingPolicy::Priority => {
                self.priority_queue.write().push(entry);
            }
            SchedulingPolicy::Fair => {
                self.fair_queue.write().push(FairEntry(entry));
            }
        }

        self.stats.write().active_processes += 1;
        info!(
            "Process {} added to scheduler (priority: {}, vruntime: {})",
            pid, priority, vruntime
        );
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
            SchedulingPolicy::Priority => {
                let mut queue = self.priority_queue.write();
                let original_len = queue.len();
                let entries: Vec<Entry> = queue.drain().filter(|e| e.pid != pid).collect();
                removed = entries.len() < original_len;
                for entry in entries {
                    queue.push(entry);
                }
            }
            SchedulingPolicy::Fair => {
                let mut queue = self.fair_queue.write();
                let original_len = queue.len();
                let entries: Vec<FairEntry> = queue.drain().filter(|e| e.0.pid != pid).collect();
                removed = entries.len() < original_len;
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
        drop(current);

        if removed {
            let mut stats = self.stats.write();
            stats.active_processes = stats.active_processes.saturating_sub(1);
            drop(stats);
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
            let elapsed = entry
                .last_scheduled
                .map(|t| now.duration_since(t))
                .unwrap_or_default();

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
                    SchedulingPolicy::Priority => {
                        self.priority_queue.write().push(new_entry);
                    }
                    SchedulingPolicy::Fair => {
                        self.fair_queue.write().push(FairEntry(new_entry));
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
                // For Fair scheduling, select process with minimum vruntime - O(log n)
                self.fair_queue.write().pop().map(|fe| fe.0)
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
                SchedulingPolicy::Priority => {
                    self.priority_queue.write().push(new_entry);
                }
                SchedulingPolicy::Fair => {
                    self.fair_queue.write().push(FairEntry(new_entry));
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

    /// Get number of processes in scheduler
    pub fn len(&self) -> usize {
        let policy = *self.policy.read();
        let queue_len = match policy {
            SchedulingPolicy::RoundRobin => self.rr_queue.read().len(),
            SchedulingPolicy::Priority => self.priority_queue.read().len(),
            SchedulingPolicy::Fair => self.fair_queue.read().len(),
        };
        queue_len + if self.current.read().is_some() { 1 } else { 0 }
    }

    /// Check if scheduler is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
                queue
                    .iter()
                    .find(|e| e.pid == pid)
                    .map(|entry| ProcessStats {
                        pid: entry.pid,
                        priority: entry.priority,
                        cpu_time_micros: entry.cpu_time_micros,
                        vruntime: entry.vruntime,
                        is_current: false,
                    })
            }
            SchedulingPolicy::Priority => {
                let queue = self.priority_queue.read();
                queue
                    .iter()
                    .find(|e| e.pid == pid)
                    .map(|entry| ProcessStats {
                        pid: entry.pid,
                        priority: entry.priority,
                        cpu_time_micros: entry.cpu_time_micros,
                        vruntime: entry.vruntime,
                        is_current: false,
                    })
            }
            SchedulingPolicy::Fair => {
                let queue = self.fair_queue.read();
                queue
                    .iter()
                    .find(|e| e.0.pid == pid)
                    .map(|entry| ProcessStats {
                        pid: entry.0.pid,
                        priority: entry.0.priority,
                        cpu_time_micros: entry.0.cpu_time_micros,
                        vruntime: entry.0.vruntime,
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
            SchedulingPolicy::Priority => {
                let queue = self.priority_queue.read();
                stats.extend(queue.iter().map(|entry| ProcessStats {
                    pid: entry.pid,
                    priority: entry.priority,
                    cpu_time_micros: entry.cpu_time_micros,
                    vruntime: entry.vruntime,
                    is_current: false,
                }));
            }
            SchedulingPolicy::Fair => {
                let queue = self.fair_queue.read();
                stats.extend(queue.iter().map(|entry| ProcessStats {
                    pid: entry.0.pid,
                    priority: entry.0.priority,
                    cpu_time_micros: entry.0.cpu_time_micros,
                    vruntime: entry.0.vruntime,
                    is_current: false,
                }));
            }
        }

        stats
    }
}
