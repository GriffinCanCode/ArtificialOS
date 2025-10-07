/**
 * Pie Chart Component
 * Renders dynamic pie and donut charts
 */

import React from "react";
import {
  PieChart as RechartsPie,
  Pie,
  Cell,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import type { BaseComponentProps } from "../../dynamics/core/types";
import { useSyncState } from "../../dynamics/hooks/useSyncState";
import { getRechartsTheme, getSeriesColor } from "../utils";
import type { PieChartProps } from "../types";

export const PieChart: React.FC<BaseComponentProps> = ({ component, state }) => {
  const props = component.props as PieChartProps;

  // Subscribe to data changes
  const data = useSyncState(state, `${component.id}.data`, props.data || []);
  const theme = props.theme || "dark";
  const themeProps = getRechartsTheme(theme);

  // Dimensions
  const width = props.dimensions?.width;
  const height = props.dimensions?.height || 400;

  // Features
  const showLegend = props.legend?.show !== false;
  const showTooltip = props.tooltip?.show !== false;
  const animate = props.animate !== false;

  // Pie configuration
  const dataKey = props.dataKey || "value";
  const nameKey = props.nameKey || "name";
  const innerRadius = props.innerRadius || 0;
  const outerRadius = props.outerRadius || 80;
  const colors = props.colors || [];

  return (
    <div className="chart-container" style={{ width: width || "100%", height }}>
      <ResponsiveContainer width="100%" height="100%">
        <RechartsPie>
          {showTooltip && (
            <Tooltip {...themeProps.tooltip} />
          )}

          {showLegend && (
            <Legend
              verticalAlign={props.legend?.position === "top" ? "top" : "bottom"}
              align={props.legend?.align || "center"}
              {...themeProps.legend}
            />
          )}

          <Pie
            data={data}
            dataKey={dataKey}
            nameKey={nameKey}
            cx="50%"
            cy="50%"
            innerRadius={innerRadius}
            outerRadius={outerRadius}
            label
            {...(animate && { isAnimationActive: true })}
          >
            {data.map((entry, index) => (
              <Cell
                key={`cell-${index}`}
                fill={colors[index] || getSeriesColor(index)}
              />
            ))}
          </Pie>
        </RechartsPie>
      </ResponsiveContainer>
    </div>
  );
};

PieChart.displayName = "PieChart";
