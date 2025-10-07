/**
 * Color Palette & Harmony Utilities
 * Generate harmonious color schemes and palettes
 */

import { colord, extend } from "colord";
import harmoniesPlugin from "colord/plugins/harmonies";
import type { ColorInput } from "./core";
import { color, lighten, darken, saturate, desaturate } from "./core";

extend([harmoniesPlugin]);

// ============================================================================
// Types
// ============================================================================

export type HarmonyType =
  | "analogous"
  | "complementary"
  | "split-complementary"
  | "triadic"
  | "tetradic"
  | "square";

export interface PaletteOptions {
  count?: number;
  variation?: "light" | "dark" | "saturated" | "desaturated" | "mixed";
}

export interface ThemePalette {
  primary: string;
  secondary: string;
  accent: string;
  background: string;
  surface: string;
  text: string;
  textSecondary: string;
  border: string;
  error: string;
  warning: string;
  success: string;
  info: string;
}

// ============================================================================
// Color Harmonies
// ============================================================================

/**
 * Generate analogous colors (adjacent on color wheel)
 * Creates harmonious, cohesive palettes
 *
 * @example
 * analogous("#667eea") // 3 colors including base
 */
export function analogous(baseColor: ColorInput, count: number = 3): string[] {
  const base = colord(baseColor);
  const harmonies = base.harmonies("analogous");

  // Return requested number of colors
  return harmonies.slice(0, count).map((c) => c.toHex());
}

/**
 * Generate complementary color (opposite on color wheel)
 * Creates high contrast, vibrant combinations
 */
export function complementary(baseColor: ColorInput): string[] {
  const base = colord(baseColor);
  return base.harmonies("complementary").map((c) => c.toHex());
}

/**
 * Generate split-complementary colors
 * Softer than complementary, still high contrast
 */
export function splitComplementary(baseColor: ColorInput): string[] {
  const base = colord(baseColor);
  return base.harmonies("double-split-complementary").slice(0, 3).map((c) => c.toHex());
}

/**
 * Generate triadic colors (evenly spaced on color wheel)
 * Vibrant, balanced color scheme
 */
export function triadic(baseColor: ColorInput): string[] {
  const base = colord(baseColor);
  return base.harmonies("triadic").map((c) => c.toHex());
}

/**
 * Generate tetradic (rectangle) colors
 * Two complementary pairs
 */
export function tetradic(baseColor: ColorInput): string[] {
  const base = colord(baseColor);
  return base.harmonies("tetradic").map((c) => c.toHex());
}

/**
 * Generate square colors (tetradic with 90° spacing)
 * Evenly distributed, vibrant scheme
 */
export function square(baseColor: ColorInput): string[] {
  const base = colord(baseColor);
  return base.harmonies("tetradic").map((c) => c.toHex());
}

/**
 * Generate color harmony by type
 */
export function harmony(baseColor: ColorInput, type: HarmonyType): string[] {
  const base = colord(baseColor);

  switch (type) {
    case "analogous":
      return base.harmonies("analogous").map((c) => c.toHex());
    case "complementary":
      return base.harmonies("complementary").map((c) => c.toHex());
    case "split-complementary":
      return base.harmonies("double-split-complementary").slice(0, 3).map((c) => c.toHex());
    case "triadic":
      return base.harmonies("triadic").map((c) => c.toHex());
    case "tetradic":
    case "square":
      return base.harmonies("tetradic").map((c) => c.toHex());
    default:
      return [base.toHex()];
  }
}

// ============================================================================
// Palette Generation
// ============================================================================

/**
 * Generate monochromatic palette (variations of single hue)
 */
export function monochrome(
  baseColor: ColorInput,
  options: PaletteOptions = {}
): string[] {
  const { count = 5, variation = "mixed" } = options;
  const base = color(baseColor);
  const colors: string[] = [];

  if (variation === "mixed") {
    // Generate lighter and darker versions
    const step = 0.1;
    for (let i = 0; i < count; i++) {
      const offset = (i - Math.floor(count / 2)) * step;
      const adjusted =
        offset > 0
          ? lighten(base, Math.abs(offset))
          : offset < 0
            ? darken(base, Math.abs(offset))
            : base.toHex();
      colors.push(adjusted);
    }
  } else if (variation === "light") {
    for (let i = 0; i < count; i++) {
      colors.push(lighten(base, (i / count) * 0.5));
    }
  } else if (variation === "dark") {
    for (let i = 0; i < count; i++) {
      colors.push(darken(base, (i / count) * 0.5));
    }
  } else if (variation === "saturated") {
    for (let i = 0; i < count; i++) {
      colors.push(saturate(base, (i / count) * 0.5));
    }
  } else if (variation === "desaturated") {
    for (let i = 0; i < count; i++) {
      colors.push(desaturate(base, (i / count) * 0.5));
    }
  }

  return colors;
}

/**
 * Generate tints (lighter versions by mixing with white)
 */
export function tints(baseColor: ColorInput, count: number = 5): string[] {
  const base = color(baseColor);
  const colors: string[] = [];

  for (let i = 0; i < count; i++) {
    const amount = i / (count - 1);
    colors.push(base.mix("#ffffff", amount).toHex());
  }

  return colors;
}

/**
 * Generate shades (darker versions by mixing with black)
 */
export function shades(baseColor: ColorInput, count: number = 5): string[] {
  const base = color(baseColor);
  const colors: string[] = [];

  for (let i = 0; i < count; i++) {
    const amount = i / (count - 1);
    colors.push(base.mix("#000000", amount).toHex());
  }

  return colors;
}

