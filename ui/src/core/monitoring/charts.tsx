/**
 * Metrics Dashboard Charts
 * Real-time visualization of system metrics
 */

import React, { useState, useEffect } from "react";
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import { getAllMetrics, getMetricsSummary, type SystemMetrics } from "./dashboard";
import { getRechartsTheme, CHART_COLORS } from "../../features/visualization/utils";

// ============================================================================
// Chart Components
// ============================================================================

interface MetricsChartProps {
  data: SystemMetrics[];
  width?: number | string;
  height?: number;
}

/**
 * System Performance Chart
 * Shows latency trends over time
 */
export const PerformanceChart: React.FC<MetricsChartProps> = ({ data, height = 300 }) => {
  const theme = getRechartsTheme("dark");

  const chartData = data.map((m) => ({
    timestamp: new Date(m.timestamp).toLocaleTimeString(),
    latency: m.ui?.histograms?.["tool_execution_duration"]?.avg * 1000 || 0,
    errors: m.ui?.counters?.["errors_total"] || 0,
  }));

  return (
    <div className="chart-container" style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={chartData}>
          <defs>
            <linearGradient id="latencyGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor={CHART_COLORS.primary[0]} stopOpacity={0.8} />
              <stop offset="95%" stopColor={CHART_COLORS.primary[0]} stopOpacity={0.1} />
            </linearGradient>
          </defs>
          <CartesianGrid {...theme.cartesianGrid} />
          <XAxis dataKey="timestamp" {...theme.axis} />
          <YAxis {...theme.axis} />
          <Tooltip {...theme.tooltip} />
          <Legend {...theme.legend} />
          <Area
            type="monotone"
            dataKey="latency"
            stroke={CHART_COLORS.primary[0]}
            fill="url(#latencyGradient)"
            name="Latency (ms)"
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
};

/**
 * Error Rate Chart
 * Shows error count over time
 */
export const ErrorRateChart: React.FC<MetricsChartProps> = ({ data, height = 300 }) => {
  const theme = getRechartsTheme("dark");

  const chartData = data.map((m) => ({
    timestamp: new Date(m.timestamp).toLocaleTimeString(),
    errors: m.ui?.counters?.["errors_total"] || 0,
  }));

  return (
    <div className="chart-container" style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={chartData}>
          <CartesianGrid {...theme.cartesianGrid} />
          <XAxis dataKey="timestamp" {...theme.axis} />
          <YAxis {...theme.axis} />
          <Tooltip {...theme.tooltip} />
          <Legend {...theme.legend} />
          <Line
            type="monotone"
            dataKey="errors"
            stroke={CHART_COLORS.semantic.error}
            strokeWidth={2}
            dot={{ r: 3 }}
            name="Errors"
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};

/**
 * Tool Execution Bar Chart
 * Shows tool execution counts
 */
export const ToolExecutionChart: React.FC<{ metrics: any; height?: number }> = ({
  metrics,
  height = 300,
}) => {
  const theme = getRechartsTheme("dark");

  // Extract tool execution counts
  const toolData = Object.entries(metrics?.counters || {})
    .filter(([key]) => key.includes("tool_executions_total"))
    .map(([key, value]) => ({
      tool: key.replace("tool_executions_total_", "").replace(/_/g, " "),
      count: value as number,
    }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 10); // Top 10 tools

  return (
    <div className="chart-container" style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={toolData} layout="horizontal">
          <CartesianGrid {...theme.cartesianGrid} />
          <XAxis type="number" {...theme.axis} />
          <YAxis dataKey="tool" type="category" width={120} {...theme.axis} />
          <Tooltip {...theme.tooltip} />
          <Bar dataKey="count" fill={CHART_COLORS.primary[0]} name="Executions" />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
};

/**
 * Web Vitals Chart
 * Shows Core Web Vitals metrics
 */
export const WebVitalsChart: React.FC<{ metrics: any; height?: number }> = ({
  metrics,
  height = 300,
}) => {
  const theme = getRechartsTheme("dark");

  const vitalsData = [
    { name: "LCP", value: metrics?.LCP?.value || 0, threshold: 2500, unit: "ms" },
    { name: "FID", value: metrics?.FID?.value || 0, threshold: 100, unit: "ms" },
    { name: "CLS", value: (metrics?.CLS?.value || 0) * 1000, threshold: 100, unit: "×1000" },
    { name: "FCP", value: metrics?.FCP?.value || 0, threshold: 1800, unit: "ms" },
    { name: "TTFB", value: metrics?.TTFB?.value || 0, threshold: 800, unit: "ms" },
  ];

  return (
    <div className="chart-container" style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={vitalsData}>
          <CartesianGrid {...theme.cartesianGrid} />
          <XAxis dataKey="name" {...theme.axis} />
          <YAxis {...theme.axis} />
          <Tooltip {...theme.tooltip} />
          <Legend {...theme.legend} />
          <Bar dataKey="value" fill={CHART_COLORS.primary[1]} name="Value" />
          <Bar dataKey="threshold" fill={CHART_COLORS.semantic.warning} name="Threshold" />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
};

// ============================================================================
// Live Dashboard Component
// ============================================================================

export const LiveMetricsDashboard: React.FC = () => {
  const [metricsHistory, setMetricsHistory] = useState<SystemMetrics[]>([]);
  const [currentMetrics, setCurrentMetrics] = useState<SystemMetrics | null>(null);

  useEffect(() => {
    const fetchMetrics = async () => {
      const metrics = await getAllMetrics();
      setCurrentMetrics(metrics);
      setMetricsHistory((prev) => {
        const updated = [...prev, metrics];
        // Keep only last 60 data points
        return updated.slice(-60);
      });
    };

    // Initial fetch
    fetchMetrics();

    // Poll every 5 seconds
    const interval = setInterval(fetchMetrics, 5000);

    return () => clearInterval(interval);
  }, []);

  if (!currentMetrics) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-gray-400">Loading metrics...</div>
      </div>
    );
  }

  const summary = getMetricsSummary();

  return (
    <div className="p-8 space-y-8 bg-gray-950 min-h-screen">
      <div className="space-y-2">
        <h1 className="text-4xl font-bold bg-gradient-to-r from-purple-400 to-blue-400 bg-clip-text text-transparent">
          AgentOS Metrics Dashboard
        </h1>
        <p className="text-gray-400">Real-time system performance monitoring</p>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
          <div className="text-sm text-gray-400">Total Executions</div>
          <div className="text-3xl font-bold text-white mt-2">{summary.totalToolExecutions}</div>
        </div>
        <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
          <div className="text-sm text-gray-400">Errors</div>
          <div className="text-3xl font-bold text-red-400 mt-2">{summary.totalErrors}</div>
        </div>
        <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
          <div className="text-sm text-gray-400">Avg Latency</div>
          <div className="text-3xl font-bold text-blue-400 mt-2">
            {summary.avgLatencyMs.toFixed(2)}ms
          </div>
        </div>
        <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
          <div className="text-sm text-gray-400">Uptime</div>
          <div className="text-3xl font-bold text-green-400 mt-2">
            {Math.floor(summary.uptime / 60)}m
          </div>
        </div>
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
          <h3 className="text-xl font-semibold text-white mb-4">Performance Trend</h3>
          <PerformanceChart data={metricsHistory} />
        </div>

        <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
          <h3 className="text-xl font-semibold text-white mb-4">Error Rate</h3>
          <ErrorRateChart data={metricsHistory} />
        </div>

        <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
          <h3 className="text-xl font-semibold text-white mb-4">Tool Executions</h3>
          <ToolExecutionChart metrics={currentMetrics.ui} />
        </div>

        <div className="bg-gray-900 border border-gray-800 rounded-lg p-6">
          <h3 className="text-xl font-semibold text-white mb-4">Web Vitals</h3>
          <WebVitalsChart metrics={currentMetrics.webVitals} />
        </div>
      </div>
    </div>
  );
};
