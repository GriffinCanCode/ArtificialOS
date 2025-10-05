/**
 * Number Formatting Utilities
 * Pure functions for number transformation and formatting
 */

// ============================================================================
// Basic Formatting
// ============================================================================

export function formatNumber(value: number, decimals: number = 0): string {
  return value.toFixed(decimals);
}

export function formatThousands(value: number, separator: string = ","): string {
  return value.toString().replace(/\B(?=(\d{3})+(?!\d))/g, separator);
}

export function formatCurrency(
  value: number,
  currency: string = "$",
  decimals: number = 2
): string {
  const formatted = Math.abs(value).toFixed(decimals);
  const withThousands = formatThousands(parseFloat(formatted));
  return value < 0 ? `-${currency}${withThousands}` : `${currency}${withThousands}`;
}

export function formatPercentage(value: number, decimals: number = 0): string {
  return `${(value * 100).toFixed(decimals)}%`;
}

// ============================================================================
// Size Formatting
// ============================================================================

export function formatBytes(bytes: number, decimals: number = 2): string {
  if (bytes === 0) return "0 Bytes";

  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`;
}

export function formatFileSize(bytes: number): string {
  return formatBytes(bytes, 1);
}

// ============================================================================
// Compact Formatting
// ============================================================================

export function formatCompact(value: number): string {
  if (value < 1000) return value.toString();
  if (value < 1000000) return `${(value / 1000).toFixed(1)}K`;
  if (value < 1000000000) return `${(value / 1000000).toFixed(1)}M`;
  return `${(value / 1000000000).toFixed(1)}B`;
}

export function formatLargeNumber(value: number): string {
  const suffixes = ["", "K", "M", "B", "T"];
  const tier = (Math.log10(Math.abs(value)) / 3) | 0;

  if (tier === 0) return value.toString();

  const suffix = suffixes[tier];
  const scale = Math.pow(10, tier * 3);
  const scaled = value / scale;

  return scaled.toFixed(1) + suffix;
}

// ============================================================================
// Range Operations
// ============================================================================

export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function normalize(value: number, min: number, max: number): number {
  return (value - min) / (max - min);
}

export function denormalize(normalized: number, min: number, max: number): number {
  return normalized * (max - min) + min;
}

export function lerp(start: number, end: number, t: number): number {
  return start + (end - start) * t;
}

// ============================================================================
// Parsing
// ============================================================================

export function parseNumber(value: string): number | null {
  const cleaned = value.replace(/[^0-9.-]/g, "");
  const parsed = parseFloat(cleaned);
  return isNaN(parsed) ? null : parsed;
}

export function parseInt(value: string, radix: number = 10): number | null {
  const parsed = Number.parseInt(value, radix);
  return isNaN(parsed) ? null : parsed;
}

// ============================================================================
// Rounding
// ============================================================================

export function roundTo(value: number, decimals: number): number {
  const multiplier = Math.pow(10, decimals);
  return Math.round(value * multiplier) / multiplier;
}

export function ceilTo(value: number, decimals: number): number {
  const multiplier = Math.pow(10, decimals);
  return Math.ceil(value * multiplier) / multiplier;
}

export function floorTo(value: number, decimals: number): number {
  const multiplier = Math.pow(10, decimals);
  return Math.floor(value * multiplier) / multiplier;
}

// ============================================================================
// Ordinal
// ============================================================================

export function ordinal(value: number): string {
  const s = ["th", "st", "nd", "rd"];
  const v = value % 100;
  return value + (s[(v - 20) % 10] || s[v] || s[0]);
}
