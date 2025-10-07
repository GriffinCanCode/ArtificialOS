/**
 * OKLCH Color Utilities
 * Modern perceptually uniform color space for CSS
 *
 * Benefits of OKLCH:
 * - Perceptually uniform (better gradients)
 * - Wider color gamut support
 * - Intuitive adjustments (lightness, chroma, hue)
 * - Better for accessibility
 */

import { colord, extend } from "colord";
import labPlugin from "colord/plugins/lab";
import type { ColorInput } from "./core";
import { color } from "./core";

// Extend colord with LAB plugin (OKLCH is based on OKLAB)
extend([labPlugin]);

// ============================================================================
// Types
// ============================================================================

export interface OKLCH {
  l: number; // Lightness: 0-1 (0 = black, 1 = white)
  c: number; // Chroma: 0-0.4 (0 = gray, 0.4 = very saturated)
  h: number; // Hue: 0-360 degrees
  alpha?: number; // Alpha: 0-1
}

// ============================================================================
// Conversion Functions
// ============================================================================

/**
 * Convert any color to OKLCH values
 * Uses LAB color space as intermediate (colord doesn't have native OKLCH)
 */
export function toOklch(input: ColorInput): OKLCH {
  const c = color(input);
  const lab = c.toLab();
  const alpha = c.alpha();

  // Convert LAB to OKLCH
  // L is similar between LAB and OKLAB
  // C (chroma) = sqrt(a² + b²)
  // H (hue) = atan2(b, a)
  const l = lab.l / 100; // Normalize to 0-1
  const a = lab.a / 100; // Normalize
  const b = lab.b / 100; // Normalize

  const chroma = Math.sqrt(a * a + b * b);
  let hue = Math.atan2(b, a) * (180 / Math.PI);
  if (hue < 0) hue += 360;

  return {
    l: Math.max(0, Math.min(1, l)),
    c: Math.max(0, Math.min(0.4, chroma)),
    h: hue,
    alpha,
  };
}

/**
 * Generate CSS oklch() string
 */
export function toOklchString(input: ColorInput): string {
  const oklch = toOklch(input);
  const l = (oklch.l * 100).toFixed(2);
  const c = oklch.c.toFixed(3);
  const h = oklch.h.toFixed(1);

  if (oklch.alpha !== undefined && oklch.alpha < 1) {
    return `oklch(${l}% ${c} ${h} / ${oklch.alpha.toFixed(2)})`;
  }

  return `oklch(${l}% ${c} ${h})`;
}

/**
 * Create OKLCH color from components
 */
export function fromOklch(l: number, c: number, h: number, alpha: number = 1): string {
  // Convert OKLCH back to LAB approximation, then to RGB
  const a = c * Math.cos(h * (Math.PI / 180));
  const b = c * Math.sin(h * (Math.PI / 180));

  // Create color from LAB values (approximate)
  const labColor = colord({
    l: l * 100,
    a: a * 100,
    b: b * 100,
  });

  if (alpha < 1) {
    return labColor.alpha(alpha).toRgbString();
  }

  return labColor.toHex();
}

// ============================================================================
// Color Adjustments in OKLCH Space
// ============================================================================

/**
 * Adjust lightness in OKLCH space (perceptually uniform)
 */
export function adjustLightness(input: ColorInput, amount: number): string {
  const oklch = toOklch(input);
  const newL = Math.max(0, Math.min(1, oklch.l + amount));
  return fromOklch(newL, oklch.c, oklch.h, oklch.alpha);
}

/**
 * Adjust chroma (saturation) in OKLCH space
 */
export function adjustChroma(input: ColorInput, amount: number): string {
  const oklch = toOklch(input);
  const newC = Math.max(0, Math.min(0.4, oklch.c + amount));
  return fromOklch(oklch.l, newC, oklch.h, oklch.alpha);
}

/**
 * Rotate hue in OKLCH space
 */
export function adjustHue(input: ColorInput, degrees: number): string {
  const oklch = toOklch(input);
  let newH = oklch.h + degrees;
  while (newH < 0) newH += 360;
  while (newH >= 360) newH -= 360;
  return fromOklch(oklch.l, oklch.c, newH, oklch.alpha);
}

