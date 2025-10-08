/*!
 * Query System
 * Real-time event analysis and aggregation
 *
 * Enables powerful queries like:
 * - "Show all slow syscalls in last 5 minutes"
 * - "Find processes causing memory pressure"
 * - "Trace causality chain for failed operation"
 */

use crate::core::types::Pid;
use crate::monitoring::events::{Category, Event, EventFilter, Payload, Severity};
use std::collections::HashMap;
use std::time::Duration;

/// Query result containing matched events and aggregations
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub events: Vec<Event>,
    pub count: usize,
    pub aggregations: HashMap<String, Aggregation>,
}

/// Aggregation result types
#[derive(Debug, Clone)]
pub enum Aggregation {
    Count(u64),
    Sum(f64),
    Avg(f64),
    Min(f64),
    Max(f64),
    Percentile { p50: f64, p95: f64, p99: f64 },
    Distribution(HashMap<String, u64>),
}

/// Query builder for fluent API
pub struct Query {
    filter: EventFilter,
    limit: Option<usize>,
    aggregations: Vec<AggregationType>,
}

/// Aggregation type specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregationType {
    CountByCategory,
    CountBySeverity,
    CountByPid,
    DurationStats,
    CustomGroupBy(String),
}

impl Query {
    /// Create a new query
    pub fn new() -> Self {
        Self {
            filter: EventFilter::new(),
            limit: None,
            aggregations: Vec::new(),
        }
    }

    /// Filter by minimum severity
    pub fn severity(mut self, severity: Severity) -> Self {
        self.filter = self.filter.severity(severity);
        self
    }

    /// Filter by category
    pub fn category(mut self, category: Category) -> Self {
        self.filter = self.filter.category(category);
        self
    }

    /// Filter by process ID
    pub fn pid(mut self, pid: Pid) -> Self {
        self.filter = self.filter.pid(pid);
        self
    }

    /// Filter by time range (events newer than duration ago)
    pub fn since(mut self, duration: Duration) -> Self {
        self.filter = self.filter.since(duration);
        self
    }

    /// Limit number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add aggregation
    pub fn aggregate(mut self, agg: AggregationType) -> Self {
        self.aggregations.push(agg);
        self
    }

    /// Execute query on event slice
    pub fn execute(&self, events: &[Event]) -> QueryResult {
        // Filter events
        let mut filtered: Vec<Event> = events
            .iter()
            .filter(|e| e.matches(&self.filter))
            .cloned()
            .collect();

        // Apply limit
        if let Some(limit) = self.limit {
            filtered.truncate(limit);
        }

        // Compute aggregations
        let mut aggregations = HashMap::new();
        for agg_type in &self.aggregations {
            match agg_type {
                AggregationType::CountByCategory => {
                    let agg = Self::count_by_category(&filtered);
                    aggregations.insert("by_category".to_string(), agg);
                }
                AggregationType::CountBySeverity => {
                    let agg = Self::count_by_severity(&filtered);
                    aggregations.insert("by_severity".to_string(), agg);
                }
                AggregationType::CountByPid => {
                    let agg = Self::count_by_pid(&filtered);
                    aggregations.insert("by_pid".to_string(), agg);
                }
                AggregationType::DurationStats => {
                    let agg = Self::duration_stats(&filtered);
                    aggregations.insert("duration_stats".to_string(), agg);
                }
                AggregationType::CustomGroupBy(field) => {
                    let agg = Self::group_by_field(&filtered, field);
                    aggregations.insert(format!("by_{}", field), agg);
                }
            }
        }

        QueryResult {
            count: filtered.len(),
            events: filtered,
            aggregations,
        }
    }

    /// Count events by category
    fn count_by_category(events: &[Event]) -> Aggregation {
        let mut counts: HashMap<String, u64> = HashMap::new();
        for event in events {
            *counts.entry(format!("{:?}", event.category)).or_insert(0) += 1;
        }
        Aggregation::Distribution(counts)
    }

    /// Count events by severity
    fn count_by_severity(events: &[Event]) -> Aggregation {
        let mut counts: HashMap<String, u64> = HashMap::new();
        for event in events {
            *counts.entry(format!("{:?}", event.severity)).or_insert(0) += 1;
        }
        Aggregation::Distribution(counts)
    }

    /// Count events by PID
    fn count_by_pid(events: &[Event]) -> Aggregation {
        let mut counts: HashMap<String, u64> = HashMap::new();
        for event in events {
            if let Some(pid) = event.pid {
                *counts.entry(pid.to_string()).or_insert(0) += 1;
            }
        }
        Aggregation::Distribution(counts)
    }

    /// Calculate duration statistics from syscall events
    fn duration_stats(events: &[Event]) -> Aggregation {
        let mut durations: Vec<f64> = events
            .iter()
            .filter_map(|e| match &e.payload {
                Payload::SyscallExit { duration_us, .. } => Some(*duration_us as f64),
                Payload::SyscallSlow { duration_ms, .. } => Some(*duration_ms as f64 * 1000.0),
                _ => None,
            })
            .collect();

        if durations.is_empty() {
            return Aggregation::Percentile {
                p50: 0.0,
                p95: 0.0,
                p99: 0.0,
            };
        }

        // Sort by duration, treating NaN as equal (should never occur for durations)
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p50 = Self::percentile(&durations, 0.50);
        let p95 = Self::percentile(&durations, 0.95);
        let p99 = Self::percentile(&durations, 0.99);

        Aggregation::Percentile { p50, p95, p99 }
    }

