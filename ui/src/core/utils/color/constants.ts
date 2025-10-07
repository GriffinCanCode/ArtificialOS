/**
 * Theme Color Constants
 * Centralized color values for consistent access across the application
 */

// ============================================================================
// Primary Theme Colors
// ============================================================================

export const THEME_COLORS = {
  primary: "#667eea",
  secondary: "#764ba2",
  accent: "#f093fb",
  get brand() {
    return this.primary;
  }, // Alias for brand color
} as const;

// ============================================================================
// UI Colors
// ============================================================================

export const UI_COLORS = {
  background: {
    primary: "#0a0a0a",
    secondary: "#1a1a1a",
    tertiary: "#2a2a2a",
  },
  text: {
    primary: "#ffffff",
    secondary: "rgba(255, 255, 255, 0.7)",
    tertiary: "rgba(255, 255, 255, 0.5)",
    muted: "rgba(255, 255, 255, 0.4)",
  },
  border: {
    default: "#2a2a2a",
    subtle: "rgba(255, 255, 255, 0.06)",
    focus: "#667eea",
  },
} as const;

// ============================================================================
// Semantic Colors
// ============================================================================

export const SEMANTIC_COLORS = {
  success: "#43e97b",
  warning: "#fee140",
  error: "#fa709a",
  info: "#4facfe",
  neutral: "#a0aec0",
} as const;

// ============================================================================
// Alpha Values (for consistency)
// ============================================================================

export const ALPHA_VALUES = {
  high: 0.95,
  medium: 0.7,
  low: 0.5,
  subtle: 0.3,
  verySubtle: 0.15,
  ghost: 0.06,
} as const;

// ============================================================================
// Utility Type for Color Access
// ============================================================================

export type ThemeColor = typeof THEME_COLORS;
export type UIColor = typeof UI_COLORS;
export type SemanticColor = typeof SEMANTIC_COLORS;
