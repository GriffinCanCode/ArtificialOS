/**
 * Color Utilities
 * Comprehensive color manipulation powered by colord
 *
 * Benefits:
 * - Tiny bundle size (1.7KB gzipped)
 * - Full TypeScript support with strict typing
 * - Immutable operations for predictable behavior
 * - Extensive format support (hex, rgb, hsl, hsv, etc.)
 * - WCAG accessibility compliance checking
 * - Dynamic theme generation
 * - Color harmonies and palette generation
 * - Advanced gradient utilities
 *
 * Organization:
 * - core: Basic color creation, conversion, and manipulation
 * - contrast: WCAG accessibility and readability testing
 * - themes: Dynamic theme generation
 * - gradients: Advanced gradient utilities
 * - palettes: Color harmony and palette generation
 * - cssVariables: CSS variable generation
 */

// Core color utilities
export * from "./core";
export type { ColorInput, ColorFormat, RGB, RGBA, HSL, HSLA, HSV } from "./core";

// Contrast and accessibility
export * from "./contrast";
export type {
  WCAGLevel,
  WCAGSize,
  ContrastResult,
  AccessibilityResult,
} from "./contrast";

// Gradient generation
export * from "./gradients";
export type { GradientDirection, GradientStop, GradientOptions } from "./gradients";

// Palette and harmony generation
export * from "./palettes";
export type { HarmonyType, PaletteOptions, ThemePalette } from "./palettes";

// Theme generation
export * from "./themes";
export type { ColorScale, SemanticColors, ThemeColors, Theme } from "./themes";

// CSS Variable generation
export * from "./cssVariables";

// OKLCH modern color space
export * from "./oklch";
export type { OKLCH } from "./oklch";

// Initialization
export * from "./init";

// Constants
export * from "./constants";

// Accessibility helpers
export * from "./accessibility";
