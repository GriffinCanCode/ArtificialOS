/**
 * Performance Monitoring Utilities
 * Tracks and reports performance metrics for optimization
 */

import { logger } from './logger';

export interface PerformanceMetrics {
  name: string;
  duration: number;
  timestamp: number;
  metadata?: Record<string, any>;
}

class PerformanceMonitor {
  private metrics: PerformanceMetrics[] = [];
  private maxMetrics = 1000; // Keep last 1000 measurements
  private timers: Map<string, number> = new Map();

  /**
   * Start a performance timer
   */
  start(name: string, metadata?: Record<string, any>): void {
    const key = this.getKey(name, metadata);
    this.timers.set(key, performance.now());
  }

  /**
   * End a performance timer and record the duration
   */
  end(name: string, metadata?: Record<string, any>): number {
    const key = this.getKey(name, metadata);
    const startTime = this.timers.get(key);
    
    if (!startTime) {
      logger.warn('Performance timer not started', { name, component: 'PerformanceMonitor' });
      return 0;
    }

    const duration = performance.now() - startTime;
    this.timers.delete(key);

    // Record metric
    const metric: PerformanceMetrics = {
      name,
      duration,
      timestamp: Date.now(),
      metadata,
    };

    this.metrics.push(metric);

    // Trim old metrics
    if (this.metrics.length > this.maxMetrics) {
      this.metrics = this.metrics.slice(-this.maxMetrics);
    }

    // Log if duration exceeds threshold
    const threshold = this.getThreshold(name);
    if (duration > threshold) {
      logger.performance(name, duration, {
        ...metadata,
        component: 'PerformanceMonitor',
        threshold,
      });
    }

    return duration;
  }

  /**
   * Measure an async operation
   */
  async measure<T>(
    name: string,
    operation: () => Promise<T>,
    metadata?: Record<string, any>
  ): Promise<T> {
    this.start(name, metadata);
    try {
      const result = await operation();
      return result;
    } finally {
      this.end(name, metadata);
    }
  }

  /**
   * Measure a sync operation
   */
  measureSync<T>(
    name: string,
    operation: () => T,
    metadata?: Record<string, any>
  ): T {
    this.start(name, metadata);
    try {
      return operation();
    } finally {
      this.end(name, metadata);
    }
  }

  /**
   * Get statistics for a specific metric
   */
  getStats(name: string): {
    count: number;
    min: number;
    max: number;
    avg: number;
    p50: number;
    p95: number;
    p99: number;
  } | null {
    const relevantMetrics = this.metrics.filter(m => m.name === name);
    
    if (relevantMetrics.length === 0) {
      return null;
    }

    const durations = relevantMetrics.map(m => m.duration).sort((a, b) => a - b);
    const sum = durations.reduce((acc, d) => acc + d, 0);

    return {
      count: durations.length,
      min: durations[0],
      max: durations[durations.length - 1],
      avg: sum / durations.length,
      p50: durations[Math.floor(durations.length * 0.5)],
      p95: durations[Math.floor(durations.length * 0.95)],
      p99: durations[Math.floor(durations.length * 0.99)],
    };
  }

  /**
   * Get all recorded metrics
   */
  getAllMetrics(): PerformanceMetrics[] {
    return [...this.metrics];
  }

  /**
   * Clear all recorded metrics
   */
  clear(): void {
    this.metrics = [];
    this.timers.clear();
  }

  /**
   * Generate a performance report
   */
  getReport(): string {
    const metricNames = [...new Set(this.metrics.map(m => m.name))];
    
    let report = 'Performance Report\n';
    report += '==================\n\n';

    for (const name of metricNames) {
      const stats = this.getStats(name);
      if (stats) {
        report += `${name}:\n`;
        report += `  Count: ${stats.count}\n`;
        report += `  Avg: ${stats.avg.toFixed(2)}ms\n`;
        report += `  Min: ${stats.min.toFixed(2)}ms\n`;
        report += `  Max: ${stats.max.toFixed(2)}ms\n`;
        report += `  P95: ${stats.p95.toFixed(2)}ms\n`;
        report += `  P99: ${stats.p99.toFixed(2)}ms\n\n`;
      }
    }

    return report;
  }

  private getKey(name: string, metadata?: Record<string, any>): string {
    return metadata ? `${name}:${JSON.stringify(metadata)}` : name;
  }

  private getThreshold(name: string): number {
    // Define thresholds for different operations
    const thresholds: Record<string, number> = {
      'json_parse': 100,
      'component_render': 50,
      'ui_generation': 5000,
      'token_stream': 1000,
    };

    return thresholds[name] || 1000; // Default 1s threshold
  }
}

// Singleton instance
export const performanceMonitor = new PerformanceMonitor();

// Convenience functions
export const startPerf = (name: string, metadata?: Record<string, any>) =>
  performanceMonitor.start(name, metadata);

export const endPerf = (name: string, metadata?: Record<string, any>) =>
  performanceMonitor.end(name, metadata);

export const measurePerf = async <T>(
  name: string,
  operation: () => Promise<T>,
  metadata?: Record<string, any>
) => performanceMonitor.measure(name, operation, metadata);

export const measurePerfSync = <T>(
  name: string,
  operation: () => T,
  metadata?: Record<string, any>
) => performanceMonitor.measureSync(name, operation, metadata);

