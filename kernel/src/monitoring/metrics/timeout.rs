/*!
 * Timeout Observability
 *
 * Monitoring and metrics for timeout events across the system.
 */

use crate::core::types::Pid;
use crate::monitoring::collection::Collector;
use crate::monitoring::events::{Category, Event, Payload, Severity};
use std::sync::Arc;
use std::time::Duration;

/// Timeout event emitter for observability
pub struct TimeoutObserver {
    collector: Arc<Collector>,
}

impl TimeoutObserver {
    /// Create new timeout observer
    pub fn new(collector: Arc<Collector>) -> Self {
        Self { collector }
    }

    /// Emit lock acquisition timeout event
    pub fn emit_lock_timeout(
        &self,
        resource_type: &'static str,
        pid: Option<Pid>,
        elapsed: Duration,
        timeout: Duration,
    ) {
        self.emit_timeout_event(
            "lock_timeout",
            resource_type,
            "lock",
            pid,
            elapsed,
            Some(timeout),
        );
    }

    /// Emit IPC operation timeout event
    pub fn emit_ipc_timeout(
        &self,
        resource_type: &'static str,
        resource_id: u64,
        pid: Pid,
        elapsed: Duration,
        timeout: Duration,
    ) {
        let event = Event::new(
            Severity::Warn,
            Category::Ipc,
            Payload::MetricUpdate {
                name: "ipc_operation_timeout".to_string(),
                value: elapsed.as_millis() as f64,
                labels: vec![
                    ("resource_type".to_string(), resource_type.to_string()),
                    ("resource_id".to_string(), resource_id.to_string()),
                    ("elapsed_ms".to_string(), elapsed.as_millis().to_string()),
                    ("timeout_ms".to_string(), timeout.as_millis().to_string()),
                ],
            },
        )
        .with_pid(pid);

        self.collector.emit(event);
    }

    /// Emit I/O timeout event
    pub fn emit_io_timeout(
        &self,
        operation: &str,
        fd: u32,
        pid: Pid,
        elapsed: Duration,
        timeout: Duration,
    ) {
        let event = Event::new(
            Severity::Warn,
            Category::Resource,
            Payload::MetricUpdate {
                name: "io_operation_timeout".to_string(),
                value: elapsed.as_millis() as f64,
                labels: vec![
                    ("operation".to_string(), operation.to_string()),
                    ("fd".to_string(), fd.to_string()),
                    ("elapsed_ms".to_string(), elapsed.as_millis().to_string()),
                    ("timeout_ms".to_string(), timeout.as_millis().to_string()),
                ],
            },
        )
        .with_pid(pid);

        self.collector.emit(event);
    }

    /// Emit async task timeout event
    pub fn emit_task_timeout(
        &self,
        task_id: Option<u64>,
        pid: Option<Pid>,
        elapsed: Duration,
        timeout: Duration,
    ) {
        let mut labels = vec![
            ("elapsed_ms".to_string(), elapsed.as_millis().to_string()),
            ("timeout_ms".to_string(), timeout.as_millis().to_string()),
        ];

        if let Some(id) = task_id {
            labels.push(("task_id".to_string(), id.to_string()));
        }

        let mut event = Event::new(
            Severity::Warn,
            Category::Performance,
            Payload::MetricUpdate {
                name: "async_task_timeout".to_string(),
                value: elapsed.as_millis() as f64,
                labels,
            },
        );

        if let Some(pid) = pid {
            event = event.with_pid(pid);
        }

        self.collector.emit(event);
    }

    /// Emit generic timeout event
    fn emit_timeout_event(
        &self,
        metric_name: &str,
        resource_type: &'static str,
        category: &str,
        pid: Option<Pid>,
        elapsed: Duration,
        timeout: Option<Duration>,
    ) {
        let mut labels = vec![
            ("resource_type".to_string(), resource_type.to_string()),
            ("category".to_string(), category.to_string()),
            ("elapsed_ms".to_string(), elapsed.as_millis().to_string()),
        ];

        if let Some(t) = timeout {
            labels.push(("timeout_ms".to_string(), t.as_millis().to_string()));
        }

        let mut event = Event::new(
            Severity::Warn,
            Category::Performance,
            Payload::MetricUpdate {
                name: metric_name.to_string(),
                value: elapsed.as_millis() as f64,
                labels,
            },
        );

        if let Some(pid) = pid {
            event = event.with_pid(pid);
        }

        self.collector.emit(event);
    }

    /// Emit timeout summary metrics (for periodic aggregation)
    pub fn emit_timeout_summary(&self, category: &str, count: u64, avg_elapsed_ms: f64) {
        let event = Event::new(
            Severity::Info,
            Category::Performance,
            Payload::MetricUpdate {
                name: "timeout_summary".to_string(),
                value: count as f64,
                labels: vec![
                    ("category".to_string(), category.to_string()),
                    ("count".to_string(), count.to_string()),
                    ("avg_elapsed_ms".to_string(), avg_elapsed_ms.to_string()),
                ],
            },
        );

        self.collector.emit(event);
    }
}

/// Global timeout statistics
pub struct TimeoutStats {
    pub lock_timeouts: u64,
    pub ipc_timeouts: u64,
    pub io_timeouts: u64,
    pub task_timeouts: u64,
    pub total_timeouts: u64,
}

impl TimeoutStats {
    pub fn new() -> Self {
        Self {
            lock_timeouts: 0,
            ipc_timeouts: 0,
            io_timeouts: 0,
            task_timeouts: 0,
            total_timeouts: 0,
        }
    }

    pub fn record_lock_timeout(&mut self) {
        self.lock_timeouts += 1;
        self.total_timeouts += 1;
    }

    pub fn record_ipc_timeout(&mut self) {
        self.ipc_timeouts += 1;
        self.total_timeouts += 1;
    }

    pub fn record_io_timeout(&mut self) {
        self.io_timeouts += 1;
        self.total_timeouts += 1;
    }

    pub fn record_task_timeout(&mut self) {
        self.task_timeouts += 1;
        self.total_timeouts += 1;
    }
}

impl Default for TimeoutStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_stats() {
        let mut stats = TimeoutStats::new();

        stats.record_lock_timeout();
        stats.record_ipc_timeout();
        stats.record_io_timeout();

        assert_eq!(stats.lock_timeouts, 1);
        assert_eq!(stats.ipc_timeouts, 1);
        assert_eq!(stats.io_timeouts, 1);
        assert_eq!(stats.total_timeouts, 3);
    }
}
