/*!
 * Event Collection and Distribution
 * Manages eBPF event subscriptions and distribution
 */

use super::types::*;
use crate::core::sync::StripedMap;
use crate::core::types::Pid;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use uuid::Uuid;

/// Maximum events to keep in history
use crate::core::limits::MAX_EBPF_EVENT_HISTORY as MAX_EVENT_HISTORY;

/// Event collector and distributor
pub struct EventCollector {
    /// Event history buffer
    history: Arc<RwLock<VecDeque<EbpfEvent>>>,
    /// Active subscriptions
    subscriptions: Arc<StripedMap<String, Subscription>>,
    /// Event statistics
    stats: Arc<RwLock<EventStats>>,
}

/// Subscription information
struct Subscription {
    #[allow(dead_code)]
    id: String,
    event_type: SubscriptionType,
    callback: EventCallback,
}

/// Subscription type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionType {
    Syscall,
    Network,
    File,
    All,
}

/// Event statistics
struct EventStats {
    syscall_events: u64,
    network_events: u64,
    file_events: u64,
    total_events: u64,
    events_per_sec: f64,
    last_update: std::time::Instant,
}

impl EventCollector {
    pub fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_EVENT_HISTORY))),
            subscriptions: Arc::new(StripedMap::new(32)),
            stats: Arc::new(RwLock::new(EventStats {
                syscall_events: 0,
                network_events: 0,
                file_events: 0,
                total_events: 0,
                events_per_sec: 0.0,
                last_update: std::time::Instant::now(),
            })),
        }
    }

    /// Emit an event
    pub fn emit(&self, event: EbpfEvent) {
        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_events += 1;
            match event {
                EbpfEvent::Syscall(_) => stats.syscall_events += 1,
                EbpfEvent::Network(_) => stats.network_events += 1,
                EbpfEvent::File(_) => stats.file_events += 1,
            }

            // Update events per second
            let elapsed = stats.last_update.elapsed().as_secs_f64();
            if elapsed >= 1.0 {
                stats.events_per_sec = stats.total_events as f64 / elapsed;
                stats.last_update = std::time::Instant::now();
            }
        }

        // Add to history
        {
            let mut history = self.history.write();
            if history.len() >= MAX_EVENT_HISTORY {
                history.pop_front();
            }
            history.push_back(event.clone());
        }

        // Notify subscribers
        let event_clone = event.clone();
        self.subscriptions.iter(|_id, subscription| {
            let should_notify = match subscription.event_type {
                SubscriptionType::All => true,
                SubscriptionType::Syscall => matches!(&event_clone, EbpfEvent::Syscall(_)),
                SubscriptionType::Network => matches!(&event_clone, EbpfEvent::Network(_)),
                SubscriptionType::File => matches!(&event_clone, EbpfEvent::File(_)),
            };

            if should_notify {
                (subscription.callback)(event_clone.clone());
            }
        });
    }

    /// Subscribe to events
    pub fn subscribe(&self, event_type: SubscriptionType, callback: EventCallback) -> String {
        let id = Uuid::new_v4().to_string();
        let subscription = Subscription {
            id: id.clone(),
            event_type,
            callback,
        };

        self.subscriptions.insert(id.clone(), subscription);
        id
    }

    /// Unsubscribe from events
    pub fn unsubscribe(&self, subscription_id: &str) -> EbpfResult<()> {
        self.subscriptions
            .remove(&subscription_id.to_string())
            .ok_or_else(|| EbpfError::InvalidFilter {
                reason: format!("Subscription {} not found", subscription_id),
            })?;
        Ok(())
    }

    /// Get recent events
    pub fn recent(&self, limit: usize) -> Vec<EbpfEvent> {
        let history = self.history.read();
        let start = history.len().saturating_sub(limit);
        history.iter().skip(start).cloned().collect()
    }

    /// Get events by PID
    pub fn by_pid(&self, pid: Pid, limit: usize) -> Vec<EbpfEvent> {
        let history = self.history.read();
        history
            .iter()
            .rev()
            .filter(|e| e.pid() == pid)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64, u64, f64) {
        let stats = self.stats.read();
        (
            stats.syscall_events,
            stats.network_events,
            stats.file_events,
            stats.events_per_sec,
        )
    }

    /// Clear event history
    pub fn clear(&self) {
        self.history.write().clear();
    }
}

impl Default for EventCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventCollector {
    fn clone(&self) -> Self {
        Self {
            history: Arc::clone(&self.history),
            subscriptions: Arc::clone(&self.subscriptions),
            stats: Arc::clone(&self.stats),
        }
    }
}

// Re-export subscription type for external use
pub use SubscriptionType as EventType;
