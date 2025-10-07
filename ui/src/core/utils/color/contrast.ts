/**
 * Color Contrast & Accessibility Utilities
 * WCAG 2.1 compliance checking and readability testing
 */

import { colord, extend } from "colord";
import a11yPlugin from "colord/plugins/a11y";
import type { ColorInput } from "./core";
import { color } from "./core";

extend([a11yPlugin]);

// ============================================================================
// Types
// ============================================================================

export type WCAGLevel = "AA" | "AAA";
export type WCAGSize = "normal" | "large";

export interface ContrastResult {
  ratio: number;
  AA: boolean;
  AAA: boolean;
  AAALarge: boolean;
  AALarge: boolean;
}

export interface AccessibilityResult {
  isReadable: boolean;
  contrast: number;
  wcagAA: boolean;
  wcagAAA: boolean;
  recommendation: string;
}

// ============================================================================
// Contrast Calculation
// ============================================================================

/**
 * Calculate contrast ratio between two colors (1-21)
 * Based on WCAG 2.1 guidelines
 *
 * @example
 * contrast("#ffffff", "#000000") // 21 (maximum contrast)
 * contrast("#667eea", "#ffffff") // ~2.5
 */
export function contrast(foreground: ColorInput, background: ColorInput): number {
  const fg = color(foreground);
  const bg = color(background);

  const l1 = fg.luminance();
  const l2 = bg.luminance();

  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);

  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Get detailed contrast information
 */
export function contrastDetails(
  foreground: ColorInput,
  background: ColorInput
): ContrastResult {
  const ratio = contrast(foreground, background);

  return {
    ratio: Math.round(ratio * 100) / 100,
    AA: ratio >= 4.5,
    AAA: ratio >= 7,
    AALarge: ratio >= 3,
    AAALarge: ratio >= 4.5,
  };
}

// ============================================================================
// WCAG Compliance
// ============================================================================

/**
 * Check if color combination meets WCAG AA standards
 * - Normal text: 4.5:1
 * - Large text (18pt+ or 14pt+ bold): 3:1
 */
export function isWcagAA(
  foreground: ColorInput,
  background: ColorInput,
  size: WCAGSize = "normal"
): boolean {
  const ratio = contrast(foreground, background);
  return size === "large" ? ratio >= 3 : ratio >= 4.5;
}

/**
 * Check if color combination meets WCAG AAA standards
 * - Normal text: 7:1
 * - Large text: 4.5:1
 */
export function isWcagAAA(
  foreground: ColorInput,
  background: ColorInput,
  size: WCAGSize = "normal"
): boolean {
  const ratio = contrast(foreground, background);
  return size === "large" ? ratio >= 4.5 : ratio >= 7;
}

/**
 * Check if text is readable on background
 * Uses colord's built-in readability check
 */
export function isReadable(
  foreground: ColorInput,
  background: ColorInput,
  level: WCAGLevel = "AA",
  size: WCAGSize = "normal"
): boolean {
  const fg = colord(foreground);
  return fg.isReadable(background, {
    level,
    size,
  });
}

/**
 * Get comprehensive accessibility assessment
 */
export function accessibility(
  foreground: ColorInput,
  background: ColorInput,
  size: WCAGSize = "normal"
): AccessibilityResult {
  const contrastRatio = contrast(foreground, background);
  const aa = isWcagAA(foreground, background, size);
  const aaa = isWcagAAA(foreground, background, size);

  let recommendation = "";
  if (aaa) {
    recommendation = "Excellent contrast - passes WCAG AAA";
  } else if (aa) {
    recommendation = "Good contrast - passes WCAG AA";
  } else {
    recommendation = "Poor contrast - fails WCAG standards";
  }

  return {
    isReadable: aa,
    contrast: Math.round(contrastRatio * 100) / 100,
    wcagAA: aa,
    wcagAAA: aaa,
    recommendation,
  };
}

// ============================================================================
// Color Adjustment for Accessibility
// ============================================================================

/**
 * Find the closest color that meets WCAG standards
 * Adjusts lightness to achieve target contrast
 */
export function ensureContrast(
  foreground: ColorInput,
  background: ColorInput,
  level: WCAGLevel = "AA",
  size: WCAGSize = "normal"
): string {
  let fg = color(foreground);
  const bg = color(background);

  // Already readable
  if (isReadable(fg, bg, level, size)) {
    return fg.toHex();
  }

  // Determine target contrast ratio
  const targetRatio = level === "AAA" ? (size === "large" ? 4.5 : 7) : size === "large" ? 3 : 4.5;

  // Try lightening and darkening
  const bgLuminance = bg.luminance();
  const shouldLighten = bgLuminance < 0.5;

  let step = shouldLighten ? 0.05 : -0.05;
  let attempts = 0;
  const maxAttempts = 20;

  while (attempts < maxAttempts) {
    const adjusted = shouldLighten
      ? fg.lighten(Math.abs(step) * attempts)
      : fg.darken(Math.abs(step) * attempts);

    if (contrast(adjusted, bg) >= targetRatio) {
      return adjusted.toHex();
    }

    attempts++;
  }

  // Fallback: use pure white or black
  return shouldLighten ? "#ffffff" : "#000000";
}

/**
 * Get best text color (black or white) for a background
 */
export function bestTextColor(background: ColorInput, threshold: number = 0.5): string {
  const bg = color(background);
  return bg.luminance() > threshold ? "#000000" : "#ffffff";
}

/**
 * Get optimal text color that meets WCAG AA
 */
export function optimalTextColor(
  background: ColorInput,
  preferredColor?: ColorInput,
  level: WCAGLevel = "AA"
): string {
  const bg = color(background);

  // Try preferred color first
  if (preferredColor) {
    const preferred = color(preferredColor);
    if (isReadable(preferred, bg, level)) {
      return preferred.toHex();
    }
  }

  // Try black and white
  const black = "#000000";
  const white = "#ffffff";

  const blackContrast = contrast(black, bg);
  const whiteContrast = contrast(white, bg);

  // Return whichever has better contrast
  return whiteContrast > blackContrast ? white : black;
}

// ============================================================================
// Color Blindness Simulation
// ============================================================================

/**
 * Simulate color blindness (protanopia - red-blind)
 * This is a simplified simulation
 */
export function simulateProtanopia(input: ColorInput): string {
  const c = color(input);
  const { r, g, b } = c.toRgb();

  // Simplified protanopia matrix
  const newR = 0.567 * r + 0.433 * g;
  const newG = 0.558 * r + 0.442 * g;
  const newB = 0.242 * g + 0.758 * b;

  return colord({
    r: Math.round(newR),
    g: Math.round(newG),
    b: Math.round(newB),
  }).toHex();
}

/**
 * Simulate deuteranopia (green-blind)
 */
export function simulateDeuteranopia(input: ColorInput): string {
  const c = color(input);
  const { r, g, b } = c.toRgb();

  const newR = 0.625 * r + 0.375 * g;
  const newG = 0.7 * r + 0.3 * g;
  const newB = 0.3 * g + 0.7 * b;

  return colord({
    r: Math.round(newR),
    g: Math.round(newG),
    b: Math.round(newB),
  }).toHex();
}

/**
 * Simulate tritanopia (blue-blind)
 */
export function simulateTritanopia(input: ColorInput): string {
  const c = color(input);
  const { r, g, b } = c.toRgb();

  const newR = 0.95 * r + 0.05 * g;
  const newG = 0.433 * g + 0.567 * b;
  const newB = 0.475 * g + 0.525 * b;

  return colord({
    r: Math.round(newR),
    g: Math.round(newG),
    b: Math.round(newB),
  }).toHex();
}
