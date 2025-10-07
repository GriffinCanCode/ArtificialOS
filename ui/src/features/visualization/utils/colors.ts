/**
 * Color Utilities
 * Theme-aware color management for visualizations
 * Now powered by core color utilities (colord)
 */

import {
  toHex,
  toRgb,
  toRgbaString,
  gradient as createGradient,
  contrast,
  isWcagAA,
  type ColorInput,
} from "@/core/utils/color";

export {
  generateDataPalette as generateCategoricalPalette,
  generateSequentialPalette,
  generateDivergingPalette,
  generateHeatmapColors as generateHeatmapPalette,
  generateTimeSeriesPalette,
} from "./palettes";

// ============================================================================
// Color Palettes
// ============================================================================

export const CHART_COLORS = {
  primary: [
    "#667eea", // Primary purple
    "#764ba2", // Deep purple
    "#f093fb", // Light purple
    "#4facfe", // Blue
    "#00f2fe", // Cyan
    "#43e97b", // Green
    "#38f9d7", // Teal
    "#fa709a", // Pink
    "#fee140", // Yellow
  ],
  gradient: ["#667eea", "#764ba2", "#f093fb", "#4facfe", "#00f2fe"],
  sequential: [
    "#f7fafc",
    "#e2e8f0",
    "#cbd5e0",
    "#a0aec0",
    "#718096",
    "#4a5568",
    "#2d3748",
    "#1a202c",
  ],
  diverging: ["#4facfe", "#00f2fe", "#f7fafc", "#fee140", "#fa709a"],
  categorical: [
    "#667eea", // Purple
    "#4facfe", // Blue
    "#43e97b", // Green
    "#fee140", // Yellow
    "#fa709a", // Pink
    "#764ba2", // Deep purple
    "#38f9d7", // Teal
    "#f093fb", // Light purple
  ],
  semantic: {
    success: "#43e97b",
    warning: "#fee140",
    error: "#fa709a",
    info: "#4facfe",
    neutral: "#a0aec0",
  },
};

// ============================================================================
// Color Functions
// ============================================================================

/**
 * Get color for chart series by index
 */
export function getSeriesColor(
  index: number,
  palette: keyof typeof CHART_COLORS = "primary"
): string {
  const colors = Array.isArray(CHART_COLORS[palette])
    ? (CHART_COLORS[palette] as string[])
    : CHART_COLORS.primary;
  return colors[index % colors.length];
}

/**
 * Get semantic color by type
 */
export function getSemanticColor(type: keyof typeof CHART_COLORS.semantic): string {
  return CHART_COLORS.semantic[type];
}

/**
 * Generate gradient from two colors
 * Now uses core gradient utility
 */
export function generateGradient(from: string, to: string, stops = 10): string[] {
  return createGradient(from, to, stops);
}

/**
 * Convert hex color to RGB
 * Now uses core color utility
 */
export function hexToRgb(hex: string): { r: number; g: number; b: number } {
  return toRgb(hex);
}

/**
 * Convert RGB to hex color
 * Now uses core color utility
 */
export function rgbToHex(r: number, g: number, b: number): string {
  return toHex({ r, g, b });
}

/**
 * Adjust color opacity
 * Now uses core color utility
 */
export function withOpacity(color: ColorInput, opacity: number): string {
  return toRgbaString(color, opacity);
}

/**
 * Get color for value in range
 */
export function colorFromValue(
  value: number,
  min: number,
  max: number,
  palette: string[] = CHART_COLORS.sequential
): string {
  const normalized = Math.max(0, Math.min(1, (value - min) / (max - min)));
  const index = Math.floor(normalized * (palette.length - 1));
  return palette[index];
}

/**
 * Validate color contrast for accessibility
 * Ensures text is readable on given background
 */
export function validateColorContrast(foreground: string, background: string): {
  ratio: number;
  isAccessible: boolean;
  recommendation: string;
} {
  const ratio = contrast(foreground, background);
  const isAccessible = isWcagAA(foreground, background);

  let recommendation = "";
  if (ratio >= 7) {
    recommendation = "Excellent contrast (AAA)";
  } else if (ratio >= 4.5) {
    recommendation = "Good contrast (AA)";
  } else if (ratio >= 3) {
    recommendation = "Acceptable for large text only";
  } else {
    recommendation = "Poor contrast - not recommended";
  }

  return {
    ratio: Math.round(ratio * 100) / 100,
    isAccessible,
    recommendation,
  };
}