/**
 * Generate tones (mixing with gray)
 */
export function tones(baseColor: ColorInput, count: number = 5): string[] {
  const base = color(baseColor);
  const colors: string[] = [];

  for (let i = 0; i < count; i++) {
    const amount = i / (count - 1);
    colors.push(base.mix("#808080", amount).toHex());
  }

  return colors;
}

/**
 * Generate full palette with tints, base, and shades
 */
export function fullPalette(baseColor: ColorInput, steps: number = 9): string[] {
  const halfSteps = Math.floor(steps / 2);

  const tintsArray = tints(baseColor, halfSteps).reverse().slice(1);
  const shadesArray = shades(baseColor, halfSteps + 1).slice(1);

  return [...tintsArray, color(baseColor).toHex(), ...shadesArray];
}

// ============================================================================
// Material Design Style Palettes
// ============================================================================

/**
 * Generate Material Design style palette (50-900)
 */
export function materialPalette(baseColor: ColorInput): Record<number, string> {
  const base = color(baseColor);

  return {
    50: base.mix("#ffffff", 0.95).toHex(),
    100: base.mix("#ffffff", 0.9).toHex(),
    200: base.mix("#ffffff", 0.7).toHex(),
    300: base.mix("#ffffff", 0.5).toHex(),
    400: base.mix("#ffffff", 0.3).toHex(),
    500: base.toHex(),
    600: base.darken(0.1).toHex(),
    700: base.darken(0.2).toHex(),
    800: base.darken(0.3).toHex(),
    900: base.darken(0.4).toHex(),
  };
}

// ============================================================================
// Theme Generation
// ============================================================================

/**
 * Generate complete theme palette from single base color
 */
export function generateThemePalette(baseColor: ColorInput): ThemePalette {
  const base = color(baseColor);
  const harmonies = triadic(base);

  return {
    primary: base.toHex(),
    secondary: harmonies[1],
    accent: harmonies[2],
    background: "#0a0a0a",
    surface: "#1a1a1a",
    text: "#ffffff",
    textSecondary: base.mix("#ffffff", 0.7).alpha(0.7).toRgbString(),
    border: "#2a2a2a",
    error: "#ef4444",
    warning: "#f59e0b",
    success: "#10b981",
    info: "#3b82f6",
  };
}

/**
 * Generate light theme palette
 */
export function generateLightThemePalette(baseColor: ColorInput): ThemePalette {
  const base = color(baseColor);
  const harmonies = triadic(base);

  return {
    primary: base.toHex(),
    secondary: harmonies[1],
    accent: harmonies[2],
    background: "#ffffff",
    surface: "#f7f7f7",
    text: "#1a1a1a",
    textSecondary: base.mix("#000000", 0.4).alpha(0.6).toRgbString(),
    border: "#e0e0e0",
    error: "#dc2626",
    warning: "#d97706",
    success: "#059669",
    info: "#2563eb",
  };
}

/**
 * Generate dark theme palette
 */
export function generateDarkThemePalette(baseColor: ColorInput): ThemePalette {
  return generateThemePalette(baseColor);
}

// ============================================================================
// Categorical Palettes
// ============================================================================

/**
 * Generate categorical colors (distinct, evenly distributed)
 * Perfect for charts and data visualization
 */
export function categorical(count: number, saturation: number = 0.7): string[] {
  const colors: string[] = [];
  const hueStep = 360 / count;

  for (let i = 0; i < count; i++) {
    const hue = i * hueStep;
    const c = colord({ h: hue, s: saturation * 100, l: 50 });
    colors.push(c.toHex());
  }

  return colors;
}

/**
 * Generate sequential colors (for ordered data)
 */
export function sequential(
  baseColor: ColorInput,
  count: number = 9,
  reverse: boolean = false
): string[] {
  const palette = fullPalette(baseColor, count);
  return reverse ? palette.reverse() : palette;
}

/**
 * Generate diverging colors (for data with positive/negative)
 */
export function diverging(
  negativeColor: ColorInput,
  neutralColor: ColorInput,
  positiveColor: ColorInput,
  steps: number = 11
): string[] {
  const halfSteps = Math.floor(steps / 2);

  const negative = color(negativeColor);
  const neutral = color(neutralColor);
  const positive = color(positiveColor);

  const negativeGradient: string[] = [];
  const positiveGradient: string[] = [];

  for (let i = 0; i < halfSteps; i++) {
    const ratio = i / (halfSteps - 1);
    negativeGradient.push(negative.mix(neutral, ratio).toHex());
    positiveGradient.push(neutral.mix(positive, ratio).toHex());
  }

  // If odd number of steps, include neutral in middle
  if (steps % 2 === 1) {
    return [...negativeGradient, neutral.toHex(), ...positiveGradient.slice(1)];
  }

  return [...negativeGradient, ...positiveGradient];
}

// ============================================================================
// Random Palettes
// ============================================================================

/**
 * Generate random harmonious colors
 */
export function randomPalette(count: number = 5, harmonyType?: HarmonyType): string[] {
  const randomHue = Math.floor(Math.random() * 360);
  const baseColor = colord({ h: randomHue, s: 70, l: 50 });

  if (harmonyType) {
    const harmonies = harmony(baseColor, harmonyType);
    return harmonies.slice(0, count);
  }

  return categorical(count);
}
