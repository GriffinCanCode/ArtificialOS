/**
 * Area Chart Component
 * Renders dynamic area charts with gradient fills
 */

import React from "react";
import {
  AreaChart as RechartsArea,
  Area,
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
import type { AreaChartProps } from "../types";

export const AreaChart: React.FC<BaseComponentProps> = ({ component, state }) => {
  const props = component.props as AreaChartProps;

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
  const stacked = props.stacked || false;

  return (
    <div className="chart-container" style={{ width: width || "100%", height }}>
      <ResponsiveContainer width="100%" height="100%">
        <RechartsArea
          data={data}
          margin={margin}
          {...(animate && { isAnimationActive: true })}
        >
          <defs>
            {series.map((s, index) => {
              const color = s.color || getSeriesColor(index);
              return (
                <linearGradient key={s.dataKey} id={`gradient-${s.dataKey}`} x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor={color} stopOpacity={0.8} />
                  <stop offset="95%" stopColor={color} stopOpacity={0.1} />
                </linearGradient>
              );
            })}
          </defs>

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

          {series.map((s, index) => {
            const color = s.color || getSeriesColor(index);
            return (
              <Area
                key={s.dataKey}
                type={props.smooth ? "monotone" : "linear"}
                dataKey={s.dataKey}
                name={s.name || s.dataKey}
                stroke={color}
                strokeWidth={2}
                fill={`url(#gradient-${s.dataKey})`}
                fillOpacity={s.fillOpacity || 1}
                stackId={stacked ? "stack" : undefined}
                yAxisId={s.yAxisId || "left"}
              />
            );
          })}
        </RechartsArea>
      </ResponsiveContainer>
    </div>
  );
};

AreaChart.displayName = "AreaChart";
