/**
 * CSS Variable Generation Utilities
 * Generate and apply CSS custom properties from color themes
 * Now with OKLCH support for better color quality
 */

import { generateDarkTheme, generateLightTheme, themeToCssVars, type Theme } from "./themes";
import { CHART_COLORS } from "../../../features/visualization/utils/colors";
import { toOklchString, supportsOklch } from "./oklch";

// ============================================================================
// CSS Variable Generation
// ============================================================================

/**
 * Generate CSS variables object from theme colors
 */
export function generateCSSVariables(primaryColor: string, mode: "light" | "dark" = "dark"): Record<string, string> {
  const theme = mode === "dark" ? generateDarkTheme(primaryColor) : generateLightTheme(primaryColor);
  return themeToCssVars(theme);
}

/**
 * Apply theme as CSS variables to the document root
 */
export function applyThemeVariables(primaryColor: string, mode: "light" | "dark" = "dark"): void {
  const variables = generateCSSVariables(primaryColor, mode);

  Object.entries(variables).forEach(([key, value]) => {
    document.documentElement.style.setProperty(key, value);
  });
}

/**
 * Remove all theme CSS variables from document root
 */
export function clearThemeVariables(): void {
  const styles = document.documentElement.style;

  // Remove all --color-* variables
  for (let i = styles.length - 1; i >= 0; i--) {
    const prop = styles[i];
    if (prop.startsWith("--color-")) {
      document.documentElement.style.removeProperty(prop);
    }
  }
}

/**
 * Generate CSS variables for visualization colors
 * Converts to OKLCH when supported for better color accuracy
 */
export function generateVisualizationVariables(): Record<string, string> {
  const vars: Record<string, string> = {};
  const useOklch = supportsOklch();

  // Helper to format color
  const formatColor = (color: string): string => {
    return useOklch ? toOklchString(color) : color;
  };

  // Chart primary colors
  CHART_COLORS.primary.forEach((color, index) => {
    vars[`--chart-primary-${index + 1}`] = formatColor(color);
  });

  // Gradient colors
  CHART_COLORS.gradient.forEach((color, index) => {
    vars[`--chart-gradient-${index + 1}`] = formatColor(color);
  });

  // Sequential colors
  CHART_COLORS.sequential.forEach((color, index) => {
    vars[`--chart-sequential-${index + 1}`] = formatColor(color);
  });

  // Semantic colors
  vars["--chart-success"] = formatColor(CHART_COLORS.semantic.success);
  vars["--chart-warning"] = formatColor(CHART_COLORS.semantic.warning);
  vars["--chart-error"] = formatColor(CHART_COLORS.semantic.error);
  vars["--chart-info"] = formatColor(CHART_COLORS.semantic.info);
  vars["--chart-neutral"] = formatColor(CHART_COLORS.semantic.neutral);

  return vars;
}

/**
 * Apply visualization variables to document root
 */
export function applyVisualizationVariables(): void {
  const variables = generateVisualizationVariables();

  Object.entries(variables).forEach(([key, value]) => {
    document.documentElement.style.setProperty(key, value);
  });
}

/**
 * Generate complete CSS string with all variables
 */
export function generateThemeCSS(primaryColor: string, mode: "light" | "dark" = "dark"): string {
  const themeVars = generateCSSVariables(primaryColor, mode);
  const vizVars = generateVisualizationVariables();

  const allVars = { ...themeVars, ...vizVars };

  const cssVars = Object.entries(allVars)
    .map(([key, value]) => `  ${key}: ${value};`)
    .join("\n");

  return `:root {\n${cssVars}\n}`;
}

/**
 * Download theme CSS as a file
 */
export function downloadThemeCSS(primaryColor: string, mode: "light" | "dark" = "dark", filename: string = "theme.css"): void {
  const css = generateThemeCSS(primaryColor, mode);
  const blob = new Blob([css], { type: "text/css" });
  const url = URL.createObjectURL(blob);

  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

// ============================================================================
// Reactive Theme System
// ============================================================================

export interface ThemeConfig {
  primaryColor: string;
  mode: "light" | "dark";
  autoApply?: boolean;
}

export class ThemeManager {
  private config: ThemeConfig;
  private listeners: Set<(theme: Theme) => void> = new Set();

  constructor(config: ThemeConfig) {
    this.config = config;

    if (config.autoApply !== false) {
      this.apply();
    }
  }

  /**
   * Update theme configuration
   */
  setConfig(config: Partial<ThemeConfig>): void {
    this.config = { ...this.config, ...config };
    this.apply();
    this.notifyListeners();
  }

  /**
   * Get current theme object
   */
  getTheme(): Theme {
    return this.config.mode === "dark"
      ? generateDarkTheme(this.config.primaryColor)
      : generateLightTheme(this.config.primaryColor);
  }

  /**
   * Apply current theme to document
   */
  apply(): void {
    applyThemeVariables(this.config.primaryColor, this.config.mode);
    applyVisualizationVariables();
  }

  /**
   * Toggle between light and dark mode
   */
  toggleMode(): void {
    this.setConfig({ mode: this.config.mode === "dark" ? "light" : "dark" });
  }

  /**
   * Subscribe to theme changes
   */
  subscribe(listener: (theme: Theme) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /**
   * Notify all listeners of theme change
   */
  private notifyListeners(): void {
    const theme = this.getTheme();
    this.listeners.forEach(listener => listener(theme));
  }

  /**
   * Clear all theme variables
   */
  clear(): void {
    clearThemeVariables();
  }
}

/**
 * Create a theme manager instance
 */
export function createThemeManager(config: ThemeConfig): ThemeManager {
  return new ThemeManager(config);
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Get CSS variable value from document
 */
export function getCSSVariable(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

/**
 * Set CSS variable on document root
 */
export function setCSSVariable(name: string, value: string): void {
  document.documentElement.style.setProperty(name, value);
}

/**
 * Check if CSS variables are supported
 */
export function supportsCSSVariables(): boolean {
  return CSS?.supports?.("(--test: 0)") ?? false;
}
