/*!
 * Unified Collector
 * Central orchestrator for all observability data
 *
 * Integrates: events, metrics, tracing, sampling, anomaly detection
 */

use crate::core::types::Pid;
use crate::monitoring::analysis::{Detector, Query, QueryResult, SampleDecision, Sampler};
use crate::monitoring::events::{Category, Event, Payload, Severity, SyscallResult};
use crate::monitoring::metrics::{MetricsCollector, MetricsSnapshot};
use crate::monitoring::streaming::{EventStream, StreamStats, Subscriber};
use std::sync::Arc;

/// Unified observability collector
pub struct Collector {
    /// Event stream
    stream: EventStream,

    /// Metrics collector (legacy support)
    metrics: Arc<MetricsCollector>,

    /// Adaptive sampler
    sampler: Sampler,

    /// Anomaly detector
    detector: Detector,

    /// Causality ID generator
    causality_gen: Arc<std::sync::atomic::AtomicU64>,
}

impl Collector {
    /// Create a new collector
    pub fn new() -> Self {
        Self {
            stream: EventStream::new(),
            metrics: Arc::new(MetricsCollector::new()),
            sampler: Sampler::new(),
            detector: Detector::new(),
            causality_gen: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }

    /// Emit an event (primary API)
    #[inline]
    pub fn emit(&self, event: Event) {
        // Apply sampling
        if self.sampler.should_sample() == SampleDecision::Reject {
            return;
        }

        // Check for anomalies
        if let Some(anomaly) = self.detector.check(&event) {
            // Emit anomaly event
            let anomaly_event = Event::new(
                Severity::Warn,
                Category::Performance,
                Payload::AnomalyDetected {
                    metric: anomaly.metric,
                    value: anomaly.value,
                    expected: anomaly.expected,
                    deviation: anomaly.deviation,
                },
            );
            let _ = self.stream.publish(anomaly_event);
        }

        // Update legacy metrics
        self.update_metrics(&event);

        // Publish to stream
        let _ = self.stream.publish(event);
    }

    /// Emit event with automatic causality tracking
    #[inline]
    pub fn emit_causal(&self, event: Event) -> u64 {
        let causality_id = self
            .causality_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.emit(event.with_causality(causality_id));
        causality_id
    }

    /// Emit event in causality chain
    #[inline]
    pub fn emit_in_chain(&self, event: Event, causality_id: u64) {
        self.emit(event.with_causality(causality_id));
    }

    /// Subscribe to event stream
    pub fn subscribe(&self) -> Subscriber {
        self.stream.subscribe()
    }

    /// Query events (requires subscriber)
    pub fn query(&self, query: Query, subscriber: &mut Subscriber) -> QueryResult {
        let events = self.collect_events(subscriber);
        query.execute(&events)
    }

    /// Collect all available events from subscriber
    fn collect_events(&self, subscriber: &mut Subscriber) -> Vec<Event> {
        let mut events = Vec::new();
        while let Some(event) = subscriber.next() {
            events.push(event);
        }
        events
    }

    /// Get metrics snapshot
    pub fn metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Get stream statistics
    pub fn stream_stats(&self) -> StreamStats {
        self.stream.stats()
    }

    /// Get sampling rate
    pub fn sampling_rate(&self) -> u8 {
        self.sampler.rate()
    }

    /// Update sampling overhead estimate
    pub fn update_overhead(&self, overhead_pct: u8) {
        self.sampler.update_overhead(overhead_pct);
    }

    /// Update legacy metrics from event
    fn update_metrics(&self, event: &Event) {
        match &event.payload {
            Payload::SyscallExit {
                name, duration_us, ..
            } => {
                self.metrics.observe_histogram(
                    &format!("syscall.{}", name),
                    *duration_us as f64 / 1_000_000.0,
                );
                self.metrics.inc_counter("syscall.total", 1.0);
            }
            Payload::MemoryAllocated { size, .. } => {
                self.metrics
                    .inc_counter("memory.allocated_bytes", *size as f64);
            }
            Payload::MemoryFreed { size, .. } => {
                self.metrics.inc_counter("memory.freed_bytes", *size as f64);
            }
            Payload::ProcessCreated { .. } => {
                self.metrics.inc_counter("process.created", 1.0);
            }
            Payload::ProcessTerminated { .. } => {
                self.metrics.inc_counter("process.terminated", 1.0);
            }
            Payload::MetricUpdate { name, value, .. } => {
                self.metrics.set_gauge(name, *value);
            }
            _ => {}
        }
    }

    /// Reset all observability state
    pub fn reset(&self) {
        self.metrics.reset();
        self.sampler.reset();
        self.detector.reset();
    }
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Collector {
    fn clone(&self) -> Self {
        Self {
            stream: self.stream.clone(),
            metrics: Arc::clone(&self.metrics),
            sampler: self.sampler.clone(),
            detector: self.detector.clone(),
            causality_gen: Arc::clone(&self.causality_gen),
        }
    }
}

/// Convenience functions for common events
impl Collector {
    /// Record process created
    pub fn process_created(&self, pid: Pid, name: String, priority: u8) {
        self.emit(
            Event::new(
                Severity::Info,
                Category::Process,
                Payload::ProcessCreated { name, priority },
            )
            .with_pid(pid),
        );
    }

    /// Record process terminated
    pub fn process_terminated(&self, pid: Pid, exit_code: Option<i32>) {
        self.emit(
            Event::new(
                Severity::Info,
                Category::Process,
                Payload::ProcessTerminated { exit_code },
            )
            .with_pid(pid),
        );
    }

    /// Record resource cleanup with detailed stats
    pub fn resource_cleanup(
        &self,
        pid: Pid,
        resources_freed: usize,
        bytes_freed: usize,
        cleanup_duration_micros: u64,
        by_type: std::collections::HashMap<String, usize>,
        errors: Vec<String>,
    ) {
        let severity = if errors.is_empty() {
            Severity::Info
        } else {
            Severity::Warn
        };

        // Emit unified cleanup event
        self.emit(
            Event::new(
                severity,
                Category::Resource,
                Payload::ResourceReclaimed {
                    resource: "unified".to_string(),
                    count: resources_freed as u64,
                },
            )
            .with_pid(pid),
        );

        // Track per-type cleanup metrics
        for (type_name, count) in &by_type {
            self.metrics
                .inc_counter(&format!("resource.{}.freed", type_name), *count as f64);
        }

        // Track cleanup timing
        self.metrics.observe_histogram(
            "resource.cleanup_duration_ms",
            cleanup_duration_micros as f64 / 1000.0,
        );

        // Track bytes freed
        if bytes_freed > 0 {
            self.metrics
                .inc_counter("resource.bytes_freed", bytes_freed as f64);
        }

        // Track errors
        if !errors.is_empty() {
            self.metrics
                .inc_counter("resource.cleanup_errors", errors.len() as f64);

            // Emit error events
            for _error in &errors {
                self.emit(
                    Event::new(
                        Severity::Error,
                        Category::Resource,
                        Payload::ResourceLeaked {
                            resource: "cleanup_error".to_string(),
                            count: 1,
                        },
                    )
                    .with_pid(pid),
                );
            }
        }
    }

    /// Record syscall execution
    pub fn syscall_exit(&self, pid: Pid, name: String, duration_us: u64, success: bool) {
        let result = if success {
            SyscallResult::Success
        } else {
            SyscallResult::Error
        };

        self.emit(
            Event::new(
                Severity::Debug,
                Category::Syscall,
                Payload::SyscallExit {
                    name,
                    duration_us,
                    result,
                },
            )
            .with_pid(pid),
        );
    }

    /// Record memory pressure
    pub fn memory_pressure(&self, usage_pct: u8, available_mb: u64) {
        let severity = if usage_pct > 90 {
            Severity::Error
        } else if usage_pct > 75 {
            Severity::Warn
        } else {
            Severity::Info
        };

        self.emit(Event::new(
            severity,
            Category::Memory,
            Payload::MemoryPressure {
                usage_pct,
                available_mb,
            },
        ));
    }

    /// Record slow operation
    pub fn slow_operation(&self, operation: String, duration_ms: u64, p99_ms: u64) {
        self.emit(Event::new(
            Severity::Warn,
            Category::Performance,
            Payload::OperationSlow {
                operation,
                duration_ms,
                p99_ms,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_emit() {
        let collector = Collector::new();

        let event = Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "test".to_string(),
                priority: 5,
            },
        );

        collector.emit(event);

        let stats = collector.stream_stats();
        assert!(stats.events_produced > 0);
    }

    #[test]
    fn test_collector_subscribe() {
        let collector = Collector::new();

        collector.process_created(123, "test".to_string(), 5);

        let mut sub = collector.subscribe();
        let events = collector.collect_events(&mut sub);

        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_collector_causality() {
        let collector = Collector::new();

        let event1 = Event::new(
            Severity::Info,
            Category::Process,
            Payload::ProcessCreated {
                name: "test".to_string(),
                priority: 5,
            },
        );

        let causality_id = collector.emit_causal(event1);

        let event2 = Event::new(
            Severity::Info,
            Category::Memory,
            Payload::MemoryAllocated {
                size: 1024,
                region_id: 1,
            },
        );

        collector.emit_in_chain(event2, causality_id);

        let mut sub = collector.subscribe();
        let events = collector.collect_events(&mut sub);

        let causal_events: Vec<_> = events
            .iter()
            .filter(|e| e.causality_id == Some(causality_id))
            .collect();

        assert_eq!(causal_events.len(), 2);
    }

    #[test]
    fn test_collector_query() {
        let collector = Collector::new();

        collector.process_created(123, "test".to_string(), 5);
        collector.memory_pressure(85, 100);

        let mut sub = collector.subscribe();

        let query = Query::new().category(Category::Memory);
        let result = collector.query(query, &mut sub);

        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_collector_metrics_integration() {
        let collector = Collector::new();

        collector.syscall_exit(123, "read".to_string(), 1000, true);

        let metrics = collector.metrics();
        assert!(metrics.counters.contains_key("syscall.total"));
    }
}
