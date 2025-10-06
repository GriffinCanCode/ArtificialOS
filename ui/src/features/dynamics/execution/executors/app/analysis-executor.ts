/**
 * System Analysis Tool Executor
 * Handles system metrics collection and display
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { getAllMetrics } from "../../../../../core/monitoring";
import { ExecutorContext, AsyncExecutor } from "../core/types";

export class AnalysisExecutor implements AsyncExecutor {
  private context: ExecutorContext;
  private refreshInterval: number | null = null;

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

      // Update summary cards
      this.updateSummaryCards(allMetrics);

      // Update detailed metrics lists
      this.updateBackendMetrics(allMetrics.backend);
      this.updateKernelMetrics(allMetrics.kernel);
      this.updateAIMetrics(allMetrics.aiService);

      // Update last refresh time
      this.context.componentState.set(
        "last-update",
        `Updated: ${new Date().toLocaleTimeString()}`
      );

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
    const summary = metrics.backend?.summary || {};

    // Uptime
    const uptimeSeconds = summary.uptime_seconds || metrics.ui.uptime_seconds || 0;
    this.context.componentState.set("uptime-value", this.formatUptime(uptimeSeconds));

    // Total requests
    const totalRequests = summary.total_requests || 0;
    this.context.componentState.set("requests-value", this.formatNumber(totalRequests));

    // Average latency
    const avgLatency = summary.average_latency_ms || 0;
    this.context.componentState.set("latency-value", `${avgLatency.toFixed(1)}ms`);

    // Active apps/connections
    const activeApps = summary.active_connections || 0;
    this.context.componentState.set("apps-value", activeApps.toString());
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
          borderBottom: "1px solid rgba(255,255,255,0.06)",
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
              color: "rgba(255,255,255,0.7)",
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
      return this.formatBytes(value);
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
   * Get color for metric value
   */
  private getMetricColor(key: string, value: any): string {
    if (typeof value !== "number") {
      return "#fff";
    }

    // Error metrics (red if > 0)
    if (key.includes("error") || key.includes("denied") || key.includes("failed")) {
      return value > 0 ? "#f87171" : "#4ade80";
    }

    // Latency metrics (green < 100ms, yellow < 1s, red >= 1s)
    if (key.includes("latency") || key.includes("duration")) {
      if (value < 100) return "#4ade80";
      if (value < 1000) return "#fbbf24";
      return "#f87171";
    }

    // Success rate (green > 95%, yellow > 90%, red otherwise)
    if (key.includes("rate") && !key.includes("error")) {
      const percent = value * 100;
      if (percent > 95) return "#4ade80";
      if (percent > 90) return "#fbbf24";
      return "#f87171";
    }

    return "#fff";
  }

  /**
   * Format uptime duration
   */
  private formatUptime(seconds: number): string {
    if (seconds < 60) return `${Math.floor(seconds)}s`;

    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m`;

    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ${minutes % 60}m`;

    const days = Math.floor(hours / 24);
    return `${days}d ${hours % 24}h`;
  }

  /**
   * Format byte values
   */
  private formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)}KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)}MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)}GB`;
  }

  /**
   * Format large numbers
   */
  private formatNumber(num: number): string {
    if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(2)}K`;
    return num.toString();
  }

  /**
   * Cleanup on unmount
   */
  cleanup(): void {
    this.stopAutoRefresh();
    logger.info("Analysis executor cleaned up", { component: "AnalysisExecutor" });
  }
}
