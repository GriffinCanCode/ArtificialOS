/**
 * Data Transform Utilities
 * Transformations and processing for visualization data
 */

import type { DataPoint, TimeSeriesPoint, TransformOptions, MetricData } from "../types";

// ============================================================================
// Basic Transforms
// ============================================================================

/**
 * Normalize data to 0-1 range
 */
export function normalize(data: DataPoint[], key: string): DataPoint[] {
  const values = data.map((d) => Number(d[key]) || 0);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;

  return data.map((d) => ({
    ...d,
    [key]: ((Number(d[key]) || 0) - min) / range,
  }));
}

/**
 * Calculate cumulative sum
 */
export function cumulative(data: DataPoint[], key: string): DataPoint[] {
  let sum = 0;
  return data.map((d) => {
    sum += Number(d[key]) || 0;
    return { ...d, [key]: sum };
  });
}

/**
 * Calculate derivative (rate of change)
 */
export function derivative(data: DataPoint[], key: string): DataPoint[] {
  if (data.length < 2) return data;

  return data.map((d, i) => {
    if (i === 0) return { ...d, [key]: 0 };
    const curr = Number(d[key]) || 0;
    const prev = Number(data[i - 1][key]) || 0;
    return { ...d, [key]: curr - prev };
  });
}

/**
 * Moving average smoothing
 */
export function movingAverage(data: DataPoint[], key: string, window = 3): DataPoint[] {
  return data.map((d, i) => {
    const start = Math.max(0, i - Math.floor(window / 2));
    const end = Math.min(data.length, i + Math.ceil(window / 2));
    const slice = data.slice(start, end);
    const avg = slice.reduce((sum, item) => sum + (Number(item[key]) || 0), 0) / slice.length;
    return { ...d, [key]: avg };
  });
}

// ============================================================================
// Time Series Operations
// ============================================================================

/**
 * Resample time series to fixed intervals
 */
export function resample(
  data: TimeSeriesPoint[],
  intervalMs: number,
  aggregation: "mean" | "sum" | "min" | "max" = "mean"
): TimeSeriesPoint[] {
  if (data.length === 0) return [];

  const sorted = [...data].sort((a, b) => a.timestamp - b.timestamp);
  const start = sorted[0].timestamp;
  const end = sorted[sorted.length - 1].timestamp;
  const buckets = Math.ceil((end - start) / intervalMs) + 1;

  const result: TimeSeriesPoint[] = [];
  const keys = Object.keys(sorted[0]).filter((k) => k !== "timestamp");

  for (let i = 0; i < buckets; i++) {
    const bucketStart = start + i * intervalMs;
    const bucketEnd = bucketStart + intervalMs;
    const bucketData = sorted.filter((d) => d.timestamp >= bucketStart && d.timestamp < bucketEnd);

    if (bucketData.length === 0) continue;

    const aggregated: TimeSeriesPoint = { timestamp: bucketStart };

    for (const key of keys) {
      const values = bucketData.map((d) => Number(d[key]) || 0);
      switch (aggregation) {
        case "mean":
          aggregated[key] = values.reduce((a, b) => a + b, 0) / values.length;
          break;
        case "sum":
          aggregated[key] = values.reduce((a, b) => a + b, 0);
          break;
        case "min":
          aggregated[key] = Math.min(...values);
          break;
        case "max":
          aggregated[key] = Math.max(...values);
          break;
      }
    }

    result.push(aggregated);
  }

  return result;
}

/**
 * Fill gaps in time series
 */
export function fillGaps(
  data: TimeSeriesPoint[],
  intervalMs: number,
  method: "linear" | "forward" | "zero" = "linear"
): TimeSeriesPoint[] {
  if (data.length < 2) return data;

  const sorted = [...data].sort((a, b) => a.timestamp - b.timestamp);
  const result: TimeSeriesPoint[] = [sorted[0]];
  const keys = Object.keys(sorted[0]).filter((k) => k !== "timestamp");

  for (let i = 1; i < sorted.length; i++) {
    const prev = sorted[i - 1];
    const curr = sorted[i];
    const gap = curr.timestamp - prev.timestamp;

    if (gap > intervalMs * 1.5) {
      const steps = Math.floor(gap / intervalMs);
      for (let j = 1; j < steps; j++) {
        const ratio = j / steps;
        const filled: TimeSeriesPoint = {
          timestamp: prev.timestamp + j * intervalMs,
        };

        for (const key of keys) {
          const prevVal = Number(prev[key]) || 0;
          const currVal = Number(curr[key]) || 0;

          switch (method) {
            case "linear":
              filled[key] = prevVal + (currVal - prevVal) * ratio;
              break;
            case "forward":
              filled[key] = prevVal;
              break;
            case "zero":
              filled[key] = 0;
              break;
          }
        }

        result.push(filled);
      }
    }

    result.push(curr);
  }

  return result;
}

// ============================================================================
// Statistical Operations
// ============================================================================

/**
 * Calculate percentiles
 */
/**
 * @deprecated Use statisticsSummary from core/utils/math instead
 */
export { percentile, statisticsSummary as statistics } from "../../../core/utils/math";

// ============================================================================
// Format Converters
// ============================================================================

/**
 * Convert metrics to chart data
 */
export function metricsToChartData(metrics: MetricData[]): DataPoint[] {
  return metrics.map((m) => ({
    name: m.name,
    value: m.value,
    timestamp: m.timestamp || Date.now(),
  }));
}

/**
 * Pivot data for multi-series charts
 */
export function pivotData(
  data: DataPoint[],
  indexKey: string,
  columnKey: string,
  valueKey: string
): DataPoint[] {
  const pivoted = new Map<string | number, DataPoint>();

  for (const row of data) {
    const index = row[indexKey];
    const column = row[columnKey];
    const value = row[valueKey];

    if (!pivoted.has(index)) {
      pivoted.set(index, { [indexKey]: index });
    }

    const point = pivoted.get(index)!;
    point[String(column)] = value;
  }

  return Array.from(pivoted.values());
}

/**
 * Apply transform to data
 */
export function applyTransform(
  data: DataPoint[],
  key: string,
  options: TransformOptions
): DataPoint[] {
  let transformed = data;

  switch (options.type) {
    case "cumulative":
      transformed = cumulative(data, key);
      break;
    case "derivative":
      transformed = derivative(data, key);
      break;
    case "normalize":
      transformed = normalize(data, key);
      break;
    case "none":
    default:
      break;
  }

  if (options.smoothing && options.window) {
    transformed = movingAverage(transformed, key, options.window);
  }

  return transformed;
}
