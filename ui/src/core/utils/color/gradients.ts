/**
 * Gradient Generation Utilities
 * Create sophisticated gradients with proper color interpolation
 * Now with OKLCH support for perceptually uniform gradients
 */

import { colord } from "colord";
import type { ColorInput } from "./core";
import { color } from "./core";
import { oklchGradient, supportsOklch, toOklchString } from "./oklch";

// ============================================================================
// Types
// ============================================================================

export type GradientDirection =
  | "to-right"
  | "to-left"
  | "to-top"
  | "to-bottom"
  | "to-top-right"
  | "to-top-left"
  | "to-bottom-right"
  | "to-bottom-left"
  | number; // angle in degrees

export interface GradientStop {
  color: string;
  position: number; // 0-100
}

export interface GradientOptions {
  steps?: number;
  mode?: "rgb" | "lrgb" | "hsl" | "hsv" | "lab" | "lch";
  easing?: "linear" | "ease-in" | "ease-out" | "ease-in-out";
}

// ============================================================================
// Gradient Generation
// ============================================================================

/**
 * Generate gradient color array between two colors
 * Uses OKLCH when supported for perceptually uniform gradients
 *
 * @example
 * gradient("#667eea", "#764ba2", 5)
 * // ["#667eea", "#6d73dd", "#7468d0", "#7a5dc3", "#764ba2"]
 */
export function gradient(
  from: ColorInput,
  to: ColorInput,
  steps: number = 10,
  options: GradientOptions = {}
): string[] {
  const { easing = "linear" } = options;

  // Use OKLCH gradient for perceptually uniform transitions (when linear easing)
  if (supportsOklch() && easing === "linear") {
    return oklchGradient(from, to, steps);
  }

  // Fallback to RGB mixing for custom easing or no OKLCH support
  const colors: string[] = [];
  const start = color(from);
  const end = color(to);

  for (let i = 0; i < steps; i++) {
    const t = applyEasing(i / (steps - 1), easing);
    const mixed = start.mix(end, t);
    colors.push(mixed.toHex());
  }

  return colors;
}

/**
 * Generate gradient with multiple color stops
 *
 * @example
 * multiStopGradient([
 *   { color: "#667eea", position: 0 },
 *   { color: "#764ba2", position: 50 },
 *   { color: "#f093fb", position: 100 }
 * ], 10)
 */
export function multiStopGradient(stops: GradientStop[], totalSteps: number = 100): string[] {
  if (stops.length < 2) {
    throw new Error("At least 2 color stops required");
  }

  // Sort stops by position
  const sortedStops = [...stops].sort((a, b) => a.position - b.position);

  const colors: string[] = [];

  for (let i = 0; i < totalSteps; i++) {
    const position = (i / (totalSteps - 1)) * 100;

    // Find the two stops this position falls between
    let beforeStop = sortedStops[0];
    let afterStop = sortedStops[sortedStops.length - 1];

    for (let j = 0; j < sortedStops.length - 1; j++) {
      if (position >= sortedStops[j].position && position <= sortedStops[j + 1].position) {
        beforeStop = sortedStops[j];
        afterStop = sortedStops[j + 1];
        break;
      }
    }

    // Interpolate between the two stops
    const range = afterStop.position - beforeStop.position;
    const t = range === 0 ? 0 : (position - beforeStop.position) / range;

    const mixed = colord(beforeStop.color).mix(afterStop.color, t);
    colors.push(mixed.toHex());
  }

  return colors;
}

/**
 * Generate CSS linear gradient string
 * Uses OKLCH when supported for smoother gradients
 *
 * @example
 * cssLinearGradient("#667eea", "#764ba2", "to-right")
 * // "linear-gradient(to right, oklch(...), oklch(...))"
 */
export function cssLinearGradient(
  from: ColorInput,
  to: ColorInput,
  direction: GradientDirection = "to-right"
): string {
  const useOklch = supportsOklch();
  const start = useOklch ? toOklchString(from) : color(from).toHex();
  const end = useOklch ? toOklchString(to) : color(to).toHex();

  const dir = typeof direction === "number" ? `${direction}deg` : direction.replace("-", " ");

  return `linear-gradient(${dir}, ${start}, ${end})`;
}

/**
 * Generate CSS linear gradient with multiple stops
 * Uses OKLCH when supported for better color transitions
 */
export function cssMultiStopGradient(
  stops: GradientStop[],
  direction: GradientDirection = "to-right"
): string {
  const dir = typeof direction === "number" ? `${direction}deg` : direction.replace("-", " ");
  const useOklch = supportsOklch();

  const stopStrings = stops
    .map((stop) => {
      const colorStr = useOklch ? toOklchString(stop.color) : color(stop.color).toHex();
      return `${colorStr} ${stop.position}%`;
    })
    .join(", ");

  return `linear-gradient(${dir}, ${stopStrings})`;
}

