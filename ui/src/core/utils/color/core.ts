/**
 * Core Color Utilities
 * Foundation color manipulation powered by colord
 *
 * Benefits over custom implementation:
 * - Tiny bundle size (1.7KB gzipped)
 * - Full TypeScript support with strict typing
 * - Immutable operations for predictable behavior
 * - Extensive format support (hex, rgb, hsl, hsv, etc.)
 * - Battle-tested color space conversions
 *
 * Organization:
 * - core: Basic color creation and conversion
 * - contrast: WCAG accessibility and readability
 * - themes: Dynamic theme generation
 * - gradients: Advanced gradient utilities
 * - palettes: Color harmony and palette generation
 * - picker: Color picker utilities
 */

import { colord, extend, Colord } from "colord";
import a11yPlugin from "colord/plugins/a11y";
import harmoniesPlugin from "colord/plugins/harmonies";
import mixPlugin from "colord/plugins/mix";
import namesPlugin from "colord/plugins/names";

// Extend colord with plugins
extend([a11yPlugin, harmoniesPlugin, mixPlugin, namesPlugin]);

// ============================================================================
// Types
// ============================================================================

export interface RGB {
  r: number;
  g: number;
  b: number;
}

export interface RGBA extends RGB {
  a: number;
}

export interface HSL {
  h: number;
  s: number;
  l: number;
}

export interface HSLA extends HSL {
  a: number;
}

export interface HSV {
  h: number;
  s: number;
  v: number;
}

export type ColorFormat = "hex" | "rgb" | "rgba" | "hsl" | "hsla" | "hsv" | "name";

export type ColorInput = string | RGB | RGBA | HSL | HSLA | HSV | Colord;

// ============================================================================
// Core Color Creation
// ============================================================================

/**
 * Create a color instance from any input
 * Accepts: hex, rgb, hsl, hsv, named colors, or objects
 *
 * @example
 * color("#667eea")
 * color("rgb(102, 126, 234)")
 * color({ r: 102, g: 126, b: 234 })
 * color("blue")
 */
export function color(input: ColorInput): Colord {
  return colord(input);
}

/**
 * Validate if a string is a valid color
 */
export function isValid(input: ColorInput): boolean {
  return colord(input).isValid();
}

/**
 * Parse color from various formats with fallback
 */
export function parse(input: ColorInput, fallback: string = "#000000"): Colord {
  const parsed = colord(input);
  return parsed.isValid() ? parsed : colord(fallback);
}

// ============================================================================
// Color Conversion
// ============================================================================

/**
 * Convert color to hex format
 */
export function toHex(input: ColorInput, withAlpha: boolean = false): string {
  const c = color(input);
  return withAlpha && c.alpha() < 1 ? c.toHex() : c.toHex().substring(0, 7);
}

/**
 * Convert color to RGB object
 */
export function toRgb(input: ColorInput): RGB {
  return color(input).toRgb();
}

/**
 * Convert color to RGBA object
 */
export function toRgba(input: ColorInput): RGBA {
  const c = color(input);
  return { ...c.toRgb(), a: c.alpha() };
}

/**
 * Convert color to RGB string
 */
export function toRgbString(input: ColorInput): string {
  const { r, g, b } = toRgb(input);
  return `rgb(${r}, ${g}, ${b})`;
}

/**
 * Convert color to RGBA string
 */
