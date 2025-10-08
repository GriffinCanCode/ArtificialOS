/*!
 * Event System
 * Strongly-typed observability events with zero-copy semantics
 */

use crate::core::data_structures::InlineString;
use crate::core::types::Pid;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Event severity for filtering and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Severity {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Critical = 5,
}

/// Event category for organization and querying
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Category {
    Process,
    Memory,
    Syscall,
    Network,
    Ipc,
    Scheduler,
    Security,
    Performance,
    Resource,
}

/// Unified event type - all observability events flow through this
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Monotonic timestamp (nanoseconds since boot)
    pub timestamp_ns: u64,
    /// Event severity
    pub severity: Severity,
    /// Event category
    pub category: Category,
    /// Causality chain ID (links related events)
    pub causality_id: Option<u64>,
    /// Process ID if applicable
    pub pid: Option<Pid>,
    /// Event payload
    pub payload: Payload,
}

/// Event payload - strongly typed variants for each event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Payload {
    // Process events
    ProcessCreated {
        name: InlineString,
        priority: u8,
    },
    ProcessTerminated {
        exit_code: Option<i32>,
    },
    ProcessStateChanged {
        from: InlineString,
        to: InlineString,
    },

    // Syscall events
    SyscallEnter {
        name: InlineString,
        args_hash: u64,
    },
    SyscallExit {
        name: InlineString,
        duration_us: u64,
        result: SyscallResult,
    },
    SyscallSlow {
        name: InlineString,
        duration_ms: u64,
        threshold_ms: u64,
    },

    // Memory events
    MemoryAllocated {
        size: usize,
        region_id: u64,
    },
    MemoryFreed {
        size: usize,
        region_id: u64,
    },
    MemoryPressure {
        usage_pct: u8,
        available_mb: u64,
    },

    // Scheduler events
    ContextSwitch {
        from_pid: Pid,
        to_pid: Pid,
        reason: InlineString,
    },
    ProcessPreempted {
        quantum_remaining_us: u64,
    },
    SchedulerLatency {
        wake_to_run_us: u64,
    },

    // Network events
    ConnectionEstablished {
        protocol: InlineString,
        local_port: u16,
        remote_addr: InlineString,
    },
    ConnectionClosed {
        bytes_sent: u64,
        bytes_received: u64,
    },
    NetworkError {
        error: InlineString,
        retry_count: u8,
    },

    // IPC events
    MessageSent {
        queue_id: u64,
        size: usize,
    },
    MessageReceived {
        queue_id: u64,
        size: usize,
        wait_time_us: u64,
    },
    IpcTimeout {
        queue_id: u64,
        timeout_ms: u64,
    },

    // Security events
    PermissionDenied {
        operation: InlineString,
        required: InlineString,
    },
    RateLimitExceeded {
        limit: u32,
        current: u32,
    },
    SecurityViolation {
        description: InlineString,
    },

    // Performance events
    OperationSlow {
        operation: InlineString,
        duration_ms: u64,
        p99_ms: u64,
    },
    BudgetExceeded {
        operation: InlineString,
        budget_ms: u64,
        actual_ms: u64,
    },
    CpuThrottled {
        usage_pct: u8,
        duration_ms: u64,
    },

    // Resource events
    ResourceExhausted {
        resource: InlineString,
        limit: u64,
    },
    ResourceLeaked {
        resource: InlineString,
        count: u64,
    },
    ResourceReclaimed {
        resource: InlineString,
        count: u64,
    },

    // Anomaly detection
    AnomalyDetected {
        metric: InlineString,
        value: f64,
        expected: f64,
        deviation: f64,
    },

    // Custom metric update
    MetricUpdate {
        name: InlineString,
        value: f64,
        labels: Vec<(InlineString, InlineString)>,
    },
}

/// Syscall result for fast pattern matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyscallResult {
    Success,
    Error,
    Timeout,
}

impl Event {
    /// Create a new event with current timestamp
    #[inline]
    pub fn new(severity: Severity, category: Category, payload: Payload) -> Self {
        Self {
            timestamp_ns: Self::now_ns(),
            severity,
            category,
            causality_id: None,
            pid: None,
            payload,
        }
    }

    /// Create event with causality tracking
    #[inline]
    pub fn with_causality(mut self, causality_id: u64) -> Self {
        self.causality_id = Some(causality_id);
        self
    }

    /// Create event with process context
    #[inline]
    pub fn with_pid(mut self, pid: Pid) -> Self {
        self.pid = Some(pid);
        self
    }

    /// Get current time in nanoseconds (monotonic)
    #[inline]
    fn now_ns() -> u64 {
        // Use a cached Instant for relative timing
        static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        let start = START.get_or_init(Instant::now);
        start.elapsed().as_nanos() as u64
    }

    /// Get event age
    #[inline]
    pub fn age(&self) -> Duration {
        Duration::from_nanos(Self::now_ns() - self.timestamp_ns)
    }

    /// Check if event matches filter criteria
    #[inline]
    pub fn matches(&self, filter: &EventFilter) -> bool {
        if let Some(min_severity) = filter.min_severity {
            if self.severity < min_severity {
                return false;
            }
        }

        if let Some(category) = filter.category {
            if self.category != category {
                return false;
            }
        }

        if let Some(pid) = filter.pid {
            if self.pid != Some(pid) {
                return false;
            }
        }

        true
    }
}

/// Event filter for querying
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub min_severity: Option<Severity>,
    pub category: Option<Category>,
    pub pid: Option<Pid>,
    pub since_ns: Option<u64>,
    pub until_ns: Option<u64>,
}

impl EventFilter {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn severity(mut self, severity: Severity) -> Self {
        self.min_severity = Some(severity);
        self
    }

    #[inline]
    pub fn category(mut self, category: Category) -> Self {
        self.category = Some(category);
        self
    }

    #[inline]
    pub fn pid(mut self, pid: Pid) -> Self {
        self.pid = Some(pid);
        self
    }

    #[inline]
    pub fn since(mut self, duration: Duration) -> Self {
        let now = Event::now_ns();
        self.since_ns = Some(now.saturating_sub(duration.as_nanos() as u64));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "test".into(),
                priority: 5,
            },
        );

        assert_eq!(event.severity, Severity::Info);
        assert_eq!(event.category, Category::Process);
    }

    #[test]
    fn test_event_filter() {
        let event = Event::new(
            Severity::Warn,
            Category::Memory,
            Payload::MemoryPressure {
                usage_pct: 85,
                available_mb: 100,
            },
        )
        .with_pid(123);

        let filter = EventFilter::new()
            .severity(Severity::Info)
            .category(Category::Memory)
            .pid(123);

        assert!(event.matches(&filter));

        let filter = EventFilter::new().severity(Severity::Error);
        assert!(!event.matches(&filter));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warn);
        assert!(Severity::Warn > Severity::Info);
        assert!(Severity::Critical > Severity::Error);
    }
}
