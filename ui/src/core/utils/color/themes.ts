/**
 * Dynamic Theme Generation Utilities
 * Create complete theme systems from base colors
 * Now with OKLCH support for perceptually uniform colors
 */

import type { ColorInput } from "./core";
import { color, lighten, darken, withAlpha } from "./core";
import { bestTextColor, ensureContrast } from "./contrast";
import { triadic } from "./palettes";
import { generateOklchScale, toOklchString, supportsOklch } from "./oklch";

// ============================================================================
// Types
// ============================================================================

export interface ColorScale {
  50: string;
  100: string;
  200: string;
  300: string;
  400: string;
  500: string;
  600: string;
  700: string;
  800: string;
  900: string;
  950: string;
}

export interface SemanticColors {
  primary: ColorScale;
  secondary: ColorScale;
  accent: ColorScale;
  success: ColorScale;
  warning: ColorScale;
  error: ColorScale;
  info: ColorScale;
  neutral: ColorScale;
}

export interface ThemeColors {
  background: string;
  foreground: string;
  card: string;
  cardForeground: string;
  popover: string;
  popoverForeground: string;
  primary: string;
  primaryForeground: string;
  secondary: string;
  secondaryForeground: string;
  muted: string;
  mutedForeground: string;
  accent: string;
  accentForeground: string;
  destructive: string;
  destructiveForeground: string;
  border: string;
  input: string;
  ring: string;
}

export interface Theme {
  colors: ThemeColors;
  semantic: SemanticColors;
  mode: "light" | "dark";
}

// ============================================================================
// Color Scale Generation
// ============================================================================

/**
 * Generate complete color scale from base color
 * Creates 11 shades (50-950) similar to Tailwind
 * Uses OKLCH for perceptually uniform color progression
 */
export function generateScale(baseColor: ColorInput): ColorScale {
  // Use OKLCH scale generation for better perceptual uniformity
  if (supportsOklch()) {
    const oklchScale = generateOklchScale(baseColor);
    return {
      50: oklchScale[50],
      100: oklchScale[100],
      200: oklchScale[200],
      300: oklchScale[300],
      400: oklchScale[400],
      500: oklchScale[500],
      600: oklchScale[600],
      700: oklchScale[700],
      800: oklchScale[800],
      900: oklchScale[900],
      950: oklchScale[950],
    };
  }

  // Fallback to hex for browsers without OKLCH support
  const base = color(baseColor);
  return {
    50: base.mix("#ffffff", 0.95).toHex(),
    100: base.mix("#ffffff", 0.9).toHex(),
    200: base.mix("#ffffff", 0.75).toHex(),
    300: base.mix("#ffffff", 0.6).toHex(),
    400: base.mix("#ffffff", 0.35).toHex(),
    500: base.toHex(),
    600: darken(base, 0.1),
    700: darken(base, 0.2),
    800: darken(base, 0.3),
    900: darken(base, 0.4),
    950: darken(base, 0.5),
  };
}

/**
 * Generate neutral (gray) scale
 */
export function generateNeutralScale(isDark: boolean = true): ColorScale {
  const base = isDark ? "#666666" : "#888888";
  return generateScale(base);
}

// ============================================================================
// Semantic Color Generation
// ============================================================================

/**
 * Generate all semantic color scales
 */
export function generateSemanticColors(primaryColor: ColorInput): SemanticColors {
  const harmonies = triadic(primaryColor);

  return {
    primary: generateScale(primaryColor),
    secondary: generateScale(harmonies[1]),
    accent: generateScale(harmonies[2]),
    success: generateScale("#10b981"),
    warning: generateScale("#f59e0b"),
    error: generateScale("#ef4444"),
    info: generateScale("#3b82f6"),
    neutral: generateNeutralScale(),
  };
}

// ============================================================================
// Dark Theme Generation
// ============================================================================

/**
 * Generate complete dark theme
 */
