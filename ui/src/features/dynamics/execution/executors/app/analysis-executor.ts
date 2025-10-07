/**
 * System Analysis Tool Executor
 * Handles system metrics collection and display
 */

import { formatBytes, formatDuration, formatCompact } from "../../../../../core/utils/math";
import { logger } from "../../../../../core/utils/monitoring/logger";
import { getAllMetrics } from "../../../../../core/monitoring";
import { formatTime } from "../../../../../core/utils/dates";
import { ExecutorContext, AsyncExecutor } from "../core/types";
import { toRgbaString, UI_COLORS, ALPHA_VALUES } from "../../../../../core/utils/color";

// Centralized color constants from core utilities
const METRIC_COLORS = {
  success: "#4ade80", // green-400
  warning: "#fbbf24", // yellow-400
  error: "#f87171", // red-400
  info: "#60a5fa", // blue-400
  neutral: "#ffffff", // white
} as const;

export class AnalysisExecutor implements AsyncExecutor {
  private context: ExecutorContext;
  private refreshInterval: number | null = null;
  private metricsHistory: any[] = [];
  private readonly MAX_HISTORY = 60; // Keep last 60 data points

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  async execute(action: string, _params: Record<string, any>): Promise<any> {
    switch (action) {
      case "refresh":
        return await this.refreshMetrics();

      case "start_auto_refresh":
        return this.startAutoRefresh();

      case "stop_auto_refresh":
        return this.stopAutoRefresh();

      case "export":
        return await this.exportMetrics();

      default:
        logger.warn("Unknown analysis action", { component: "AnalysisExecutor", action });
        return null;
    }
  }

  /**
   * Refresh all system metrics
   */
  private async refreshMetrics(): Promise<boolean> {
    try {
      logger.info("Refreshing system metrics", { component: "AnalysisExecutor" });

      // Fetch metrics from all services
      const allMetrics = await getAllMetrics();

      // Add to history
      this.metricsHistory.push({
        timestamp: Date.now(),
        metrics: allMetrics,
      });

      // Trim history to max size
      if (this.metricsHistory.length > this.MAX_HISTORY) {
        this.metricsHistory = this.metricsHistory.slice(-this.MAX_HISTORY);
      }

      // Update summary cards
      this.updateSummaryCards(allMetrics);

      // Update charts
      this.updateCharts();

      // Update detailed metrics lists
      this.updateBackendMetrics(allMetrics.backend);
      this.updateKernelMetrics(allMetrics.kernel);
      this.updateAIMetrics(allMetrics.aiService);

      // Update last refresh time
      this.context.componentState.set("last-update", `Updated: ${formatTime(new Date(), false)}`);

      logger.info("Metrics refreshed successfully", { component: "AnalysisExecutor" });
      return true;
    } catch (error) {
      logger.error("Failed to refresh metrics", error as Error, {
        component: "AnalysisExecutor",
      });
      this.context.componentState.set("last-update", "Update failed");
      return false;
    }
  }

  /**
   * Start auto-refresh timer (every 5 seconds)
   */
  private startAutoRefresh(): boolean {
    if (this.refreshInterval) {
      logger.debug("Auto-refresh already running", { component: "AnalysisExecutor" });
      return false;
    }

    // Initial refresh
    this.refreshMetrics();

    // Set up interval
    this.refreshInterval = window.setInterval(() => {
      this.refreshMetrics();
    }, 5000); // 5 seconds

    logger.info("Auto-refresh started (5s interval)", { component: "AnalysisExecutor" });
    return true;
  }