/**
 * Create a lighter version (in OKLCH space for better perception)
 */
export function lighterOklch(input: ColorInput, amount: number = 0.1): string {
  return adjustLightness(input, amount);
}

/**
 * Create a darker version (in OKLCH space)
 */
export function darkerOklch(input: ColorInput, amount: number = 0.1): string {
  return adjustLightness(input, -amount);
}

/**
 * Create more saturated version
 */
export function moreVibrant(input: ColorInput, amount: number = 0.05): string {
  return adjustChroma(input, amount);
}

/**
 * Create less saturated version
 */
export function lessVibrant(input: ColorInput, amount: number = 0.05): string {
  return adjustChroma(input, -amount);
}

// ============================================================================
// OKLCH Gradients
// ============================================================================

/**
 * Generate gradient in OKLCH space (perceptually smoother)
 */
export function oklchGradient(from: ColorInput, to: ColorInput, steps: number = 10): string[] {
  const startOklch = toOklch(from);
  const endOklch = toOklch(to);

  const colors: string[] = [];

  for (let i = 0; i < steps; i++) {
    const t = i / (steps - 1);

    const l = startOklch.l + (endOklch.l - startOklch.l) * t;
    const c = startOklch.c + (endOklch.c - startOklch.c) * t;

    // Handle hue interpolation (shortest path around circle)
    let h1 = startOklch.h;
    let h2 = endOklch.h;
    const diff = h2 - h1;

    if (diff > 180) {
      h1 += 360;
    } else if (diff < -180) {
      h2 += 360;
    }

    let h = h1 + (h2 - h1) * t;
    while (h < 0) h += 360;
    while (h >= 360) h -= 360;

    const alpha = (startOklch.alpha || 1) + ((endOklch.alpha || 1) - (startOklch.alpha || 1)) * t;

    colors.push(fromOklch(l, c, h, alpha));
  }

  return colors;
}

// ============================================================================
// CSS Variable Generation
// ============================================================================

/**
 * Generate OKLCH CSS variable definition
 */
export function toCssVariable(name: string, input: ColorInput): string {
  const oklchString = toOklchString(input);
  return `  --${name}: ${oklchString};`;
}

/**
 * Generate scale of OKLCH colors
 */
export function generateOklchScale(baseColor: ColorInput): Record<number, string> {
  const base = toOklch(baseColor);

  // Generate scale with perceptually uniform lightness steps
  return {
    50: toOklchString(fromOklch(0.98, base.c * 0.2, base.h)),
    100: toOklchString(fromOklch(0.95, base.c * 0.3, base.h)),
    200: toOklchString(fromOklch(0.88, base.c * 0.5, base.h)),
    300: toOklchString(fromOklch(0.78, base.c * 0.7, base.h)),
    400: toOklchString(fromOklch(0.68, base.c * 0.85, base.h)),
    500: toOklchString(fromOklch(base.l, base.c, base.h)), // Base color
    600: toOklchString(fromOklch(base.l * 0.9, base.c, base.h)),
    700: toOklchString(fromOklch(base.l * 0.75, base.c * 0.95, base.h)),
    800: toOklchString(fromOklch(base.l * 0.6, base.c * 0.85, base.h)),
    900: toOklchString(fromOklch(base.l * 0.45, base.c * 0.7, base.h)),
    950: toOklchString(fromOklch(base.l * 0.35, base.c * 0.6, base.h)),
  };
}

// ============================================================================
// Utilities
// ============================================================================

/**
 * Check if browser supports oklch()
 */
export function supportsOklch(): boolean {
  if (typeof CSS === "undefined" || !CSS.supports) return false;
  return CSS.supports("color", "oklch(0.5 0.1 180)");
}

/**
 * Get color with fallback for browsers without OKLCH support
 */
export function withFallback(input: ColorInput): string {
  if (supportsOklch()) {
    return toOklchString(input);
  }
  return color(input).toHex();
}