export function generateDarkTheme(primaryColor: ColorInput): Theme {
  const semantic = generateSemanticColors(primaryColor);
  const primary = semantic.primary;

  const background = "#0a0a0a";
  const foreground = "#ffffff";

  return {
    mode: "dark",
    semantic,
    colors: {
      background,
      foreground,
      card: "#1a1a1a",
      cardForeground: foreground,
      popover: "#1a1a1a",
      popoverForeground: foreground,
      primary: primary[500],
      primaryForeground: bestTextColor(primary[500]),
      secondary: "#2a2a2a",
      secondaryForeground: foreground,
      muted: "#2a2a2a",
      mutedForeground: withAlpha(foreground, 0.6),
      accent: semantic.accent[500],
      accentForeground: bestTextColor(semantic.accent[500]),
      destructive: semantic.error[500],
      destructiveForeground: bestTextColor(semantic.error[500]),
      border: "#2a2a2a",
      input: "#2a2a2a",
      ring: primary[500],
    },
  };
}

// ============================================================================
// Light Theme Generation
// ============================================================================

/**
 * Generate complete light theme
 */
export function generateLightTheme(primaryColor: ColorInput): Theme {
  const semantic = generateSemanticColors(primaryColor);
  const primary = semantic.primary;

  const background = "#ffffff";
  const foreground = "#0a0a0a";

  return {
    mode: "light",
    semantic,
    colors: {
      background,
      foreground,
      card: "#fafafa",
      cardForeground: foreground,
      popover: "#ffffff",
      popoverForeground: foreground,
      primary: primary[500],
      primaryForeground: bestTextColor(primary[500]),
      secondary: "#f5f5f5",
      secondaryForeground: foreground,
      muted: "#f5f5f5",
      mutedForeground: withAlpha(foreground, 0.6),
      accent: semantic.accent[500],
      accentForeground: bestTextColor(semantic.accent[500]),
      destructive: semantic.error[600],
      destructiveForeground: bestTextColor(semantic.error[600]),
      border: "#e5e5e5",
      input: "#e5e5e5",
      ring: primary[500],
    },
  };
}

// ============================================================================
// Theme Variants
// ============================================================================

/**
 * Generate theme with custom background
 */
export function generateThemeWithBackground(
  primaryColor: ColorInput,
  backgroundColor: ColorInput,
  mode: "light" | "dark" = "dark"
): Theme {
  const baseTheme =
    mode === "dark" ? generateDarkTheme(primaryColor) : generateLightTheme(primaryColor);

  const bg = color(backgroundColor).toHex();
  const fg = bestTextColor(bg);

  return {
    ...baseTheme,
    colors: {
      ...baseTheme.colors,
      background: bg,
      foreground: fg,
      card: mode === "dark" ? lighten(bg, 0.05) : darken(bg, 0.02),
      cardForeground: fg,
      popover: mode === "dark" ? lighten(bg, 0.08) : darken(bg, 0.03),
      popoverForeground: fg,
    },
  };
}

/**
 * Generate glassmorphism theme
 */
export function generateGlassTheme(primaryColor: ColorInput): Theme {
  const baseTheme = generateDarkTheme(primaryColor);

  return {
    ...baseTheme,
    colors: {
      ...baseTheme.colors,
      background: withAlpha("#0a0a0a", 0.8),
      card: withAlpha("#1a1a1a", 0.6),
      popover: withAlpha("#1a1a1a", 0.7),
      secondary: withAlpha("#2a2a2a", 0.5),
    },
  };
}

// ============================================================================
// CSS Variable Generation
// ============================================================================

/**
 * Convert theme to CSS custom properties
 * Converts colors to OKLCH format when supported for better color quality
 */