export function toRgbaString(input: ColorInput, alpha?: number): string {
  const c = color(input);
  const a = alpha !== undefined ? alpha : c.alpha();
  const { r, g, b } = c.toRgb();
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

/**
 * Convert color to HSL object
 */
export function toHsl(input: ColorInput): HSL {
  return color(input).toHsl();
}

/**
 * Convert color to HSLA object
 */
export function toHsla(input: ColorInput): HSLA {
  const c = color(input);
  return { ...c.toHsl(), a: c.alpha() };
}

/**
 * Convert color to HSL string
 */
export function toHslString(input: ColorInput): string {
  const { h, s, l } = toHsl(input);
  return `hsl(${Math.round(h)}, ${Math.round(s)}%, ${Math.round(l)}%)`;
}

/**
 * Convert color to HSV object
 */
export function toHsv(input: ColorInput): HSV {
  return color(input).toHsv();
}

/**
 * Get color name if available
 */
export function toName(input: ColorInput): string | undefined {
  return color(input).toName();
}

// ============================================================================
// Color Manipulation
// ============================================================================

/**
 * Set alpha (opacity) of a color
 */
export function withAlpha(input: ColorInput, alpha: number): string {
  return color(input).alpha(alpha).toHex();
}

/**
 * Lighten a color by percentage
 */
export function lighten(input: ColorInput, amount: number): string {
  return color(input).lighten(amount).toHex();
}

/**
 * Darken a color by percentage
 */
export function darken(input: ColorInput, amount: number): string {
  return color(input).darken(amount).toHex();
}

/**
 * Saturate a color by percentage
 */
export function saturate(input: ColorInput, amount: number): string {
  return color(input).saturate(amount).toHex();
}

/**
 * Desaturate a color by percentage
 */
export function desaturate(input: ColorInput, amount: number): string {
  return color(input).desaturate(amount).toHex();
}

/**
 * Make a color grayscale
 */
export function grayscale(input: ColorInput): string {
  return color(input).grayscale().toHex();
}

/**
 * Invert a color
 */
export function invert(input: ColorInput): string {
  return color(input).invert().toHex();
}

/**
 * Rotate hue by degrees
 */
export function rotate(input: ColorInput, degrees: number): string {
  return color(input).rotate(degrees).toHex();
}

/**
 * Mix two colors together
 */
export function mix(color1: ColorInput, color2: ColorInput, ratio: number = 0.5): string {
  return colord(color1).mix(color2, ratio).toHex();
}

// ============================================================================
// Color Properties
// ============================================================================

/**
 * Get luminance value (0-1)
 */
export function luminance(input: ColorInput): number {
  return color(input).luminance();
}

/**
 * Get brightness value (0-255)
 */
export function brightness(input: ColorInput): number {
  return color(input).brightness();
}

/**
 * Check if color is light
 */
export function isLight(input: ColorInput): boolean {
  return color(input).isLight();
}

/**
 * Check if color is dark
 */
export function isDark(input: ColorInput): boolean {
  return color(input).isDark();
}

/**
 * Get alpha (opacity) value
 */
export function alpha(input: ColorInput): number {
  return color(input).alpha();
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Clamp value between 0 and 1
 */
export function clampAlpha(value: number): number {
  return Math.max(0, Math.min(1, value));
}

/**
 * Clamp value between 0 and 255
 */
export function clampRgb(value: number): number {
  return Math.max(0, Math.min(255, Math.round(value)));
}

/**
 * Clamp hue between 0 and 360
 */
export function clampHue(value: number): number {
  return ((value % 360) + 360) % 360;
}

/**
 * Clamp percentage between 0 and 100
 */
export function clampPercentage(value: number): number {
  return Math.max(0, Math.min(100, value));
}

// ============================================================================
// Color Class (for advanced usage)
// ============================================================================

/**
 * Color class for chainable operations
 * Provides a more object-oriented interface
 */
export class Color {
  private _color: Colord;

  constructor(input: ColorInput) {
    this._color = colord(input);
  }

  get value(): Colord {
    return this._color;
  }

  isValid(): boolean {
    return this._color.isValid();
  }

  // Conversions
  toHex(withAlpha: boolean = false): string {
    return withAlpha ? this._color.toHex() : this._color.toHex().substring(0, 7);
  }

  toRgb(): RGB {
    return this._color.toRgb();
  }

  toRgba(): RGBA {
    return { ...this._color.toRgb(), a: this._color.alpha() };
  }

  toHsl(): HSL {
    return this._color.toHsl();
  }

  toHsla(): HSLA {
    return { ...this._color.toHsl(), a: this._color.alpha() };
  }

  toHsv(): HSV {
    return this._color.toHsv();
  }

  toString(format: ColorFormat = "hex"): string {
    switch (format) {
      case "hex":
        return this.toHex();
      case "rgb":
        return toRgbString(this._color);
      case "rgba":
        return toRgbaString(this._color);
      case "hsl":
        return toHslString(this._color);
      case "name":
        return this._color.toName() || this.toHex();
      default:
        return this.toHex();
    }
  }

  // Manipulations
  withAlpha(alpha: number): Color {
    return new Color(this._color.alpha(alpha));
  }

  lighten(amount: number): Color {
    return new Color(this._color.lighten(amount));
  }

  darken(amount: number): Color {
    return new Color(this._color.darken(amount));
  }

  saturate(amount: number): Color {
    return new Color(this._color.saturate(amount));
  }

  desaturate(amount: number): Color {
    return new Color(this._color.desaturate(amount));
  }

  grayscale(): Color {
    return new Color(this._color.grayscale());
  }

  invert(): Color {
    return new Color(this._color.invert());
  }

  rotate(degrees: number): Color {
    return new Color(this._color.rotate(degrees));
  }

  mix(other: ColorInput, ratio: number = 0.5): Color {
    return new Color(this._color.mix(other, ratio));
  }

  // Properties
  luminance(): number {
    return this._color.luminance();
  }

  brightness(): number {
    return this._color.brightness();
  }

  isLight(): boolean {
    return this._color.isLight();
  }

  isDark(): boolean {
    return this._color.isDark();
  }

  alpha(): number {
    return this._color.alpha();
  }

  clone(): Color {
    return new Color(this._color);
  }
}
