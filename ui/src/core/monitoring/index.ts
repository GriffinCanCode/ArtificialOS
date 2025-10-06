/**
 * Monitoring Module
 * Centralized performance monitoring and metrics collection
 */

export { metricsCollector, Timer, measureAsync, measureSync } from "./metrics";
export { initWebVitals, getWebVitalsMetrics } from "./vitals";
export {
  getAllMetrics,
  fetchBackendMetrics,
  getMetricsSummary,
  logAllMetrics,
  downloadMetrics,
} from "./dashboard";
export type {
  MetricType,
  MetricValue,
  Histogram,
  HistogramStats,
  MetricsSnapshot,
  PerformanceMetrics,
  WebVitalsMetrics,
} from "./types";
