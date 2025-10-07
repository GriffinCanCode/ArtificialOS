/**
 * Frontend Math Utilities
 *
 * Focused on UI-specific math operations:
 * - Safe expression evaluation (replaces eval)
 * - Number formatting for display
 * - UI calculations (clamping, interpolation, etc.)
 * - Chart/visualization statistics
 *
 * Note: For comprehensive math operations (trig, advanced stats, etc.),
 * use the backend math provider via tool execution.
 */

import { evaluate } from "mathjs";

// ============================================================================
// Safe Expression Evaluation
// ============================================================================

/**
 * Safely evaluate a mathematical expression
 * Replaces dangerous eval() with mathjs
 *
 * Supports:
 * - Basic arithmetic: +, -, *, /
 * - Advanced functions: sqrt, sin, cos, tan, log, exp, etc.
 * - Constants: pi, e
 * - Parentheses and order of operations
 *
 * @example
 * evaluateExpression("2 + 2") // 4
 * evaluateExpression("sqrt(16)") // 4
 * evaluateExpression("sin(pi/2)") // 1
 */
export function evaluateExpression(expression: string): number | string {
  try {
    // Sanitize special unicode operators to ASCII
    const sanitized = String(expression)
      .replace(/×/g, "*")
      .replace(/÷/g, "/")
      .replace(/−/g, "-")
      .trim();

    if (!sanitized) {
      return "0";
    }

    const result = evaluate(sanitized);

    // Handle special cases
    if (typeof result === "number") {
      if (!isFinite(result)) {
        return "Error";
      }
      // Round to avoid floating point precision issues
      return roundToSignificant(result, 12);
    }

    return String(result);
  } catch (error) {
    return "Error";
  }
}

/**
 * Validate if a string is a safe mathematical expression
 */
export function isValidExpression(expression: string): boolean {
  try {
    const sanitized = String(expression).replace(/×/g, "*").replace(/÷/g, "/").replace(/−/g, "-");

    evaluate(sanitized);
    return true;
  } catch {
    return false;
  }
}

// ============================================================================
// Number Formatting for UI Display
// ============================================================================

/**
 * Format number with specified decimal places
 */
export function formatNumber(value: number, decimals: number = 2): string {
  return value.toFixed(decimals);
}

/**
 * Format number with thousand separators
 */
export function formatWithThousands(value: number, separator: string = ","): string {
  const parts = value.toString().split(".");
  parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, separator);
  return parts.join(".");
}

/**
 * Format as percentage
 */
export function formatPercentage(value: number, decimals: number = 1): string {
  return `${(value * 100).toFixed(decimals)}%`;
}

/**
 * Format bytes to human readable size
 */
export function formatBytes(bytes: number, decimals: number = 2): string {
  if (bytes === 0) return "0 Bytes";

  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`;
}

/**
 * Format duration in milliseconds to human readable
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${Math.round(ms)}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;

  const minutes = Math.floor(ms / 60000);
  if (minutes < 60) return `${minutes}m`;

  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ${minutes % 60}m`;

  const days = Math.floor(hours / 24);
  return `${days}d ${hours % 24}h`;
}

/**
 * Format large numbers with abbreviations (K, M, B)
 */
export function formatCompact(value: number, decimals: number = 1): string {
  if (Math.abs(value) < 1000) return value.toFixed(decimals);
  if (Math.abs(value) < 1000000) return `${(value / 1000).toFixed(decimals)}K`;
  if (Math.abs(value) < 1000000000) return `${(value / 1000000).toFixed(decimals)}M`;
  return `${(value / 1000000000).toFixed(decimals)}B`;
}

// ============================================================================
// UI-Specific Math Operations
// ============================================================================

/**
 * Clamp a value between min and max
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

/**
 * Linear interpolation between two values
 */
export function lerp(start: number, end: number, t: number): number {
  return start + (end - start) * t;
}

/**
 * Normalize a value to 0-1 range
 */
export function normalize(value: number, min: number, max: number): number {
  if (max === min) return 0;
  return clamp((value - min) / (max - min), 0, 1);
}

/**
 * Map a value from one range to another
 */
export function mapRange(
  value: number,
  inMin: number,
  inMax: number,
  outMin: number,
  outMax: number
): number {
  const normalized = normalize(value, inMin, inMax);
  return lerp(outMin, outMax, normalized);
}

