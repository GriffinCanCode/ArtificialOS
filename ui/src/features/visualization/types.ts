/**
 * Visualization Type Definitions
 * Strongly typed interfaces for charts and graphs
 */

// ============================================================================
// Chart Types
// ============================================================================

export type ChartType = "line" | "bar" | "area" | "pie" | "composed";

export interface DataPoint {
  [key: string]: string | number | boolean | null;
}

export interface ChartSeries {
  dataKey: string;
  name?: string;
  color?: string;
  type?: "line" | "bar" | "area";
  yAxisId?: "left" | "right";
  strokeWidth?: number;
  fillOpacity?: number;
}

export interface ChartAxis {
  dataKey?: string;
  label?: string;
  domain?: [number | string, number | string];
  tickFormatter?: string;
  hide?: boolean;
}

export interface ChartLegend {
  show?: boolean;
  position?: "top" | "bottom" | "left" | "right";
  align?: "left" | "center" | "right";
}

export interface ChartTooltip {
  show?: boolean;
  formatter?: string;
  labelFormatter?: string;
}

export interface ChartGrid {
  show?: boolean;
  stroke?: string;
  strokeDasharray?: string;
}

export interface ChartMargin {
  top?: number;
  right?: number;
  bottom?: number;
  left?: number;
}

export interface ChartDimensions {
  width?: number | string;
  height?: number | string;
  aspectRatio?: number;
}

// ============================================================================
// Graph Types (Network Visualization)
// ============================================================================

export interface GraphNode {
  id: string;
  label?: string;
  type?: string;
  data?: Record<string, any>;
  position?: { x: number; y: number };
  style?: Record<string, any>;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  type?: "default" | "step" | "smoothstep" | "straight";
  animated?: boolean;
  style?: Record<string, any>;
}

export interface GraphLayout {
  type?: "dagre" | "force" | "grid" | "circular";
  direction?: "TB" | "BT" | "LR" | "RL";
  spacing?: number;
}

export interface GraphControls {
  zoom?: boolean;
  pan?: boolean;
  fitView?: boolean;
  minimap?: boolean;
  background?: boolean;
}

// ============================================================================
// Component Props
// ============================================================================

export interface BaseChartProps {
  data: DataPoint[];
  series: ChartSeries[];
  xAxis?: ChartAxis;
  yAxis?: ChartAxis;
  legend?: ChartLegend;
  tooltip?: ChartTooltip;
  grid?: ChartGrid;
  margin?: ChartMargin;
  dimensions?: ChartDimensions;
  theme?: "light" | "dark";
  animate?: boolean;
}

export interface LineChartProps extends BaseChartProps {
  smooth?: boolean;
  connectNulls?: boolean;
}

export interface BarChartProps extends BaseChartProps {
  stacked?: boolean;
  barSize?: number;
}

export interface AreaChartProps extends BaseChartProps {
  stacked?: boolean;
  smooth?: boolean;
}

export interface PieChartProps {
  data: DataPoint[];
  dataKey: string;
  nameKey: string;
  colors?: string[];
  innerRadius?: number;
  outerRadius?: number;
  legend?: ChartLegend;
  tooltip?: ChartTooltip;
  dimensions?: ChartDimensions;
  theme?: "light" | "dark";
  animate?: boolean;
}

export interface GraphProps {
  nodes: GraphNode[];
  edges: GraphEdge[];
  layout?: GraphLayout;
  controls?: GraphControls;
  dimensions?: ChartDimensions;
  theme?: "light" | "dark";
  interactive?: boolean;
}

// ============================================================================
// Utility Types
// ============================================================================

export interface TimeSeriesPoint {
  timestamp: number;
  [key: string]: number | string;
}

export interface MetricData {
  name: string;
  value: number;
  timestamp?: number;
  unit?: string;
}

export interface HistogramData {
  bucket: string;
  count: number;
  percentage?: number;
}

export type DataTransform = "none" | "cumulative" | "derivative" | "normalize";

export interface TransformOptions {
  type: DataTransform;
  window?: number;
  smoothing?: boolean;
}
