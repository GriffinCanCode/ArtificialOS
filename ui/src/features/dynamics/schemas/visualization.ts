/**
 * Visualization Component Schemas
 * Zod schemas for chart and graph components
 */

import { z } from "zod";

// ============================================================================
// Shared Schemas
// ============================================================================

const chartSeriesSchema = z.object({
  dataKey: z.string(),
  name: z.string().optional(),
  color: z.string().optional(),
  type: z.enum(["line", "bar", "area"]).optional(),
  yAxisId: z.enum(["left", "right"]).optional(),
  strokeWidth: z.number().optional(),
  fillOpacity: z.number().optional(),
});

const chartAxisSchema = z.object({
  dataKey: z.string().optional(),
  label: z.string().optional(),
  domain: z.tuple([z.union([z.number(), z.string()]), z.union([z.number(), z.string()])]).optional(),
  tickFormatter: z.string().optional(),
  hide: z.boolean().optional(),
});

const chartLegendSchema = z.object({
  show: z.boolean().optional(),
  position: z.enum(["top", "bottom", "left", "right"]).optional(),
  align: z.enum(["left", "center", "right"]).optional(),
});

const chartTooltipSchema = z.object({
  show: z.boolean().optional(),
  formatter: z.string().optional(),
  labelFormatter: z.string().optional(),
});

const chartGridSchema = z.object({
  show: z.boolean().optional(),
  stroke: z.string().optional(),
  strokeDasharray: z.string().optional(),
});

const chartMarginSchema = z.object({
  top: z.number().optional(),
  right: z.number().optional(),
  bottom: z.number().optional(),
  left: z.number().optional(),
});

const chartDimensionsSchema = z.object({
  width: z.union([z.number(), z.string()]).optional(),
  height: z.union([z.number(), z.string()]).optional(),
  aspectRatio: z.number().optional(),
});

const baseChartPropsSchema = z.object({
  data: z.array(z.record(z.union([z.string(), z.number(), z.boolean(), z.null()]))),
  series: z.array(chartSeriesSchema),
  xAxis: chartAxisSchema.optional(),
  yAxis: chartAxisSchema.optional(),
  legend: chartLegendSchema.optional(),
  tooltip: chartTooltipSchema.optional(),
  grid: chartGridSchema.optional(),
  margin: chartMarginSchema.optional(),
  dimensions: chartDimensionsSchema.optional(),
  theme: z.enum(["light", "dark"]).optional(),
  animate: z.boolean().optional(),
});

// ============================================================================
// Chart Schemas
// ============================================================================

export const lineChartSchema = baseChartPropsSchema.extend({
  smooth: z.boolean().optional(),
  connectNulls: z.boolean().optional(),
});

export const barChartSchema = baseChartPropsSchema.extend({
  stacked: z.boolean().optional(),
  barSize: z.number().optional(),
});

export const areaChartSchema = baseChartPropsSchema.extend({
  stacked: z.boolean().optional(),
  smooth: z.boolean().optional(),
});

export const pieChartSchema = z.object({
  data: z.array(z.record(z.union([z.string(), z.number(), z.boolean(), z.null()]))),
  dataKey: z.string(),
  nameKey: z.string(),
  colors: z.array(z.string()).optional(),
  innerRadius: z.number().optional(),
  outerRadius: z.number().optional(),
  legend: chartLegendSchema.optional(),
  tooltip: chartTooltipSchema.optional(),
  dimensions: chartDimensionsSchema.optional(),
  theme: z.enum(["light", "dark"]).optional(),
  animate: z.boolean().optional(),
});

// ============================================================================
// Graph Schema
// ============================================================================

const graphNodeSchema = z.object({
  id: z.string(),
  label: z.string().optional(),
  type: z.string().optional(),
  data: z.record(z.any()).optional(),
  position: z.object({ x: z.number(), y: z.number() }).optional(),
  style: z.record(z.any()).optional(),
});

const graphEdgeSchema = z.object({
  id: z.string(),
  source: z.string(),
  target: z.string(),
  label: z.string().optional(),
  type: z.enum(["default", "step", "smoothstep", "straight"]).optional(),
  animated: z.boolean().optional(),
  style: z.record(z.any()).optional(),
});

const graphLayoutSchema = z.object({
  type: z.enum(["dagre", "force", "grid", "circular"]).optional(),
  direction: z.enum(["TB", "BT", "LR", "RL"]).optional(),
  spacing: z.number().optional(),
});

const graphControlsSchema = z.object({
  zoom: z.boolean().optional(),
  pan: z.boolean().optional(),
  fitView: z.boolean().optional(),
  minimap: z.boolean().optional(),
  background: z.boolean().optional(),
});

export const graphSchema = z.object({
  nodes: z.array(graphNodeSchema),
  edges: z.array(graphEdgeSchema),
  layout: graphLayoutSchema.optional(),
  controls: graphControlsSchema.optional(),
  dimensions: chartDimensionsSchema.optional(),
  theme: z.enum(["light", "dark"]).optional(),
  interactive: z.boolean().optional(),
});
