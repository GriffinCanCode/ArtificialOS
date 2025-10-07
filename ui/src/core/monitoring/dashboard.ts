/**
 * Metrics Dashboard
 * Central location for viewing all system metrics
 */

import { metricsCollector } from "./metrics";
import { getWebVitalsMetrics } from "./vitals";
import { formatDate } from "../utils/dates";
import type { MetricsSnapshot } from "./types";

export interface SystemMetrics {
  timestamp: number;
  ui: MetricsSnapshot;
  webVitals: Record<string, any>;
  backend?: any;
  kernel?: any;
  aiService?: any;
}

/**
 * Fetch metrics from backend
 */
export async function fetchBackendMetrics(): Promise<any> {
  try {
    const response = await fetch("http://localhost:8000/metrics/json");
    if (!response.ok) return null;
    return await response.json();
  } catch (error) {
    console.error("Failed to fetch backend metrics:", error);
    return null;
  }
}

/**
 * Get all system metrics from all services
 */
export async function getAllMetrics(): Promise<SystemMetrics> {
  const backendMetrics = await fetchBackendMetrics();

  return {
    timestamp: Date.now(),
    ui: metricsCollector.getMetricsJSON(),
    webVitals: getWebVitalsMetrics(),
    backend: backendMetrics?.backend,
    kernel: backendMetrics?.kernel,
    aiService: backendMetrics?.ai_service,
  };
}

/**
 * Export metrics to console (for debugging)
 */
export function logAllMetrics(): void {
  console.group("📊 AgentOS Metrics");

  console.group("🖥️ UI Metrics");
  console.table(metricsCollector.getMetricsJSON());
  console.groupEnd();

  console.group("⚡ Web Vitals");
  console.table(getWebVitalsMetrics());
  console.groupEnd();

  console.groupEnd();
}

/**
 * Export metrics as downloadable JSON file
 */
export function downloadMetrics(): void {
  getAllMetrics().then((metrics) => {
    const blob = new Blob([JSON.stringify(metrics, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `agentos-metrics-${formatDate(new Date(), "yyyy-MM-dd-HHmmss")}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  });
}

/**
 * Get metrics summary for quick overview
 */
export function getMetricsSummary() {
  const metrics = metricsCollector.getMetricsJSON();

  // Calculate totals
  let totalToolExecutions = 0;
  let totalErrors = 0;

  for (const [key, value] of Object.entries(metrics.counters)) {
    if (key.includes("tool_executions_total")) {
      totalToolExecutions += value as number;
    }
    if (key.includes("error")) {
      totalErrors += value as number;
    }
  }

  // Calculate average latencies
  const latencies: number[] = [];
  for (const [key, stats] of Object.entries(metrics.histograms)) {
    if (key.includes("duration") || key.includes("latency")) {
      latencies.push((stats as { avg: number }).avg);
    }
  }

  const avgLatency =
    latencies.length > 0 ? latencies.reduce((a, b) => a + b, 0) / latencies.length : 0;

  return {
    totalToolExecutions,
    totalErrors,
    errorRate: totalToolExecutions > 0 ? totalErrors / totalToolExecutions : 0,
    avgLatencyMs: avgLatency * 1000,
    uptime: metrics.uptime_seconds,
  };
}

// Make functions available globally for console access
if (typeof window !== "undefined") {
  (window as any).agentOSMetrics = {
    getAll: getAllMetrics,
    getSummary: getMetricsSummary,
    log: logAllMetrics,
    download: downloadMetrics,
    openDashboard: () => window.open("http://localhost:8000/metrics/dashboard", "_blank"),
  };

  console.log(
    "%c🚀 AgentOS Metrics",
    "font-size: 16px; font-weight: bold; color: #667eea;",
    "\n\nAccess metrics via:\n" +
      "  • agentOSMetrics.getAll() - Get all metrics\n" +
      "  • agentOSMetrics.getSummary() - Get summary\n" +
      "  • agentOSMetrics.log() - Log to console\n" +
      "  • agentOSMetrics.download() - Download JSON\n" +
      "  • agentOSMetrics.openDashboard() - Open web dashboard"
  );
}
