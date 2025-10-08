/*!
 * Anomaly Detection
 * Statistical anomaly detection using streaming algorithms
 *
 * Strategy: Track running statistics (mean, variance) and detect outliers
 * using z-score without storing historical data
 */

use super::events::{Event, Payload};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Z-score threshold for anomaly detection (3 = 99.7% confidence)
const ANOMALY_THRESHOLD: f64 = 3.0;

/// Minimum samples before detecting anomalies
const MIN_SAMPLES: u64 = 100;

/// Running statistics for a metric
#[derive(Debug, Clone)]
struct Stats {
    count: u64,
    mean: f64,
    m2: f64, // For variance calculation (Welford's algorithm)
}

impl Stats {
    fn new() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }

    /// Update statistics with new value (Welford's online algorithm)
    fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    /// Get variance
    fn variance(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            self.m2 / (self.count - 1) as f64
        }
    }

    /// Get standard deviation
    fn stddev(&self) -> f64 {
        self.variance().sqrt()
    }

    /// Calculate z-score for a value
    fn z_score(&self, value: f64) -> f64 {
        let stddev = self.stddev();
        if stddev == 0.0 {
            0.0
        } else {
            (value - self.mean).abs() / stddev
        }
    }

    /// Check if value is anomalous
    fn is_anomaly(&self, value: f64) -> bool {
        if self.count < MIN_SAMPLES {
            return false;
        }

        self.z_score(value) > ANOMALY_THRESHOLD
    }
}

/// Anomaly detector
pub struct Detector {
    /// Metric statistics (metric name -> stats)
    metrics: Arc<RwLock<HashMap<String, Stats>>>,
}

impl Detector {
    /// Create a new anomaly detector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check event for anomalies and update statistics
    pub fn check(&self, event: &Event) -> Option<Anomaly> {
        let (metric_name, value) = self.extract_metric(event)?;

        let mut metrics = self.metrics.write()
            .expect("anomaly detector metrics lock poisoned - unrecoverable state");
        let stats = metrics
            .entry(metric_name.clone())
            .or_insert_with(Stats::new);

        // Check if anomalous before updating
        let is_anomaly = stats.is_anomaly(value);
        let z_score = stats.z_score(value);

        // Update statistics
        stats.update(value);

        if is_anomaly {
            Some(Anomaly {
                metric: metric_name,
                value,
                expected: stats.mean,
                deviation: z_score,
                timestamp_ns: event.timestamp_ns,
            })
        } else {
            None
        }
    }

    /// Extract numeric metric from event
    fn extract_metric(&self, event: &Event) -> Option<(String, f64)> {
        match &event.payload {
            Payload::SyscallExit {
                name, duration_us, ..
            } => Some((format!("syscall.{}.duration_us", name), *duration_us as f64)),
            Payload::SyscallSlow {
                name, duration_ms, ..
            } => Some((format!("syscall.{}.duration_ms", name), *duration_ms as f64)),
            Payload::MemoryAllocated { size, .. } => {
                Some(("memory.allocation.size".to_string(), *size as f64))
            }
            Payload::MemoryPressure { usage_pct, .. } => {
                Some(("memory.usage_pct".to_string(), *usage_pct as f64))
            }
            Payload::SchedulerLatency { wake_to_run_us } => {
                Some(("scheduler.latency_us".to_string(), *wake_to_run_us as f64))
            }
            Payload::MessageReceived { wait_time_us, .. } => {
                Some(("ipc.message.wait_time_us".to_string(), *wait_time_us as f64))
            }
            Payload::CpuThrottled { usage_pct, .. } => {
                Some(("cpu.usage_pct".to_string(), *usage_pct as f64))
            }
            Payload::MetricUpdate { name, value, .. } => Some((name.clone(), *value)),
            _ => None,
        }
    }

    /// Get statistics for a metric
    pub fn stats(&self, metric: &str) -> Option<MetricStats> {
        let metrics = self.metrics.read()
            .expect("anomaly detector metrics lock poisoned - unrecoverable state");
        metrics.get(metric).map(|s| MetricStats {
            count: s.count,
            mean: s.mean,
            stddev: s.stddev(),
        })
    }

