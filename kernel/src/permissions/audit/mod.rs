/*!
 * Permission Audit Trail
 * Tracks permission checks and denials for security monitoring
 */

use crate::permissions::types::{PermissionRequest, PermissionResponse, Resource};
use crate::core::types::Pid;
use ahash::RandomState;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, TimestampSeconds};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::SystemTime;

/// Maximum events to keep in memory
use crate::core::limits::{MAX_AUDIT_EVENTS, MAX_AUDIT_EVENTS_PER_PID as MAX_PID_EVENTS};

/// Audit event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Info,
    Warning,
    Critical,
}

/// Permission audit event
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuditEvent {
    pub request: PermissionRequest,
    pub response: PermissionResponse,
    pub severity: AuditSeverity,
    #[serde_as(as = "TimestampSeconds<i64>")]
    pub logged_at: SystemTime,
}

impl AuditEvent {
    pub fn new(request: PermissionRequest, response: PermissionResponse) -> Self {
        let severity = if response.is_allowed() {
            AuditSeverity::Info
        } else {
            // Denied requests are more severe
            match &request.resource {
                Resource::System { .. } => AuditSeverity::Critical,
                Resource::Process { .. } => AuditSeverity::Critical,
                _ => AuditSeverity::Warning,
            }
        };

        Self {
            request,
            response,
            severity,
            logged_at: SystemTime::now(),
        }
    }

    pub fn with_severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }
}

/// Audit logger for permission checks
pub struct AuditLogger {
    /// Global event log (ring buffer)
    events: parking_lot::RwLock<VecDeque<AuditEvent>>,
    /// Per-PID event logs
    pid_events: Arc<DashMap<Pid, VecDeque<AuditEvent>, RandomState>>,
    /// Denial counters for monitoring
    denial_counts: Arc<DashMap<Pid, u64, RandomState>>,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            events: parking_lot::RwLock::new(VecDeque::with_capacity(MAX_AUDIT_EVENTS)),
            pid_events: Arc::new(DashMap::with_hasher(RandomState::new())),
            denial_counts: Arc::new(DashMap::with_hasher(RandomState::new())),
        }
    }

    /// Log a permission check
    pub fn log(&self, event: AuditEvent) {
        let pid = event.request.pid;
        let is_denied = !event.response.is_allowed();

        // Add to global log
        {
            let mut events = self.events.write();
            if events.len() >= MAX_AUDIT_EVENTS {
                events.pop_front();
            }
            events.push_back(event.clone());
        }

        // Add to PID-specific log
        self.pid_events
            .entry(pid)
            .or_insert_with(|| VecDeque::with_capacity(MAX_PID_EVENTS))
            .push_back(event);

        // Trim PID log if needed
        if let Some(mut entry) = self.pid_events.get_mut(&pid) {
            if entry.len() > MAX_PID_EVENTS {
                entry.pop_front();
            }
        }

        // Track denials
        if is_denied {
            self.denial_counts
                .entry(pid)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    /// Get recent events
    pub fn recent(&self, limit: usize) -> Vec<AuditEvent> {
        let events = self.events.read();
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Get events for a specific PID
    pub fn for_pid(&self, pid: Pid, limit: usize) -> Vec<AuditEvent> {
        if let Some(entry) = self.pid_events.get(&pid) {
            entry.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get denial count for PID
    pub fn denial_count(&self, pid: Pid) -> u64 {
        self.denial_counts.get(&pid).map(|e| *e).unwrap_or(0)
    }

    /// Get all PIDs with denials
    pub fn pids_with_denials(&self) -> Vec<(Pid, u64)> {
        self.denial_counts
            .iter()
            .map(|entry| (*entry.key(), *entry.value()))
            .collect()
    }

    /// Clear logs for a PID
    pub fn clear_pid(&self, pid: Pid) {
        self.pid_events.remove(&pid);
        self.denial_counts.remove(&pid);
    }

    /// Clear all logs
    pub fn clear_all(&self) {
        self.events.write().clear();
        self.pid_events.clear();
        self.denial_counts.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> AuditStats {
        let total_events = self.events.read().len();
        let total_denials: u64 = self.denial_counts.iter().map(|e| *e.value()).sum();
        let pids_tracked = self.pid_events.len();

        AuditStats {
            total_events,
            total_denials,
            pids_tracked,
        }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_events: usize,
    pub total_denials: u64,
    pub pids_tracked: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::types::{PermissionRequest, PermissionResponse};
    use std::path::PathBuf;

    #[test]
    fn test_audit_logging() {
        let logger = AuditLogger::new();
        let req = PermissionRequest::file_read(100, PathBuf::from("/test"));
        let resp = PermissionResponse::deny(req.clone(), "test");
        let event = AuditEvent::new(req, resp);

        logger.log(event);

        let recent = logger.recent(10);
        assert_eq!(recent.len(), 1);

        let for_pid = logger.for_pid(100, 10);
        assert_eq!(for_pid.len(), 1);

        assert_eq!(logger.denial_count(100), 1);
    }

    #[test]
    fn test_audit_stats() {
        let logger = AuditLogger::new();

        for i in 0..5 {
            let req = PermissionRequest::file_read(100 + i, PathBuf::from("/test"));
            let resp = if i % 2 == 0 {
                PermissionResponse::deny(req.clone(), "test")
            } else {
                PermissionResponse::allow(req.clone(), "test")
            };
            logger.log(AuditEvent::new(req, resp));
        }

        let stats = logger.stats();
        assert_eq!(stats.total_events, 5);
        assert_eq!(stats.total_denials, 3); // 0, 2, 4
    }

    #[test]
    fn test_ring_buffer() {
        let logger = AuditLogger::new();

        // Add more than MAX_AUDIT_EVENTS
        for i in 0..(MAX_AUDIT_EVENTS + 100) {
            let req = PermissionRequest::file_read(100, PathBuf::from(format!("/test{}", i)));
            let resp = PermissionResponse::allow(req.clone(), "test");
            logger.log(AuditEvent::new(req, resp));
        }

        let stats = logger.stats();
        assert_eq!(stats.total_events, MAX_AUDIT_EVENTS);
    }
}