/**
 * Round to nearest multiple
 */
export function roundToNearest(value: number, multiple: number): number {
  return Math.round(value / multiple) * multiple;
}

/**
 * Round to significant figures
 */
export function roundToSignificant(value: number, figures: number): number {
  if (value === 0) return 0;
  const magnitude = Math.floor(Math.log10(Math.abs(value)));
  const scale = Math.pow(10, figures - magnitude - 1);
  return Math.round(value * scale) / scale;
}

// ============================================================================
// Statistics for Charts and Visualization
// ============================================================================

/**
 * Calculate mean (average)
 */
export function mean(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((sum, val) => sum + val, 0) / values.length;
}

/**
 * Calculate median
 */
export function median(values: number[]): number {
  if (values.length === 0) return 0;

  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);

  return sorted.length % 2 === 0 ? (sorted[mid - 1] + sorted[mid]) / 2 : sorted[mid];
}

/**
 * Calculate standard deviation
 */
export function stdDev(values: number[]): number {
  if (values.length === 0) return 0;

  const avg = mean(values);
  const squareDiffs = values.map((value) => Math.pow(value - avg, 2));
  const variance = mean(squareDiffs);

  return Math.sqrt(variance);
}

/**
 * Calculate percentile
 */
export function percentile(values: number[], p: number): number {
  if (values.length === 0) return 0;

  const sorted = [...values].sort((a, b) => a - b);
  const index = (p / 100) * (sorted.length - 1);
  const lower = Math.floor(index);
  const upper = Math.ceil(index);
  const weight = index - lower;

  return sorted[lower] * (1 - weight) + sorted[upper] * weight;
}

/**
 * Get min and max values
 */
export function minMax(values: number[]): { min: number; max: number } {
  if (values.length === 0) return { min: 0, max: 0 };
  return {
    min: Math.min(...values),
    max: Math.max(...values),
  };
}

/**
 * Calculate statistics summary for visualization
 */
export function statisticsSummary(values: number[]): {
  min: number;
  max: number;
  mean: number;
  median: number;
  stdDev: number;
  p95: number;
  p99: number;
} {
  if (values.length === 0) {
    return { min: 0, max: 0, mean: 0, median: 0, stdDev: 0, p95: 0, p99: 0 };
  }

  const { min, max } = minMax(values);

  return {
    min,
    max,
    mean: mean(values),
    median: median(values),
    stdDev: stdDev(values),
    p95: percentile(values, 95),
    p99: percentile(values, 99),
  };
}

// ============================================================================
// Calculation Helpers
// ============================================================================

/**
 * Calculate safe division (returns 0 instead of Infinity or NaN)
 */
export function safeDivide(numerator: number, denominator: number, fallback: number = 0): number {
  if (denominator === 0 || !isFinite(denominator)) return fallback;
  const result = numerator / denominator;
  return isFinite(result) ? result : fallback;
}

/**
 * Calculate percentage change
 */
export function percentageChange(oldValue: number, newValue: number): number {
  if (oldValue === 0) return newValue === 0 ? 0 : 100;
  return ((newValue - oldValue) / oldValue) * 100;
}

/**
 * Calculate growth rate
 */
export function growthRate(values: number[]): number {
  if (values.length < 2) return 0;
  const first = values[0];
  const last = values[values.length - 1];
  return percentageChange(first, last);
}

/**
 * Calculate moving average
 */
export function movingAverage(values: number[], windowSize: number): number[] {
  if (values.length === 0 || windowSize <= 0) return [];

  const result: number[] = [];
  for (let i = 0; i < values.length; i++) {
    const start = Math.max(0, i - windowSize + 1);
    const window = values.slice(start, i + 1);
    result.push(mean(window));
  }

  return result;
}

/**
 * Generate random ID (for UI elements, not cryptographic)
 */
export function randomId(): string {
  return `${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}

/**
 * Generate random number in range
 */
export function randomInRange(min: number, max: number): number {
  return Math.random() * (max - min) + min;
}

/**
 * Generate random integer in range (inclusive)
 */
export function randomInt(min: number, max: number): number {
  return Math.floor(randomInRange(min, max + 1));
}