  /**
   * Stop auto-refresh timer
   */
  private stopAutoRefresh(): boolean {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
      this.refreshInterval = null;
      logger.info("Auto-refresh stopped", { component: "AnalysisExecutor" });
      return true;
    }
    return false;
  }

  /**
   * Export metrics as JSON file
   */
  private async exportMetrics(): Promise<boolean> {
    try {
      const allMetrics = await getAllMetrics();
      const blob = new Blob([JSON.stringify(allMetrics, null, 2)], {
        type: "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `system-metrics-${Date.now()}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      logger.info("Metrics exported", { component: "AnalysisExecutor" });
      return true;
    } catch (error) {
      logger.error("Failed to export metrics", error as Error, {
        component: "AnalysisExecutor",
      });
      return false;
    }
  }

  /**
   * Update summary cards with high-level metrics
   */
  private updateSummaryCards(metrics: any): void {
    // Get backend metrics for details, summary is at root level
    const backend = metrics.backend || {};
    const summary = metrics.summary || {};

    // Uptime - prefer backend uptime, fallback to UI uptime
    const uptimeSeconds =
      backend.uptime_seconds || summary.uptime_seconds || metrics.ui?.uptime_seconds || 0;
    this.context.componentState.set("uptime-value", this.formatUptime(uptimeSeconds));

    // Total requests - from summary
    const totalRequests = summary.total_requests || 0;
    this.context.componentState.set("requests-value", this.formatNumber(totalRequests));

    // Average latency - from summary
    const avgLatency = summary.average_latency_ms || 0;
    this.context.componentState.set("latency-value", `${avgLatency.toFixed(1)}ms`);

    // Active apps/connections - from summary
    const activeApps = summary.active_connections || backend.active_connections || 0;
    this.context.componentState.set("apps-value", activeApps.toString());
  }

  /**
   * Update all charts with historical data
   */
  private updateCharts(): void {
    if (this.metricsHistory.length === 0) return;

    const formatTimestamp = (ts: number) => {
      const date = new Date(ts);
      return `${date.getHours()}:${date.getMinutes().toString().padStart(2, "0")}`;
    };

    // Performance chart (overview)
    const performanceData = this.metricsHistory.map((h) => ({
      timestamp: formatTimestamp(h.timestamp),
      latency: h.metrics.summary?.average_latency_ms || 0,
    }));
    this.context.componentState.set("performance-chart.data", performanceData);

    // Requests chart (overview)
    const requestsData = this.metricsHistory.map((h) => ({
      timestamp: formatTimestamp(h.timestamp),
      requests: h.metrics.summary?.requests_per_second || 0,
    }));
    this.context.componentState.set("requests-chart.data", requestsData);

    // Service distribution (overview) - latest snapshot
    const latest = this.metricsHistory[this.metricsHistory.length - 1].metrics;
    const serviceDistribution = [
      { name: "Backend", value: latest.backend?.total_requests || 0 },
      { name: "Kernel", value: latest.kernel?.total_syscalls || 0 },
      { name: "AI Service", value: latest.aiService?.total_requests || 0 },
      { name: "UI", value: latest.ui?.counters?.tool_executions_total || 0 },
    ].filter((s) => s.value > 0);
    this.context.componentState.set("service-distribution.data", serviceDistribution);

    // Backend latency chart (multi-series)
    const backendLatencyData = this.metricsHistory.map((h) => {
      const backend = h.metrics.backend || {};
      return {
        timestamp: formatTimestamp(h.timestamp),
        p50: backend.p50_latency_ms || 0,
        p95: backend.p95_latency_ms || 0,
        p99: backend.p99_latency_ms || 0,
      };
    });
    this.context.componentState.set("backend-latency-chart.data", backendLatencyData);

    // Kernel operations chart (bar)
    const kernelOps = latest.kernel?.operations || {};
    const kernelOpsData = Object.entries(kernelOps)
      .map(([operation, count]) => ({
        operation: operation.replace(/_/g, " "),
        count: count as number,
      }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10);
    this.context.componentState.set("kernel-ops-chart.data", kernelOpsData);

    // AI throughput chart
    const aiThroughputData = this.metricsHistory.map((h) => ({
      timestamp: formatTimestamp(h.timestamp),
      requests: h.metrics.aiService?.requests_per_minute || 0,
    }));
    this.context.componentState.set("ai-throughput-chart.data", aiThroughputData);

    // AI cache chart (pie)
    const aiCache = latest.aiService?.cache || {};
    const aiCacheData = [
      { name: "Hits", value: aiCache.hits || 0 },
      { name: "Misses", value: aiCache.misses || 0 },
    ].filter((s) => s.value > 0);
    this.context.componentState.set("ai-cache-chart.data", aiCacheData);
  }

  /**
   * Update backend metrics list
   */
  private updateBackendMetrics(backend: any): void {
    if (!backend) {
      this.updateMetricsList("backend-metrics-list", [
        { label: "Status", value: "Unavailable", color: "#f87171" },
      ]);
      return;
    }

    const items = [];

    // Convert backend metrics to list items
    for (const [key, value] of Object.entries(backend)) {
      if (typeof value === "object") continue; // Skip nested objects

      const label = this.formatLabel(key);
      const formattedValue = this.formatMetricValue(key, value);
      const color = this.getMetricColor(key, value);

      items.push({ label, value: formattedValue, color });
    }

    this.updateMetricsList("backend-metrics-list", items);
  }

  /**
   * Update kernel metrics list
   */
  private updateKernelMetrics(kernel: any): void {
    if (!kernel) {
      this.updateMetricsList("kernel-metrics-list", [
        { label: "Status", value: "Unavailable", color: "#f87171" },
      ]);
      return;
    }

    const items = [];

    for (const [key, value] of Object.entries(kernel)) {
      if (typeof value === "object") continue;

      const label = this.formatLabel(key);
      const formattedValue = this.formatMetricValue(key, value);
      const color = this.getMetricColor(key, value);

      items.push({ label, value: formattedValue, color });
    }

    this.updateMetricsList("kernel-metrics-list", items);
  }

  /**
   * Update AI service metrics list
   */
  private updateAIMetrics(aiService: any): void {
    if (!aiService) {
      this.updateMetricsList("ai-metrics-list", [
        { label: "Status", value: "Unavailable", color: "#f87171" },
      ]);
      return;
    }

    const items = [];

    for (const [key, value] of Object.entries(aiService)) {
      if (typeof value === "object") continue;

      const label = this.formatLabel(key);
      const formattedValue = this.formatMetricValue(key, value);
      const color = this.getMetricColor(key, value);

      items.push({ label, value: formattedValue, color });
    }

    this.updateMetricsList("ai-metrics-list", items);
  }

  /**
   * Update a metrics list via postMessage
   */
  private updateMetricsList(
    listId: string,
    items: Array<{ label: string; value: string; color: string }>
  ): void {
    const listItems = items.map((item, index) => ({
      type: "container",
      id: `${listId}-item-${index}`,
      props: {
        layout: "horizontal",
        spacing: "none",
        padding: "small",
        align: "center",
        justify: "between",
        style: {
          padding: "0.75rem",
          borderBottom: `1px solid ${toRgbaString(UI_COLORS.text.primary, ALPHA_VALUES.ghost)}`,
        },
      },
      children: [
        {
          type: "text",
          id: `${listId}-label-${index}`,
          props: {
            content: item.label,
            variant: "body",
            style: {
              fontSize: "13px",
              color: UI_COLORS.text.secondary,
            },
          },
        },
        {
          type: "text",
          id: `${listId}-value-${index}`,
          props: {
            content: item.value,
            variant: "body",
            style: {
              fontSize: "13px",
              fontWeight: "600",
              fontFamily: "monospace",
              color: item.color,
            },
          },
        },
      ],
    }));

    requestAnimationFrame(() => {
      window.postMessage(
        {
          type: "update_dynamic_lists",
          lists: {
            [listId]: listItems,
          },
        },
        "*"
      );
    });
  }

  /**
   * Format label for display
   */
  private formatLabel(key: string): string {
    return key
      .replace(/_/g, " ")
      .replace(/\b\w/g, (l) => l.toUpperCase())
      .replace(/Ms$/, "")
      .replace(/Bytes$/, "");
  }

  /**
   * Format metric value based on type
   */
  private formatMetricValue(key: string, value: any): string {
    if (typeof value !== "number") {
      return String(value);
    }

    // Bytes
    if (key.includes("bytes") || key.includes("size")) {
      return formatBytes(value);
    }

    // Milliseconds
    if (key.includes("latency") || key.includes("duration") || key.includes("_ms")) {
      return `${value.toFixed(2)}ms`;
    }

    // Percentages
    if (key.includes("rate") || key.includes("percent")) {
      return `${(value * 100).toFixed(2)}%`;
    }

    // Large numbers
    if (value > 1000000) {
      return `${(value / 1000000).toFixed(2)}M`;
    }
    if (value > 1000) {
      return `${(value / 1000).toFixed(2)}K`;
    }

    return value.toFixed(2);
  }

  /**
   * Get color for metric value using centralized color system
   */
  private getMetricColor(key: string, value: any): string {
    if (typeof value !== "number") {
      return METRIC_COLORS.neutral;
    }

    // Error metrics (red if > 0)
    if (key.includes("error") || key.includes("denied") || key.includes("failed")) {
      return value > 0 ? METRIC_COLORS.error : METRIC_COLORS.success;
    }

    // Latency metrics (green < 100ms, yellow < 1s, red >= 1s)
    if (key.includes("latency") || key.includes("duration")) {
      if (value < 100) return METRIC_COLORS.success;
      if (value < 1000) return METRIC_COLORS.warning;
      return METRIC_COLORS.error;
    }

    // Success rate (green > 95%, yellow > 90%, red otherwise)
    if (key.includes("rate") && !key.includes("error")) {
      const percent = value * 100;
      if (percent > 95) return METRIC_COLORS.success;
      if (percent > 90) return METRIC_COLORS.warning;
      return METRIC_COLORS.error;
    }

    return METRIC_COLORS.neutral;
  }

  /**
   * Format uptime duration (seconds to ms for formatDuration utility)
   */
  private formatUptime(seconds: number): string {
    return formatDuration(seconds * 1000);
  }

  /**
   * Format large numbers using shared utility
   */
  private formatNumber(num: number): string {
    return formatCompact(num, 2);
  }

  /**
   * Cleanup on unmount
   */
  cleanup(): void {
    this.stopAutoRefresh();
    logger.info("Analysis executor cleaned up", { component: "AnalysisExecutor" });
  }
}
