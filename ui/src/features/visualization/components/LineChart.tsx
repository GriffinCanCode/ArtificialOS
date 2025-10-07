/**
 * Line Chart Component
 * Renders dynamic line charts with time series support
 */

import React from "react";
import {
  LineChart as RechartsLine,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import type { BaseComponentProps } from "../../dynamics/core/types";
import { useSyncState } from "../../dynamics/hooks/useSyncState";
import { getRechartsTheme, getSeriesColor } from "../utils";
import type { LineChartProps } from "../types";

export const LineChart: React.FC<BaseComponentProps> = ({ component, state }) => {
  const props = component.props as LineChartProps;

  // Subscribe to data changes
  const data = useSyncState(state, `${component.id}.data`, props.data || []);
  const series = props.series || [];
  const theme = props.theme || "dark";
  const themeProps = getRechartsTheme(theme);

  // Dimensions
  const width = props.dimensions?.width;
  const height = props.dimensions?.height || 400;

  // Margin
  const margin = props.margin || { top: 20, right: 30, left: 20, bottom: 20 };

  // Axis configuration
  const xAxis = props.xAxis || { dataKey: "name" };
  const yAxis = props.yAxis || {};

  // Features
  const showGrid = props.grid?.show !== false;
  const showLegend = props.legend?.show !== false;
  const showTooltip = props.tooltip?.show !== false;
  const animate = props.animate !== false;

  return (
    <div className="chart-container" style={{ width: width || "100%", height }}>
      <ResponsiveContainer width="100%" height="100%">
        <RechartsLine
          data={data}
          margin={margin}
          {...(animate && { isAnimationActive: true })}
        >
          {showGrid && <CartesianGrid {...themeProps.cartesianGrid} />}

          <XAxis
            dataKey={xAxis.dataKey}
            label={xAxis.label ? { value: xAxis.label, position: "insideBottom", offset: -10 } : undefined}
            hide={xAxis.hide}
            {...themeProps.axis}
          />

          <YAxis
            label={yAxis.label ? { value: yAxis.label, angle: -90, position: "insideLeft" } : undefined}
            domain={yAxis.domain}
            hide={yAxis.hide}
            {...themeProps.axis}
          />

          {showTooltip && (
            <Tooltip {...themeProps.tooltip} />
          )}

          {showLegend && (
            <Legend
              verticalAlign={props.legend?.position === "bottom" ? "bottom" : "top"}
              align={props.legend?.align || "right"}
              {...themeProps.legend}
            />
          )}

          {series.map((s, index) => (
            <Line
              key={s.dataKey}
              type={props.smooth ? "monotone" : "linear"}
              dataKey={s.dataKey}
              name={s.name || s.dataKey}
              stroke={s.color || getSeriesColor(index)}
              strokeWidth={s.strokeWidth || 2}
              dot={{ r: 3 }}
              activeDot={{ r: 5 }}
              connectNulls={props.connectNulls}
              yAxisId={s.yAxisId || "left"}
            />
          ))}
        </RechartsLine>
      </ResponsiveContainer>
    </div>
  );
};

LineChart.displayName = "LineChart";