    /// Calculate percentile from sorted values
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let idx = ((sorted.len() - 1) as f64 * p) as usize;
        sorted[idx]
    }

    /// Group by custom field
    fn group_by_field(events: &[Event], field: &str) -> Aggregation {
        let mut counts: HashMap<String, u64> = HashMap::new();

        for event in events {
            let key = match field {
                "category" => format!("{:?}", event.category),
                "severity" => format!("{:?}", event.severity),
                "pid" => event
                    .pid
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                _ => "unknown".to_string(),
            };

            *counts.entry(key).or_insert(0) += 1;
        }

        Aggregation::Distribution(counts)
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new()
    }
}

/// Prebuilt queries for common patterns
pub struct CommonQueries;

impl CommonQueries {
    /// Find slow operations
    pub fn slow_operations(_threshold_ms: u64) -> Query {
        Query::new()
            .severity(Severity::Warn)
            .category(Category::Performance)
            .since(Duration::from_secs(300)) // Last 5 minutes
    }

    /// Find errors for a process
    pub fn process_errors(pid: Pid) -> Query {
        Query::new()
            .severity(Severity::Error)
            .pid(pid)
            .since(Duration::from_secs(60))
    }

    /// System health overview
    pub fn health_check() -> Query {
        Query::new()
            .severity(Severity::Warn)
            .since(Duration::from_secs(300))
            .aggregate(AggregationType::CountByCategory)
            .aggregate(AggregationType::CountBySeverity)
    }

    /// Memory pressure events
    pub fn memory_pressure() -> Query {
        Query::new()
            .category(Category::Memory)
            .severity(Severity::Warn)
            .since(Duration::from_secs(60))
    }

    /// Syscall performance analysis
    pub fn syscall_performance() -> Query {
        Query::new()
            .category(Category::Syscall)
            .since(Duration::from_secs(300))
            .aggregate(AggregationType::DurationStats)
    }

    /// Security events
    pub fn security_events() -> Query {
        Query::new()
            .category(Category::Security)
            .severity(Severity::Warn)
            .since(Duration::from_secs(3600)) // Last hour
    }
}

/// Causality chain tracer
pub struct CausalityTracer;

impl CausalityTracer {
    /// Trace all events in a causality chain
    pub fn trace(events: &[Event], causality_id: u64) -> Vec<Event> {
        events
            .iter()
            .filter(|e| e.causality_id == Some(causality_id))
            .cloned()
            .collect()
    }

    /// Find root cause (earliest event in chain)
    pub fn root_cause(events: &[Event], causality_id: u64) -> Option<Event> {
        Self::trace(events, causality_id)
            .into_iter()
            .min_by_key(|e| e.timestamp_ns)
    }

    /// Build event timeline for causality chain
    pub fn timeline(events: &[Event], causality_id: u64) -> Vec<(Duration, Event)> {
        let mut chain = Self::trace(events, causality_id);
        chain.sort_by_key(|e| e.timestamp_ns);

        if chain.is_empty() {
            return Vec::new();
        }

        let start = chain[0].timestamp_ns;
        chain
            .into_iter()
            .map(|e| {
                let offset = Duration::from_nanos(e.timestamp_ns - start);
                (offset, e)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_events() -> Vec<Event> {
        vec![
            Event::new(
                Severity::Info,
                Category::Process,
                Payload::ProcessCreated {
                    name: "test".to_string(),
                    priority: 5,
                },
            )
            .with_pid(100),
            Event::new(
                Severity::Warn,
                Category::Memory,
                Payload::MemoryPressure {
                    usage_pct: 85,
                    available_mb: 100,
                },
            )
            .with_pid(100),
            Event::new(
                Severity::Error,
                Category::Syscall,
                Payload::SyscallExit {
                    name: "read".to_string(),
                    duration_us: 1500,
                    result: crate::monitoring::events::SyscallResult::Error,
                },
            )
            .with_pid(100),
        ]
    }

    #[test]
    fn test_query_basic() {
        let events = create_test_events();

        let result = Query::new().severity(Severity::Warn).execute(&events);

        assert_eq!(result.count, 2); // Warn and Error
    }

    #[test]
    fn test_query_with_category() {
        let events = create_test_events();

        let result = Query::new().category(Category::Memory).execute(&events);

        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_query_with_limit() {
        let events = create_test_events();

        let result = Query::new().limit(1).execute(&events);

        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_aggregation_by_category() {
        let events = create_test_events();

        let result = Query::new()
            .aggregate(AggregationType::CountByCategory)
            .execute(&events);

        assert!(result.aggregations.contains_key("by_category"));
    }

    #[test]
    fn test_causality_tracing() {
        let events = vec![
            Event::new(
                Severity::Info,
                Category::Process,
                Payload::ProcessCreated {
                    name: "test".to_string(),
                    priority: 5,
                },
            )
            .with_causality(123),
            Event::new(
                Severity::Warn,
                Category::Memory,
                Payload::MemoryPressure {
                    usage_pct: 85,
                    available_mb: 100,
                },
            )
            .with_causality(123),
        ];

        let chain = CausalityTracer::trace(&events, 123);
        assert_eq!(chain.len(), 2);

        let root = CausalityTracer::root_cause(&events, 123);
        assert!(root.is_some());
    }
}
