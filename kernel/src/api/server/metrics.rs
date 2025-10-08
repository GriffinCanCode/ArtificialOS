/*!
 * Metrics API
 * gRPC endpoint for metrics exposure
 */

use crate::monitoring::MetricsCollector;
use std::sync::Arc;
use tonic::Status;

/// Metrics service implementation
pub struct MetricsService {
    collector: Arc<MetricsCollector>,
}

impl MetricsService {
    pub fn new(collector: Arc<MetricsCollector>) -> Self {
        Self { collector }
    }

    /// Get metrics snapshot as JSON
    pub fn get_metrics_json(&self) -> Result<String, Status> {
        let snapshot = self.collector.snapshot();
        serde_json::to_string(&snapshot)
            .map_err(|e| Status::internal(format!("Failed to serialize metrics: {}", e).into()))
    }

    /// Get metrics in Prometheus format
    pub fn get_metrics_prometheus(&self) -> String {
        let snapshot = self.collector.snapshot();
        let mut output = String::new();

        // Counters
        for (name, value) in snapshot.counters {
            output.push_str(&format!("# TYPE kernel_{} counter\n", name));
            output.push_str(&format!("kernel_{} {}\n", name, value));
        }

        // Gauges
        for (name, value) in snapshot.gauges {
            output.push_str(&format!("# TYPE kernel_{} gauge\n", name));
            output.push_str(&format!("kernel_{} {}\n", name, value));
        }

        // Histograms
        for (name, stats) in snapshot.histograms {
            output.push_str(&format!("# TYPE kernel_{} summary\n", name));
            output.push_str(&format!("kernel_{}_sum {}\n", name, stats.sum));
            output.push_str(&format!("kernel_{}_count {}\n", name, stats.count));
            output.push_str(&format!(
                "kernel_{}{{quantile=\"0.5\"}} {}\n",
                name, stats.p50
            ));
            output.push_str(&format!(
                "kernel_{}{{quantile=\"0.95\"}} {}\n",
                name, stats.p95
            ));
            output.push_str(&format!(
                "kernel_{}{{quantile=\"0.99\"}} {}\n",
                name, stats.p99
            ));
        }

        // Uptime
        output.push_str("# TYPE kernel_uptime_seconds gauge\n");
        output.push_str(&format!("kernel_uptime_seconds {}\n", snapshot.uptime_secs));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_json() {
        let collector = Arc::new(MetricsCollector::new());
        collector.inc_counter("test", 1.0);

        let service = MetricsService::new(collector);
        let json = service.get_metrics_json().unwrap();

        assert!(json.contains("test"));
    }

    #[test]
    fn test_metrics_prometheus() {
        let collector = Arc::new(MetricsCollector::new());
        collector.set_gauge("memory_usage", 100.0);

        let service = MetricsService::new(collector);
        let prom = service.get_metrics_prometheus();

        assert!(prom.contains("kernel_memory_usage"));
        assert!(prom.contains("gauge"));
    }
}
