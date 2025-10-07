/*!
 * Scheduler Policy Management
 * Handle dynamic policy and quantum changes
 */

use super::entry::{Entry, FairEntry};
use super::Scheduler;
use crate::core::types::{Pid, Priority};
use crate::process::types::SchedulingPolicy;
use log::info;
use std::time::Duration;

impl Scheduler {
    /// Change scheduling policy (preserves processes but requeues them)
    pub fn set_policy(&self, new_policy: SchedulingPolicy) {
        let current_policy = *self.policy.read();
        if new_policy == current_policy {
            return;
        }

        info!(
            "Changing scheduler policy from {:?} to {:?} (requeuing all processes)",
            current_policy, new_policy
        );

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

        // Collect from fair queue
        let mut fq = self.fair_queue.write();
        all_entries.extend(fq.drain().map(|fe| fe.0));
        drop(fq);

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
                SchedulingPolicy::Priority => {
                    self.priority_queue.write().push(entry);
                }
                SchedulingPolicy::Fair => {
                    self.fair_queue.write().push(FairEntry(entry));
                }
            }
        }

        info!("Policy change complete: {} processes requeued", self.len());
    }

    /// Set time quantum dynamically
    pub fn set_quantum(&self, quantum: Duration) {
        *self.quantum.write() = quantum;
        self.stats.write().quantum_micros = quantum.as_micros() as u64;
        info!("Time quantum updated to {:?}", quantum);
    }

    /// Update process priority dynamically
    pub fn set_priority(&self, pid: Pid, new_priority: Priority) -> bool {
        let policy = *self.policy.read();

        // Update in current process
        let mut current = self.current.write();
        if let Some(ref mut entry) = *current {
            if entry.pid == pid {
                entry.priority = new_priority;
                drop(current);
                info!(
                    "Updated priority for current process {} to {}",
                    pid, new_priority
                );
                return true;
            }
        }
        drop(current);

        // Update in queues
        match policy {
            SchedulingPolicy::RoundRobin => {
                let mut queue = self.rr_queue.write();
                if let Some(entry) = queue.iter_mut().find(|e| e.pid == pid) {
                    entry.priority = new_priority;
                    info!(
                        "Updated priority for queued process {} to {}",
                        pid, new_priority
                    );
                    return true;
                }
            }
            SchedulingPolicy::Priority => {
                // For heap, need to rebuild
                let mut queue = self.priority_queue.write();
                let mut entries: Vec<Entry> = queue.drain().collect();
                let found = entries.iter_mut().any(|e| {
                    if e.pid == pid {
                        e.priority = new_priority;
                        true
                    } else {
                        false
                    }
                });

                // Rebuild heap
                for entry in entries {
                    queue.push(entry);
                }

                if found {
                    info!(
                        "Updated priority for queued process {} to {}",
                        pid, new_priority
                    );
                    return true;
                }
            }
            SchedulingPolicy::Fair => {
                // For heap, need to rebuild
                let mut queue = self.fair_queue.write();
                let mut entries: Vec<FairEntry> = queue.drain().collect();
                let found = entries.iter_mut().any(|e| {
                    if e.0.pid == pid {
                        e.0.priority = new_priority;
                        true
                    } else {
                        false
                    }
                });

                // Rebuild heap
                for entry in entries {
                    queue.push(entry);
                }

                if found {
                    info!(
                        "Updated priority for queued process {} to {}",
                        pid, new_priority
                    );
                    return true;
                }
            }
        }

        false
    }

    /// Get current scheduling policy
    pub fn policy(&self) -> SchedulingPolicy {
        *self.policy.read()
    }
}
