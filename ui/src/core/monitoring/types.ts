/**
 * Monitoring Types
 * Type definitions for metrics and performance monitoring
 */

export type MetricType = "counter" | "gauge" | "histogram" | "summary";

export interface MetricValue {
  name: string;
  value: number;
  labels?: Record<string, string>;
  timestamp: number;
}

export interface Histogram {
  buckets: number[];
  counts: number[];
  sum: number;
  count: number;
}

export interface HistogramStats {
  count: number;
  sum: number;
  avg: number;
  p50: number;
  p95: number;
  p99: number;
}

export interface MetricsSnapshot {
  counters: Record<string, number>;
  gauges: Record<string, number>;
  histograms: Record<string, HistogramStats>;
  uptime_seconds: number;
}

export interface PerformanceMetrics {
  name: string;
  duration: number;
  timestamp: number;
  metadata?: Record<string, any>;
}

export interface WebVitalsMetrics {
  // Core Web Vitals
  CLS: number; // Cumulative Layout Shift
  FID: number; // First Input Delay
  LCP: number; // Largest Contentful Paint

  // Other metrics
  FCP: number; // First Contentful Paint
  TTFB: number; // Time to First Byte
  INP: number; // Interaction to Next Paint
}
