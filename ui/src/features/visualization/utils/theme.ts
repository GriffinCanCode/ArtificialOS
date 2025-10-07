/**
 * Theme Configuration
 * Theme-aware styling for charts and graphs
 */

import { CHART_COLORS, withOpacity } from "./colors";

// ============================================================================
// Theme Types
// ============================================================================

export interface ChartTheme {
  text: string;
  grid: string;
  axis: string;
  background: string;
  tooltip: {
    background: string;
    border: string;
    text: string;
  };
  legend: {
    text: string;
  };
}

// ============================================================================
// Theme Definitions
// ============================================================================

export const LIGHT_THEME: ChartTheme = {
  text: "#1a202c",
  grid: "#e2e8f0",
  axis: "#4a5568",
  background: "#ffffff",
  tooltip: {
    background: "#ffffff",
    border: "#e2e8f0",
    text: "#1a202c",
  },
  legend: {
    text: "#1a202c",
  },
};

export const DARK_THEME: ChartTheme = {
  text: "#f7fafc",
  grid: withOpacity("#ffffff", 0.1),
  axis: "#a0aec0",
  background: "#1a202c",
  tooltip: {
    background: "#2d3748",
    border: "#4a5568",
    text: "#f7fafc",
  },
  legend: {
    text: "#f7fafc",
  },
};

// ============================================================================
// Theme Functions
// ============================================================================

/**
 * Get theme configuration
 */
export function getTheme(theme: "light" | "dark" = "dark"): ChartTheme {
  return theme === "light" ? LIGHT_THEME : DARK_THEME;
}

/**
 * Get Recharts theme props
 */
export function getRechartsTheme(theme: "light" | "dark" = "dark") {
  const t = getTheme(theme);

  return {
    text: { fill: t.text },
    grid: { stroke: t.grid },
    axis: {
      stroke: t.axis,
      tick: { fill: t.axis },
      axisLine: { stroke: t.axis },
    },
    cartesianGrid: {
      stroke: t.grid,
      strokeDasharray: "3 3",
    },
    tooltip: {
      contentStyle: {
        backgroundColor: t.tooltip.background,
        border: `1px solid ${t.tooltip.border}`,
        borderRadius: "8px",
        color: t.tooltip.text,
      },
      cursor: { fill: withOpacity(CHART_COLORS.primary[0], 0.1) },
    },
    legend: {
      iconType: "circle" as const,
      wrapperStyle: { color: t.legend.text },
    },
  };
}

/**
 * Get ReactFlow theme props
 */
export function getReactFlowTheme(theme: "light" | "dark" = "dark") {
  const t = getTheme(theme);

  return {
    background: t.background,
    node: {
      background: theme === "dark" ? "#2d3748" : "#ffffff",
      border: theme === "dark" ? "#4a5568" : "#e2e8f0",
      text: t.text,
    },
    edge: {
      stroke: theme === "dark" ? "#4a5568" : "#cbd5e0",
    },
    minimap: {
      background: theme === "dark" ? "#1a202c" : "#f7fafc",
      maskColor: withOpacity(t.background, 0.8),
    },
  };
}

/**
 * Get CSS custom properties for theming
 */
export function getThemeVariables(theme: "light" | "dark" = "dark"): Record<string, string> {
  const t = getTheme(theme);

  return {
    "--chart-text": t.text,
    "--chart-grid": t.grid,
    "--chart-axis": t.axis,
    "--chart-background": t.background,
    "--chart-tooltip-bg": t.tooltip.background,
    "--chart-tooltip-border": t.tooltip.border,
    "--chart-tooltip-text": t.tooltip.text,
  };
}