export function themeToCssVars(theme: Theme, prefix: string = "--color"): Record<string, string> {
  const vars: Record<string, string> = {};
  const useOklch = supportsOklch();

  // Helper to convert color to appropriate format
  const formatColor = (colorValue: string): string => {
    // Skip if already in oklch format
    if (colorValue.startsWith("oklch(")) {
      return colorValue;
    }
    // Convert to OKLCH if supported, otherwise keep as-is
    return useOklch ? toOklchString(colorValue) : colorValue;
  };

  // Main colors
  Object.entries(theme.colors).forEach(([key, value]) => {
    const kebabKey = key.replace(/([A-Z])/g, "-$1").toLowerCase();
    vars[`${prefix}-${kebabKey}`] = formatColor(value);
  });

  // Semantic scales
  Object.entries(theme.semantic).forEach(([colorName, scale]) => {
    Object.entries(scale).forEach(([shade, value]) => {
      vars[`${prefix}-${colorName}-${shade}`] = formatColor(String(value));
    });
  });

  return vars;
}

/**
 * Generate CSS string from theme
 */
export function themeToCss(theme: Theme, selector: string = ":root"): string {
  const vars = themeToCssVars(theme);

  const cssVars = Object.entries(vars)
    .map(([key, value]) => `  ${key}: ${value};`)
    .join("\n");

  return `${selector} {\n${cssVars}\n}`;
}

// ============================================================================
// Theme Utilities
// ============================================================================

/**
 * Ensure all theme colors meet accessibility standards
 */
export function ensureAccessibleTheme(theme: Theme): Theme {
  const { colors } = theme;

  return {
    ...theme,
    colors: {
      ...colors,
      primaryForeground: ensureContrast(colors.primaryForeground, colors.primary, "AA"),
      secondaryForeground: ensureContrast(colors.secondaryForeground, colors.secondary, "AA"),
      cardForeground: ensureContrast(colors.cardForeground, colors.card, "AA"),
      popoverForeground: ensureContrast(colors.popoverForeground, colors.popover, "AA"),
      mutedForeground: ensureContrast(colors.mutedForeground, colors.muted, "AA"),
      accentForeground: ensureContrast(colors.accentForeground, colors.accent, "AA"),
      destructiveForeground: ensureContrast(colors.destructiveForeground, colors.destructive, "AA"),
    },
  };
}

/**
 * Blend two themes together
 */
export function blendThemes(theme1: Theme, theme2: Theme, ratio: number = 0.5): Theme {
  const blendColor = (c1: string, c2: string): string => color(c1).mix(c2, ratio).toHex();

  return {
    mode: ratio < 0.5 ? theme1.mode : theme2.mode,
    semantic: theme1.semantic, // Keep semantic from first theme
    colors: {
      background: blendColor(theme1.colors.background, theme2.colors.background),
      foreground: blendColor(theme1.colors.foreground, theme2.colors.foreground),
      card: blendColor(theme1.colors.card, theme2.colors.card),
      cardForeground: blendColor(theme1.colors.cardForeground, theme2.colors.cardForeground),
      popover: blendColor(theme1.colors.popover, theme2.colors.popover),
      popoverForeground: blendColor(
        theme1.colors.popoverForeground,
        theme2.colors.popoverForeground
      ),
      primary: blendColor(theme1.colors.primary, theme2.colors.primary),
      primaryForeground: blendColor(
        theme1.colors.primaryForeground,
        theme2.colors.primaryForeground
      ),
      secondary: blendColor(theme1.colors.secondary, theme2.colors.secondary),
      secondaryForeground: blendColor(
        theme1.colors.secondaryForeground,
        theme2.colors.secondaryForeground
      ),
      muted: blendColor(theme1.colors.muted, theme2.colors.muted),
      mutedForeground: blendColor(theme1.colors.mutedForeground, theme2.colors.mutedForeground),
      accent: blendColor(theme1.colors.accent, theme2.colors.accent),
      accentForeground: blendColor(theme1.colors.accentForeground, theme2.colors.accentForeground),
      destructive: blendColor(theme1.colors.destructive, theme2.colors.destructive),
      destructiveForeground: blendColor(
        theme1.colors.destructiveForeground,
        theme2.colors.destructiveForeground
      ),
      border: blendColor(theme1.colors.border, theme2.colors.border),
      input: blendColor(theme1.colors.input, theme2.colors.input),
      ring: blendColor(theme1.colors.ring, theme2.colors.ring),
    },
  };
}
