/*!
 * Scheduler Policy Management
 * Handle dynamic policy and quantum changes
 */

use super::entry::{Entry, FairEntry};
use super::{QueueLocation, Scheduler};
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
        self.stats.set_policy(new_policy);

        // Requeue all processes under new policy and update location index
        for entry in all_entries {
            let pid = entry.pid;
            match new_policy {
                SchedulingPolicy::RoundRobin => {
                    self.rr_queue.write().push_back(entry);
                    self.process_locations
                        .insert(pid, QueueLocation::RoundRobin);
                }
                SchedulingPolicy::Priority => {
                    self.priority_queue.write().push(entry);
                    self.process_locations.insert(pid, QueueLocation::Priority);
                }
                SchedulingPolicy::Fair => {
                    self.fair_queue.write().push(FairEntry(entry));
                    self.process_locations.insert(pid, QueueLocation::Fair);
                }
            }
        }

        info!("Policy change complete: {} processes requeued", self.len());
    }

    /// Set time quantum dynamically
    pub fn set_quantum(&self, quantum: Duration) {
        *self.quantum.write() = quantum;
        self.stats.set_quantum(quantum);
        info!("Time quantum updated to {:?}", quantum);
    }

    /// Update process priority dynamically - O(1) lookup + O(n) heap rebuild
    pub fn set_priority(&self, pid: Pid, new_priority: Priority) -> bool {
        // Fast O(1) check if process exists
        let location = match self.process_locations.get(&pid) {
            Some(loc) => *loc,
            None => return false, // Process not in scheduler
        };

        // Update priority based on cached location
        match location {
            QueueLocation::Current => {
                let mut current = self.current.write();
                if let Some(ref mut entry) = *current {
                    if entry.pid == pid {
                        entry.priority = new_priority;
                        info!(
                            "Updated priority for current process {} to {}",
                            pid, new_priority
                        );
                        return true;
                    }
                }
                false
            }
            QueueLocation::RoundRobin => {
                let mut queue = self.rr_queue.write();
                if let Some(entry) = queue.iter_mut().find(|e| e.pid == pid) {
                    entry.priority = new_priority;
                    info!(
                        "Updated priority for queued process {} to {}",
                        pid, new_priority
                    );
                    return true;
                }
                false
            }
            QueueLocation::Priority => {
                // For heap, need to rebuild - unavoidable with BinaryHeap
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
                }
                found
            }
            QueueLocation::Fair => {
                // For heap, need to rebuild - unavoidable with BinaryHeap
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
                }
                found
            }
        }
    }

    /// Get current scheduling policy
    pub fn policy(&self) -> SchedulingPolicy {
        *self.policy.read()
    }
}
