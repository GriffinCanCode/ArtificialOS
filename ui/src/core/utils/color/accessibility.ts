/**
 * Accessibility Helpers
 * Easy-to-use accessibility checking for UI components
 */

import { contrast, isWcagAA, ensureContrast, bestTextColor, type WCAGLevel } from "./contrast";
import type { ColorInput } from "./core";

// ============================================================================
// Quick Accessibility Checks
// ============================================================================

/**
 * Check if text color is readable on background
 * Returns true if passes WCAG AA standards
 */
export function isReadableText(textColor: ColorInput, backgroundColor: ColorInput): boolean {
  return isWcagAA(textColor, backgroundColor);
}

/**
 * Get safe text color for any background
 * Returns either black or white, whichever has better contrast
 */
export function getSafeTextColor(backgroundColor: ColorInput): string {
  return bestTextColor(backgroundColor);
}

/**
 * Ensure text meets minimum contrast standards
 * Automatically adjusts color if needed
 */
export function ensureReadableText(
  textColor: ColorInput,
  backgroundColor: ColorInput,
  level: WCAGLevel = "AA"
): string {
  return ensureContrast(textColor, backgroundColor, level);
}

/**
 * Get contrast ratio between two colors
 * Returns number between 1 and 21
 */
export function getContrastRatio(color1: ColorInput, color2: ColorInput): number {
  return contrast(color1, color2);
}

/**
 * Check if color combination passes WCAG standards
 * Returns detailed accessibility information
 */
export function checkAccessibility(
  foreground: ColorInput,
  background: ColorInput
): {
  passes: boolean;
  ratio: number;
  recommendation: string;
} {
  const ratio = contrast(foreground, background);
  const passes = isWcagAA(foreground, background);

  let recommendation = "";
  if (ratio >= 7) {
    recommendation = "Excellent - passes AAA";
  } else if (ratio >= 4.5) {
    recommendation = "Good - passes AA";
  } else if (ratio >= 3) {
    recommendation = "Only for large text";
  } else {
    recommendation = "Fails - adjust colors";
  }

  return { passes, ratio: Math.round(ratio * 100) / 100, recommendation };
}

// ============================================================================
// UI Component Helpers
// ============================================================================

/**
 * Validate button colors for accessibility
 */
export function validateButtonColors(
  buttonBg: ColorInput,
  buttonText: ColorInput
): {
  isValid: boolean;
  suggestedText?: string;
} {
  const isValid = isWcagAA(buttonText, buttonBg);

  if (isValid) {
    return { isValid: true };
  }

  return {
    isValid: false,
    suggestedText: ensureContrast(buttonText, buttonBg, "AA"),
  };
}

/**
 * Validate link colors against background
 */
export function validateLinkColor(
  linkColor: ColorInput,
  backgroundColor: ColorInput
): {
  isValid: boolean;
  suggestedColor?: string;
} {
  const isValid = isWcagAA(linkColor, backgroundColor);

  if (isValid) {
    return { isValid: true };
  }

  return {
    isValid: false,
    suggestedColor: ensureContrast(linkColor, backgroundColor, "AA"),
  };
}
