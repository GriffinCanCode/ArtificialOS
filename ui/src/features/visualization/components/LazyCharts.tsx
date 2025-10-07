/**
 * Lazy-loaded Chart Components
 *
 * Heavy chart libraries (recharts, reactflow) are lazy loaded
 * to reduce initial bundle size. They're only needed when
 * the AI generates visualization components.
 */

import React, { lazy, Suspense } from "react";
import type { BaseComponentProps } from "../../dynamics/core/types";

// Lazy load chart components
const LineChartImpl = lazy(() =>
  import("./LineChart").then(module => ({ default: module.LineChart }))
);

const BarChartImpl = lazy(() =>
  import("./BarChart").then(module => ({ default: module.BarChart }))
);

const AreaChartImpl = lazy(() =>
  import("./AreaChart").then(module => ({ default: module.AreaChart }))
);

const PieChartImpl = lazy(() =>
  import("./PieChart").then(module => ({ default: module.PieChart }))
);

const GraphImpl = lazy(() =>
  import("./Graph").then(module => ({ default: module.Graph }))
);

// Loading fallback
const ChartLoader: React.FC = () => (
  <div className="chart-loading" style={{
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '200px',
    color: '#888'
  }}>
    Loading chart...
  </div>
);

// Wrapped components with Suspense
export const LineChart: React.FC<BaseComponentProps> = (props) => (
  <Suspense fallback={<ChartLoader />}>
    <LineChartImpl {...props} />
  </Suspense>
);

export const BarChart: React.FC<BaseComponentProps> = (props) => (
  <Suspense fallback={<ChartLoader />}>
    <BarChartImpl {...props} />
  </Suspense>
);

export const AreaChart: React.FC<BaseComponentProps> = (props) => (
  <Suspense fallback={<ChartLoader />}>
    <AreaChartImpl {...props} />
  </Suspense>
);

export const PieChart: React.FC<BaseComponentProps> = (props) => (
  <Suspense fallback={<ChartLoader />}>
    <PieChartImpl {...props} />
  </Suspense>
);

export const Graph: React.FC<BaseComponentProps> = (props) => (
  <Suspense fallback={<ChartLoader />}>
    <GraphImpl {...props} />
  </Suspense>
);

// Display names
LineChart.displayName = "LineChart";
BarChart.displayName = "BarChart";
AreaChart.displayName = "AreaChart";
PieChart.displayName = "PieChart";
Graph.displayName = "Graph";
