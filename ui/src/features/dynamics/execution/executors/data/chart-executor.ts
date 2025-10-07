/**
 * Chart Tool Executor
 * Handles chart data manipulation and updates
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";
import {
  normalize,
  cumulative,
  derivative,
  movingAverage,
  resample,
  fillGaps,
  applyTransform,
} from "../../../../visualization/utils/transforms";
import type { DataPoint, TransformOptions } from "../../../../visualization/types";

export class ChartExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    switch (action) {
      case "updateData":
        return this.updateData(params);

      case "addPoint":
        return this.addPoint(params);

      case "transform":
        return this.transformData(params);

      case "normalize":
        return this.normalizeData(params);

      case "cumulative":
        return this.cumulativeData(params);

      case "derivative":
        return this.derivativeData(params);

      case "smooth":
        return this.smoothData(params);

      case "resample":
        return this.resampleData(params);

      case "fillGaps":
        return this.fillGapsData(params);

      case "updateSeries":
        return this.updateSeries(params);

      default:
        logger.warn("Unknown chart action", { component: "ChartExecutor", action });
        return null;
    }
  }

  /**
   * Update entire chart data
   */
  private updateData(params: Record<string, any>): void {
    const { chartId, data } = params;
    if (!chartId || !data) return;

    this.context.componentState.set(`${chartId}.data`, data);
    logger.debug("Chart data updated", {
      component: "ChartExecutor",
      chartId,
      points: data.length,
    });
  }

  /**
   * Add single data point
   */
  private addPoint(params: Record<string, any>): void {
    const { chartId, point, maxPoints } = params;
    if (!chartId || !point) return;

    const currentData = this.context.componentState.get(`${chartId}.data`) || [];
    let newData = [...currentData, point];

    // Limit data points if maxPoints specified
    if (maxPoints && newData.length > maxPoints) {
      newData = newData.slice(newData.length - maxPoints);
    }

    this.context.componentState.set(`${chartId}.data`, newData);
    logger.debug("Chart point added", {
      component: "ChartExecutor",
      chartId,
      totalPoints: newData.length,
    });
  }

  /**
   * Apply data transformation
   */
  private transformData(params: Record<string, any>): DataPoint[] {
    const { data, key, options } = params;
    if (!data || !key) return data;

    const transformOptions: TransformOptions = options || { type: "none" };
    const transformed = applyTransform(data, key, transformOptions);

    logger.debug("Data transformed", {
      component: "ChartExecutor",
      transform: transformOptions.type,
      points: transformed.length,
    });

    return transformed;
  }

  /**
   * Normalize data
   */
  private normalizeData(params: Record<string, any>): DataPoint[] {
    const { data, key } = params;
    if (!data || !key) return data;

    const normalized = normalize(data, key);
    logger.debug("Data normalized", {
      component: "ChartExecutor",
      key,
      points: normalized.length,
    });

    return normalized;
  }

  /**
   * Calculate cumulative sum
   */
  private cumulativeData(params: Record<string, any>): DataPoint[] {
    const { data, key } = params;
    if (!data || !key) return data;

    const result = cumulative(data, key);
    logger.debug("Cumulative data calculated", {
      component: "ChartExecutor",
      key,
      points: result.length,
    });

    return result;
  }

  /**
   * Calculate derivative
   */
  private derivativeData(params: Record<string, any>): DataPoint[] {
    const { data, key } = params;
    if (!data || !key) return data;

    const result = derivative(data, key);
    logger.debug("Derivative calculated", {
      component: "ChartExecutor",
      key,
      points: result.length,
    });

    return result;
  }

  /**
   * Smooth data with moving average
   */
  private smoothData(params: Record<string, any>): DataPoint[] {
    const { data, key, window = 3 } = params;
    if (!data || !key) return data;

    const smoothed = movingAverage(data, key, window);
    logger.debug("Data smoothed", {
      component: "ChartExecutor",
      key,
      window,
      points: smoothed.length,
    });

    return smoothed;
  }

  /**
   * Resample time series data
   */
  private resampleData(params: Record<string, any>): DataPoint[] {
    const { data, intervalMs, aggregation = "mean" } = params;
    if (!data || !intervalMs) return data;

    const resampled = resample(data, intervalMs, aggregation);
    logger.debug("Data resampled", {
      component: "ChartExecutor",
      intervalMs,
      aggregation,
      points: resampled.length,
    });

    return resampled;
  }

  /**
   * Fill gaps in time series
   */
  private fillGapsData(params: Record<string, any>): DataPoint[] {
    const { data, intervalMs, method = "linear" } = params;
    if (!data || !intervalMs) return data;

    const filled = fillGaps(data, intervalMs, method);
    logger.debug("Gaps filled", {
      component: "ChartExecutor",
      intervalMs,
      method,
      points: filled.length,
    });

    return filled;
  }

  /**
   * Update chart series configuration
   */
  private updateSeries(params: Record<string, any>): void {
    const { chartId, series } = params;
    if (!chartId || !series) return;

    this.context.componentState.set(`${chartId}.series`, series);
    logger.debug("Chart series updated", {
      component: "ChartExecutor",
      chartId,
      seriesCount: series.length,
    });
  }
}
