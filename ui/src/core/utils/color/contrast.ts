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
export function contrastDetails(foreground: ColorInput, background: ColorInput): ContrastResult {
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
 * Transform RGB to LMS color space (cone response)
 * Using Hunt-Pointer-Estevez transformation matrix
 */
function rgbToLms(r: number, g: number, b: number): [number, number, number] {
  // Normalize RGB to [0, 1]
  const rNorm = r / 255;
  const gNorm = g / 255;
  const bNorm = b / 255;

  // Hunt-Pointer-Estevez D65 matrix
  const l = 0.31399022 * rNorm + 0.63951294 * gNorm + 0.04649755 * bNorm;
  const m = 0.15537241 * rNorm + 0.75789446 * gNorm + 0.08670142 * bNorm;
  const s = 0.01775239 * rNorm + 0.10944209 * gNorm + 0.87256922 * bNorm;

  return [l, m, s];
}

/**
 * Transform LMS back to RGB color space
 */
function lmsToRgb(l: number, m: number, s: number): [number, number, number] {
  // Inverse Hunt-Pointer-Estevez matrix
  const r = 5.47221206 * l - 4.6419601 * m + 0.16963708 * s;
  const g = -1.1252419 * l + 2.29317094 * m - 0.1678952 * s;
  const b = 0.02980165 * l - 0.19318073 * m + 1.16364789 * s;

  // Clamp to [0, 1] and convert to [0, 255]
  const rOut = Math.max(0, Math.min(255, Math.round(r * 255)));
  const gOut = Math.max(0, Math.min(255, Math.round(g * 255)));
  const bOut = Math.max(0, Math.min(255, Math.round(b * 255)));

  return [rOut, gOut, bOut];
}

/**
 * Simulate color blindness (protanopia - red-blind)
 * Based on Brettel, Viénot and Mollon (1997) and Viénot, Brettel and Mollon (1999)
 * Uses LMS color space for accurate simulation
 */
export function simulateProtanopia(input: ColorInput): string {
  const c = color(input);
  const { r, g, b } = c.toRgb();

  // Convert to LMS
  const [_l, m, s] = rgbToLms(r, g, b);

  // Protanopia: L cone absent, simulate using M and S
  // L = 2.02344 * M - 2.52581 * S
  const lSimulated = 2.02344 * m - 2.52581 * s;

  // Convert back to RGB
  const [rOut, gOut, bOut] = lmsToRgb(lSimulated, m, s);

  return colord({ r: rOut, g: gOut, b: bOut }).toHex();
}

/**
 * Simulate deuteranopia (green-blind)
 * Based on Brettel, Viénot and Mollon (1997) and Viénot, Brettel and Mollon (1999)
 * Uses LMS color space for accurate simulation
 */
export function simulateDeuteranopia(input: ColorInput): string {
  const c = color(input);
  const { r, g, b } = c.toRgb();

  // Convert to LMS
  const [l, _m, s] = rgbToLms(r, g, b);

  // Deuteranopia: M cone absent, simulate using L and S
  // M = 0.494207 * L + 1.24827 * S
  const mSimulated = 0.494207 * l + 1.24827 * s;

  // Convert back to RGB
  const [rOut, gOut, bOut] = lmsToRgb(l, mSimulated, s);

  return colord({ r: rOut, g: gOut, b: bOut }).toHex();
}

/**
 * Simulate tritanopia (blue-blind)
 * Based on Brettel, Viénot and Mollon (1997) and Viénot, Brettel and Mollon (1999)
 * Uses LMS color space for accurate simulation
 */
export function simulateTritanopia(input: ColorInput): string {
  const c = color(input);
  const { r, g, b } = c.toRgb();

  // Convert to LMS
  const [l, m, _s] = rgbToLms(r, g, b);

  // Tritanopia: S cone absent, simulate using L and M
  // S = -0.395913 * L + 0.801109 * M
  const sSimulated = -0.395913 * l + 0.801109 * m;

  // Convert back to RGB
  const [rOut, gOut, bOut] = lmsToRgb(l, m, sSimulated);

  return colord({ r: rOut, g: gOut, b: bOut }).toHex();
}

/**
 * Simulate protanomaly (weak red vision)
 * Partial simulation with adjustable severity (0-1)
 */
export function simulateProtanomaly(input: ColorInput, severity: number = 0.6): string {
  const original = color(input);
  const simulated = simulateProtanopia(input);

  // Blend between original and full simulation based on severity
  return original.mix(simulated, severity).toHex();
}

/**
 * Simulate deuteranomaly (weak green vision)
 * Partial simulation with adjustable severity (0-1)
 */
export function simulateDeuteranomaly(input: ColorInput, severity: number = 0.6): string {
  const original = color(input);
  const simulated = simulateDeuteranopia(input);

  // Blend between original and full simulation based on severity
  return original.mix(simulated, severity).toHex();
}

/**
 * Simulate tritanomaly (weak blue vision)
 * Partial simulation with adjustable severity (0-1)
 */
export function simulateTritanomaly(input: ColorInput, severity: number = 0.6): string {
  const original = color(input);
  const simulated = simulateTritanopia(input);

  // Blend between original and full simulation based on severity
  return original.mix(simulated, severity).toHex();
}
