/**
 * Advanced Palette Generation
 * Specialized palettes for data visualization
 */

import { categorical, sequential, diverging, heatmapGradient } from "@/core/utils/color";
import type { ColorInput } from "@/core/utils/color";

// ============================================================================
// Dynamic Palette Generation
// ============================================================================

/**
 * Generate optimal categorical palette for N data series
 * Better than static arrays - ensures maximum visual distinction
 */
export function generateDataPalette(seriesCount: number): string[] {
  if (seriesCount <= 0) return [];
  if (seriesCount === 1) return ["#667eea"];

  return categorical(seriesCount, 0.7);
}

/**
 * Generate sequential palette for ordered data
 * Perfect for heatmaps, choropleth maps, etc.
 */
export function generateSequentialPalette(baseColor: ColorInput, steps: number = 9): string[] {
  return sequential(baseColor, steps);
}

/**
 * Generate diverging palette for positive/negative data
 * Great for showing deviation from a center point
 */
export function generateDivergingPalette(steps: number = 11): string[] {
  return diverging("#ef4444", "#f3f4f6", "#10b981", steps);
}

/**
 * Generate heatmap colors
 * Blue (cold) -> Green -> Yellow -> Red (hot)
 */
export function generateHeatmapColors(steps: number = 100): string[] {
  return heatmapGradient(steps);
}

/**
 * Generate palette for time-series data
 * Ensures colors are distinguishable even with many series
 */
export function generateTimeSeriesPalette(seriesCount: number): string[] {
  return generateDataPalette(seriesCount);
}

/**
 * Generate palette for categorical data with custom base
 */
export function generateCategoricalFromBase(baseColor: ColorInput, count: number): string[] {
  return sequential(baseColor, count);
}