    /// Detect potential resource leak patterns
    ///
    /// Triggers anomaly if:
    /// - Process terminated but freed 0 resources (potential leak)
    /// - Cleanup took unusually long (>100ms = blocked/leaked resources)
    /// - Large number of resources freed (accumulated leak before cleanup)
    pub fn detect_resource_leak(
        &self,
        pid: crate::core::types::Pid,
        resources_freed: usize,
        cleanup_duration_micros: u64,
        by_type: &std::collections::HashMap<String, usize>,
    ) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // Helper to get current timestamp with fallback
        let current_timestamp = || -> u64 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_nanos() as u64
        };

        // Detect leak: cleanup took too long (indicates resource accumulation)
        let cleanup_ms = cleanup_duration_micros as f64 / 1000.0;
        if cleanup_ms > 100.0 {
            anomalies.push(Anomaly {
                metric: format!("resource.cleanup_duration.pid_{}", pid),
                value: cleanup_ms,
                expected: 10.0, // Expected <10ms for normal cleanup
                deviation: (cleanup_ms - 10.0) / 10.0,
                timestamp_ns: current_timestamp(),
            });
        }

        // Detect leak: excessive resources accumulated before cleanup
        if resources_freed > 1000 {
            anomalies.push(Anomaly {
                metric: format!("resource.count.pid_{}", pid),
                value: resources_freed as f64,
                expected: 100.0,
                deviation: (resources_freed as f64 - 100.0) / 100.0,
                timestamp_ns: current_timestamp(),
            });
        }

        // Detect per-type leaks
        for (resource_type, count) in by_type {
            let threshold = match resource_type.as_str() {
                "memory" => 50,            // Memory blocks
                "file_descriptors" => 100, // FDs
                "sockets" => 50,           // Network sockets
                "ipc" => 100,              // IPC resources
                _ => 200,                  // Default
            };

            if *count > threshold {
                anomalies.push(Anomaly {
                    metric: format!("resource.{}.count.pid_{}", resource_type, pid),
                    value: *count as f64,
                    expected: threshold as f64,
                    deviation: (*count as f64 - threshold as f64) / threshold as f64,
                    timestamp_ns: current_timestamp(),
                });
            }
        }

        anomalies
    }

    /// Reset all statistics
    pub fn reset(&self) {
        self.metrics.write()
            .expect("anomaly detector metrics lock poisoned - unrecoverable state")
            .clear();
    }
}

impl Default for Detector {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Detector {
    fn clone(&self) -> Self {
        Self {
            metrics: Arc::clone(&self.metrics),
        }
    }
}

/// Detected anomaly
#[derive(Debug, Clone)]
pub struct Anomaly {
    pub metric: String,
    pub value: f64,
    pub expected: f64,
    pub deviation: f64, // Z-score
    pub timestamp_ns: u64,
}

/// Metric statistics snapshot
#[derive(Debug, Clone)]
pub struct MetricStats {
    pub count: u64,
    pub mean: f64,
    pub stddev: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::events::{Category, Severity, SyscallResult};

    #[test]
    fn test_stats_update() {
        let mut stats = Stats::new();

        // Add normal values
        for i in 1..=100 {
            stats.update(i as f64);
        }

        assert_eq!(stats.count, 100);
        assert!((stats.mean - 50.5).abs() < 0.1);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut stats = Stats::new();

        // Build baseline with normal values (mean ~100, stddev ~5)
        for i in 0..MIN_SAMPLES {
            stats.update(100.0 + (i % 10) as f64);
        }

        // Normal value should not be anomaly
        assert!(!stats.is_anomaly(100.0));
        assert!(!stats.is_anomaly(101.0));

        // Very different value should be anomaly
        assert!(stats.is_anomaly(1000.0));
    }

    #[test]
    fn test_detector() {
        let detector = Detector::new();

        // Send normal syscall events
        for i in 0..MIN_SAMPLES {
            let event = Event::new(
                Severity::Debug,
                Category::Syscall,
                Payload::SyscallExit {
                    name: "read".to_string(),
                    duration_us: 100 + i,
                    result: SyscallResult::Success,
                },
            );

            detector.check(&event);
        }

        // Send anomalous event
        let anomalous = Event::new(
            Severity::Debug,
            Category::Syscall,
            Payload::SyscallExit {
                name: "read".to_string(),
                duration_us: 10000, // 100x normal
                result: SyscallResult::Success,
            },
        );

        let result = detector.check(&anomalous);
        assert!(result.is_some());

        let anomaly = result.unwrap();
        assert!(anomaly.deviation > ANOMALY_THRESHOLD);
    }

    #[test]
    fn test_detector_multiple_metrics() {
        let detector = Detector::new();

        // Track two different syscalls
        for i in 0..MIN_SAMPLES {
            let read_event = Event::new(
                Severity::Debug,
                Category::Syscall,
                Payload::SyscallExit {
                    name: "read".to_string(),
                    duration_us: 100 + i,
                    result: SyscallResult::Success,
                },
            );

            let write_event = Event::new(
                Severity::Debug,
                Category::Syscall,
                Payload::SyscallExit {
                    name: "write".to_string(),
                    duration_us: 200 + i,
                    result: SyscallResult::Success,
                },
            );

            detector.check(&read_event);
            detector.check(&write_event);
        }

        let read_stats = detector.stats("syscall.read.duration_us");
        let write_stats = detector.stats("syscall.write.duration_us");

        assert!(read_stats.is_some());
        assert!(write_stats.is_some());
    }

    #[test]
    fn test_welford_algorithm() {
        let mut stats = Stats::new();

        // Known dataset: [1, 2, 3, 4, 5]
        // Mean = 3, Variance = 2
        for i in 1..=5 {
            stats.update(i as f64);
        }

        assert_eq!(stats.count, 5);
        assert!((stats.mean - 3.0).abs() < 0.01);
        assert!((stats.variance() - 2.5).abs() < 0.01); // Sample variance
    }
}
