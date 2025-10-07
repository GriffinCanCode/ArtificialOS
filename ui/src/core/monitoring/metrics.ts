/**
 * Metrics Collection
 * Centralized performance metrics tracking with Prometheus-compatible format
 */

import type { MetricType, MetricValue, Histogram, MetricsSnapshot, HistogramStats } from "./types";

class MetricsCollector {
  private counters: Map<string, number> = new Map();
  private gauges: Map<string, number> = new Map();
  private histograms: Map<string, Histogram> = new Map();
  private startTime: number = Date.now();

  /**
   * Increment a counter
   */
  incCounter(name: string, value: number = 1, labels?: Record<string, string>): void {
    const key = this.makeKey(name, labels);
    const current = this.counters.get(key) || 0;
    this.counters.set(key, current + value);
  }

  /**
   * Set a gauge value
   */
  setGauge(name: string, value: number, labels?: Record<string, string>): void {
    const key = this.makeKey(name, labels);
    this.gauges.set(key, value);
  }

  /**
   * Increment a gauge
   */
  incGauge(name: string, value: number = 1, labels?: Record<string, string>): void {
    const key = this.makeKey(name, labels);
    const current = this.gauges.get(key) || 0;
    this.gauges.set(key, current + value);
  }

  /**
   * Decrement a gauge
   */
  decGauge(name: string, value: number = 1, labels?: Record<string, string>): void {
    this.incGauge(name, -value, labels);
  }

  /**
   * Observe a value in a histogram
   */
  observeHistogram(name: string, value: number, labels?: Record<string, string>): void {
    const key = this.makeKey(name, labels);
    let histogram = this.histograms.get(key);

    if (!histogram) {
      histogram = {
        buckets: [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
        counts: new Array(12).fill(0),
        sum: 0,
        count: 0,
      };
      this.histograms.set(key, histogram);
    }

    histogram.sum += value;
    histogram.count += 1;

    // Update bucket counts
    for (let i = 0; i < histogram.buckets.length; i++) {
      if (value <= histogram.buckets[i]) {
        histogram.counts[i] += 1;
      }
    }
  }

  /**
   * Record operation duration
   */
  recordDuration(name: string, durationMs: number, labels?: Record<string, string>): void {
    this.observeHistogram(name, durationMs / 1000, labels);
  }

  /**
   * Get all metrics in Prometheus format
   */
  getMetricsPrometheus(): string {
    let output = "";

    // Counters
    for (const [key, value] of this.counters.entries()) {
      const { name, labels } = this.parseKey(key);
      output += `# TYPE ui_${name} counter\n`;
      output += `ui_${name}${this.formatLabels(labels)} ${value}\n`;
    }

    // Gauges
    for (const [key, value] of this.gauges.entries()) {
      const { name, labels } = this.parseKey(key);
      output += `# TYPE ui_${name} gauge\n`;
      output += `ui_${name}${this.formatLabels(labels)} ${value}\n`;
    }

    // Histograms
    for (const [key, histogram] of this.histograms.entries()) {
      const { name, labels } = this.parseKey(key);
      output += `# TYPE ui_${name} summary\n`;
      output += `ui_${name}_sum${this.formatLabels(labels)} ${histogram.sum}\n`;
      output += `ui_${name}_count${this.formatLabels(labels)} ${histogram.count}\n`;

      const percentiles = this.calculatePercentiles(histogram);
      output += `ui_${name}${this.formatLabels({ ...labels, quantile: "0.5" })} ${percentiles.p50}\n`;
      output += `ui_${name}${this.formatLabels({ ...labels, quantile: "0.95" })} ${percentiles.p95}\n`;
      output += `ui_${name}${this.formatLabels({ ...labels, quantile: "0.99" })} ${percentiles.p99}\n`;
    }

    // Uptime
    output += `# TYPE ui_uptime_seconds gauge\n`;
    output += `ui_uptime_seconds ${(Date.now() - this.startTime) / 1000}\n`;

    return output;
  }

  /**
   * Get all metrics as JSON
   */
  getMetricsJSON(): MetricsSnapshot {
    const histograms: Record<string, HistogramStats> = {};
    for (const [key, histogram] of this.histograms.entries()) {
      const percentiles = this.calculatePercentiles(histogram);
      histograms[key] = {
        count: histogram.count,
        sum: histogram.sum,
        avg: histogram.count > 0 ? histogram.sum / histogram.count : 0,
        ...percentiles,
      };
    }

    return {
      counters: Object.fromEntries(this.counters),
      gauges: Object.fromEntries(this.gauges),
      histograms,
      uptime_seconds: (Date.now() - this.startTime) / 1000,
    };
  }

  /**
   * Clear all metrics
   */
  clear(): void {
    this.counters.clear();
    this.gauges.clear();
    this.histograms.clear();
  }

  /**
   * Make a unique key for a metric with labels
   */
  private makeKey(name: string, labels?: Record<string, string>): string {
    if (!labels || Object.keys(labels).length === 0) {
      return name;
    }
    return `${name}{${JSON.stringify(labels)}}`;
  }

  /**
   * Parse a key back into name and labels
   */
  private parseKey(key: string): { name: string; labels: Record<string, string> } {
    const match = key.match(/^([^{]+)(?:\{(.+)\})?$/);
    if (!match) {
      return { name: key, labels: {} };
    }

    const [, name, labelsStr] = match;
    const labels = labelsStr ? JSON.parse(labelsStr) : {};
    return { name, labels };
  }

  /**
   * Format labels for Prometheus format
   */
  private formatLabels(labels: Record<string, string>): string {
    if (!labels || Object.keys(labels).length === 0) {
      return "";
    }

    const pairs = Object.entries(labels).map(([k, v]) => `${k}="${v}"`);
    return `{${pairs.join(",")}}`;
  }

  /**
   * Calculate percentiles from histogram
   */
  private calculatePercentiles(histogram: Histogram): {
    p50: number;
    p95: number;
    p99: number;
  } {
    if (histogram.count === 0) {
      return { p50: 0, p95: 0, p99: 0 };
    }

    const findPercentile = (p: number): number => {
      const target = histogram.count * p;
      for (let i = 0; i < histogram.counts.length; i++) {
        if (histogram.counts[i] >= target) {
          return histogram.buckets[i];
        }
      }
      return histogram.buckets[histogram.buckets.length - 1];
    };

    return {
      p50: findPercentile(0.5),
      p95: findPercentile(0.95),
      p99: findPercentile(0.99),
    };
  }
}

// Singleton instance
export const metricsCollector = new MetricsCollector();

/**
 * Timer for measuring operation duration
 */
export class Timer {
  private start: number;
  private name: string;
  private labels?: Record<string, string>;

  constructor(name: string, labels?: Record<string, string>) {
    this.start = performance.now();
    this.name = name;
    this.labels = labels;
  }

  /**
   * Stop the timer and record the duration
   */
  stop(): number {
    const duration = performance.now() - this.start;
    metricsCollector.recordDuration(this.name, duration, this.labels);
    return duration;
  }
}

/**
 * Measure an async operation
 */
export async function measureAsync<T>(
  name: string,
  operation: () => Promise<T>,
  labels?: Record<string, string>
): Promise<T> {
  const timer = new Timer(name, labels);
  try {
    return await operation();
  } finally {
    timer.stop();
  }
}

/**
 * Measure a sync operation
 */
export function measureSync<T>(
  name: string,
  operation: () => T,
  labels?: Record<string, string>
): T {
  const timer = new Timer(name, labels);
  try {
    return operation();
  } finally {
    timer.stop();
  }
}
