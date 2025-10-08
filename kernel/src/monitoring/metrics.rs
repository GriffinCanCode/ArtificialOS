/*!
 * Metrics Collection
 * Lightweight performance metrics collector
 */

use crate::core::serde::is_zero_u64;
use crate::core::{ShardManager, WorkloadProfile};
use ahash::RandomState;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Individual metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct MetricValue {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: u64,
}

/// Histogram data structure
#[derive(Debug, Clone)]
struct Histogram {
    buckets: Vec<f64>,
    counts: Vec<u64>,
    sum: f64,
    count: u64,
}

impl Histogram {
    #[allow(dead_code)]
    fn new(buckets: Vec<f64>) -> Self {
        let counts = vec![0; buckets.len()];
        Self {
            buckets,
            counts,
            sum: 0.0,
            count: 0,
        }
    }

    fn observe(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;

        for (i, &bucket) in self.buckets.iter().enumerate() {
            if value <= bucket {
                self.counts[i] += 1;
            }
        }
    }

    fn percentile(&self, p: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let target = (self.count as f64 * p) as u64;
        for (i, &count) in self.counts.iter().enumerate() {
            if count >= target {
                return self.buckets[i];
            }
        }
        self.buckets.last().copied().unwrap_or(0.0)
    }
}

/// Metrics collector
///
/// # Performance
/// - Cache-line aligned to prevent false sharing in high-frequency metric updates
#[repr(C, align(64))]
pub struct MetricsCollector {
    counters: Arc<DashMap<String, f64, RandomState>>,
    gauges: Arc<DashMap<String, f64, RandomState>>,
    histograms: Arc<DashMap<String, Histogram, RandomState>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        // CPU-topology-aware shard counts for optimal concurrent performance
        let shards = ShardManager::shards(WorkloadProfile::LowContention); // metrics: infrequent updates, mostly reads
        Self {
            counters: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                shards,
            )),
            gauges: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                shards,
            )),
            histograms: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                0,
                RandomState::new(),
                shards,
            )),
            start_time: Instant::now(),
        }
    }

    /// Increment a counter
    pub fn inc_counter(&self, name: &str, value: f64) {
        // Use entry API for atomic counter increment
        self.counters
            .entry(name.to_string())
            .and_modify(|v| *v += value)
            .or_insert(value);
    }

    /// Set a gauge value
    pub fn set_gauge(&self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    /// Observe a value in a histogram
    pub fn observe_histogram(&self, name: &str, value: f64) {
        // Use entry API for atomic histogram update
        self.histograms
            .entry(name.to_string())
            .and_modify(|hist| hist.observe(value))
            .or_insert_with(|| {
                let mut hist =
                    Histogram::new(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]);
                hist.observe(value);
                hist
            });
    }

    /// Record operation duration
    pub fn record_duration(&self, name: &str, duration: Duration) {
        self.observe_histogram(name, duration.as_secs_f64());
    }

    /// Get snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let counters: HashMap<String, f64> = self
            .counters
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();

        let gauges: HashMap<String, f64> = self
            .gauges
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();

        let histogram_stats: HashMap<String, HistogramStats> = self
            .histograms
            .iter()
            .map(|entry| {
                let name = entry.key();
                let hist = entry.value();
                let stats = HistogramStats {
                    count: hist.count,
                    sum: hist.sum,
                    avg: if hist.count > 0 {
                        hist.sum / hist.count as f64
                    } else {
                        0.0
                    },
                    p50: hist.percentile(0.50),
                    p95: hist.percentile(0.95),
                    p99: hist.percentile(0.99),
                };
                (name.clone(), stats)
            })
            .collect();

        MetricsSnapshot {
            counters,
            gauges,
            histograms: histogram_stats,
            uptime_secs: self.start_time.elapsed().as_secs(),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.counters.clear();
        self.gauges.clear();
        self.histograms.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Histogram statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HistogramStats {
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub count: u64,
    pub sum: f64,
    pub avg: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

/// Snapshot of all metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MetricsSnapshot {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub counters: HashMap<String, f64>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub gauges: HashMap<String, f64>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub histograms: HashMap<String, HistogramStats>,
    pub uptime_secs: u64,
}

/// Timer for measuring operation duration
#[allow(dead_code)]
pub struct Timer {
    start: Instant,
    name: String,
    collector: Arc<MetricsCollector>,
}

impl Timer {
    #[allow(dead_code)]
    pub fn new(name: String, collector: Arc<MetricsCollector>) -> Self {
        Self {
            start: Instant::now(),
            name,
            collector,
        }
    }

    #[allow(dead_code)]
    pub fn stop(self) -> Duration {
        let duration = self.start.elapsed();
        self.collector.record_duration(&self.name, duration);
        duration
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.collector.record_duration(&self.name, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let collector = MetricsCollector::new();
        collector.inc_counter("test", 1.0);
        collector.inc_counter("test", 2.0);

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.counters.get("test"), Some(&3.0));
    }

    #[test]
    fn test_gauge() {
        let collector = MetricsCollector::new();
        collector.set_gauge("memory", 100.0);
        collector.set_gauge("memory", 200.0);

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.gauges.get("memory"), Some(&200.0));
    }

    #[test]
    fn test_histogram() {
        let collector = MetricsCollector::new();
        collector.observe_histogram("latency", 0.1);
        collector.observe_histogram("latency", 0.2);
        collector.observe_histogram("latency", 0.3);

        let snapshot = collector.snapshot();
        let stats = snapshot.histograms.get("latency").unwrap();
        assert_eq!(stats.count, 3);
        assert!((stats.avg - 0.2).abs() < 0.01);
    }
}
