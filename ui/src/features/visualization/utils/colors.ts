/**
 * Color Utilities
 * Theme-aware color management for visualizations
 */

// ============================================================================
// Color Palettes
// ============================================================================

export const CHART_COLORS = {
  primary: [
    "#667eea", // Primary purple
    "#764ba2", // Deep purple
    "#f093fb", // Light purple
    "#4facfe", // Blue
    "#00f2fe", // Cyan
    "#43e97b", // Green
    "#38f9d7", // Teal
    "#fa709a", // Pink
    "#fee140", // Yellow
  ],
  gradient: [
    "#667eea",
    "#764ba2",
    "#f093fb",
    "#4facfe",
    "#00f2fe",
  ],
  sequential: [
    "#f7fafc",
    "#e2e8f0",
    "#cbd5e0",
    "#a0aec0",
    "#718096",
    "#4a5568",
    "#2d3748",
    "#1a202c",
  ],
  diverging: [
    "#4facfe",
    "#00f2fe",
    "#f7fafc",
    "#fee140",
    "#fa709a",
  ],
  categorical: [
    "#667eea", // Purple
    "#4facfe", // Blue
    "#43e97b", // Green
    "#fee140", // Yellow
    "#fa709a", // Pink
    "#764ba2", // Deep purple
    "#38f9d7", // Teal
    "#f093fb", // Light purple
  ],
  semantic: {
    success: "#43e97b",
    warning: "#fee140",
    error: "#fa709a",
    info: "#4facfe",
    neutral: "#a0aec0",
  },
};

// ============================================================================
// Color Functions
// ============================================================================

/**
 * Get color for chart series by index
 */
export function getSeriesColor(index: number, palette: keyof typeof CHART_COLORS = "primary"): string {
  const colors = Array.isArray(CHART_COLORS[palette])
    ? CHART_COLORS[palette] as string[]
    : CHART_COLORS.primary;
  return colors[index % colors.length];
}

/**
 * Get semantic color by type
 */
export function getSemanticColor(type: keyof typeof CHART_COLORS.semantic): string {
  return CHART_COLORS.semantic[type];
}

/**
 * Generate gradient from two colors
 */
export function generateGradient(from: string, to: string, stops = 10): string[] {
  // Simple linear interpolation for hex colors
  const fromRgb = hexToRgb(from);
  const toRgb = hexToRgb(to);

  const gradient: string[] = [];
  for (let i = 0; i < stops; i++) {
    const ratio = i / (stops - 1);
    const r = Math.round(fromRgb.r + (toRgb.r - fromRgb.r) * ratio);
    const g = Math.round(fromRgb.g + (toRgb.g - fromRgb.g) * ratio);
    const b = Math.round(fromRgb.b + (toRgb.b - fromRgb.b) * ratio);
    gradient.push(rgbToHex(r, g, b));
  }

  return gradient;
}

/**
 * Convert hex color to RGB
 */
export function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16),
      }
    : { r: 0, g: 0, b: 0 };
}

/**
 * Convert RGB to hex color
 */
export function rgbToHex(r: number, g: number, b: number): string {
  return "#" + [r, g, b].map((x) => {
    const hex = x.toString(16);
    return hex.length === 1 ? "0" + hex : hex;
  }).join("");
}

/**
 * Adjust color opacity
 */
export function withOpacity(color: string, opacity: number): string {
  const rgb = hexToRgb(color);
  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${opacity})`;
}

/**
 * Get color for value in range
 */
export function colorFromValue(
  value: number,
  min: number,
  max: number,
  palette: string[] = CHART_COLORS.sequential
): string {
  const normalized = Math.max(0, Math.min(1, (value - min) / (max - min)));
  const index = Math.floor(normalized * (palette.length - 1));
  return palette[index];
}