/**
 * Generate CSS radial gradient
 */
export function cssRadialGradient(
  from: ColorInput,
  to: ColorInput,
  shape: "circle" | "ellipse" = "circle"
): string {
  const start = color(from).toHex();
  const end = color(to).toHex();

  return `radial-gradient(${shape}, ${start}, ${end})`;
}

/**
 * Generate CSS conic gradient
 */
export function cssConicGradient(colors: ColorInput[], startAngle: number = 0): string {
  const hexColors = colors.map((c) => color(c).toHex()).join(", ");
  return `conic-gradient(from ${startAngle}deg, ${hexColors})`;
}

// ============================================================================
// Gradient Analysis
// ============================================================================

/**
 * Extract colors from a gradient array at specific positions
 */
export function sampleGradient(gradientColors: string[], positions: number[]): string[] {
  return positions.map((pos) => {
    const normalizedPos = Math.max(0, Math.min(1, pos));
    const index = Math.floor(normalizedPos * (gradientColors.length - 1));
    return gradientColors[index];
  });
}

/**
 * Reverse a gradient
 */
export function reverseGradient(gradientColors: string[]): string[] {
  return [...gradientColors].reverse();
}

/**
 * Create gradient with smooth transitions using bezier-like easing
 */
export function smoothGradient(
  from: ColorInput,
  to: ColorInput,
  steps: number = 10
): string[] {
  return gradient(from, to, steps, { easing: "ease-in-out" });
}

// ============================================================================
// Specialized Gradients
// ============================================================================

/**
 * Generate a heatmap gradient (blue -> green -> yellow -> red)
 */
export function heatmapGradient(steps: number = 100): string[] {
  return multiStopGradient(
    [
      { color: "#0000ff", position: 0 }, // Blue
      { color: "#00ff00", position: 33 }, // Green
      { color: "#ffff00", position: 66 }, // Yellow
      { color: "#ff0000", position: 100 }, // Red
    ],
    steps
  );
}

/**
 * Generate a rainbow gradient
 */
export function rainbowGradient(steps: number = 100): string[] {
  return multiStopGradient(
    [
      { color: "#ff0000", position: 0 }, // Red
      { color: "#ff7f00", position: 17 }, // Orange
      { color: "#ffff00", position: 33 }, // Yellow
      { color: "#00ff00", position: 50 }, // Green
      { color: "#0000ff", position: 67 }, // Blue
      { color: "#4b0082", position: 83 }, // Indigo
      { color: "#9400d3", position: 100 }, // Violet
    ],
    steps
  );
}

/**
 * Generate a cool gradient (blue -> cyan -> white)
 */
export function coolGradient(steps: number = 100): string[] {
  return multiStopGradient(
    [
      { color: "#0000ff", position: 0 },
      { color: "#00ffff", position: 50 },
      { color: "#ffffff", position: 100 },
    ],
    steps
  );
}

/**
 * Generate a warm gradient (red -> orange -> yellow)
 */
export function warmGradient(steps: number = 100): string[] {
  return multiStopGradient(
    [
      { color: "#ff0000", position: 0 },
      { color: "#ff8800", position: 50 },
      { color: "#ffff00", position: 100 },
    ],
    steps
  );
}

/**
 * Generate a monochromatic gradient (light -> dark)
 */
export function monochromeGradient(baseColor: ColorInput, steps: number = 10): string[] {
  const base = color(baseColor);
  const light = base.lighten(0.3).toHex();
  const dark = base.darken(0.3).toHex();

  return gradient(light, dark, steps);
}

// ============================================================================
// Easing Functions
// ============================================================================

/**
 * Apply easing function to value
 */
function applyEasing(t: number, easing: GradientOptions["easing"]): number {
  switch (easing) {
    case "ease-in":
      return t * t;
    case "ease-out":
      return t * (2 - t);
    case "ease-in-out":
      return t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
    case "linear":
    default:
      return t;
  }
}

// ============================================================================
// Gradient Utilities
// ============================================================================

/**
 * Get color at specific position in gradient
 */
export function gradientAt(
  from: ColorInput,
  to: ColorInput,
  position: number // 0-1
): string {
  const normalizedPos = Math.max(0, Math.min(1, position));
  return color(from).mix(to, normalizedPos).toHex();
}

/**
 * Create gradient that passes through middle color
 */
export function gradientThrough(
  from: ColorInput,
  through: ColorInput,
  to: ColorInput,
  steps: number = 10
): string[] {
  const halfSteps = Math.ceil(steps / 2);

  const firstHalf = gradient(from, through, halfSteps);
  const secondHalf = gradient(through, to, steps - halfSteps + 1).slice(1);

  return [...firstHalf, ...secondHalf];
}
