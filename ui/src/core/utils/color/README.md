# Color Utilities

Comprehensive color manipulation system powered by [colord](https://github.com/omgovich/colord).

## Features

- **Tiny Bundle**: 1.7KB gzipped
- **TypeScript Native**: Full type support with strict typing
- **Immutable**: All operations return new values
- **WCAG Compliance**: Built-in accessibility checking
- **Color Harmonies**: Generate complementary, triadic, analogous schemes
- **Dynamic Themes**: Complete theme generation from base colors
- **Advanced Gradients**: Multi-stop gradients with easing
- **Format Support**: hex, rgb, rgba, hsl, hsla, hsv, named colors

## Quick Start

```typescript
import { color, lighten, darken, contrast, gradient } from "@/core/utils/color";

// Basic color manipulation
const primary = "#667eea";
const lightened = lighten(primary, 0.2);
const darkened = darken(primary, 0.3);

// Check accessibility
const contrastRatio = contrast("#ffffff", "#667eea");
const isReadable = isWcagAA("#ffffff", "#667eea");

// Generate gradients
const grad = gradient("#667eea", "#764ba2", 10);
```

## Core Utilities

### Color Creation & Validation

```typescript
import { color, isValid, parse } from "@/core/utils/color";

// Create color from any format
color("#667eea");
color("rgb(102, 126, 234)");
color({ r: 102, g: 126, b: 234 });
color("blue");

// Validate colors
isValid("#667eea"); // true
isValid("not-a-color"); // false

// Parse with fallback
parse("invalid", "#000000");
```

### Color Conversion

```typescript
import { toHex, toRgb, toHsl, toRgbaString } from "@/core/utils/color";

const color = "#667eea";

toHex(color); // "#667eea"
toRgb(color); // { r: 102, g: 126, b: 234 }
toHsl(color); // { h: 229, s: 76, l: 66 }
toRgbaString(color, 0.5); // "rgba(102, 126, 234, 0.5)"
```

### Color Manipulation

```typescript
import {
  lighten,
  darken,
  saturate,
  desaturate,
  withAlpha,
  mix,
  rotate,
} from "@/core/utils/color";

// Adjust lightness
lighten("#667eea", 0.2); // Lighter by 20%
darken("#667eea", 0.3); // Darker by 30%

// Adjust saturation
saturate("#667eea", 0.2);
desaturate("#667eea", 0.3);

// Set opacity
withAlpha("#667eea", 0.5);

// Mix colors
mix("#667eea", "#764ba2", 0.5);

// Rotate hue
rotate("#667eea", 30); // Rotate 30 degrees
```

## Accessibility

### WCAG Compliance

```typescript
import {
  contrast,
  isWcagAA,
  isWcagAAA,
  accessibility,
} from "@/core/utils/color";

// Check contrast ratio (1-21)
contrast("#ffffff", "#000000"); // 21 (maximum)
contrast("#667eea", "#ffffff"); // ~2.5

// WCAG AA: 4.5:1 normal, 3:1 large
isWcagAA("#ffffff", "#667eea"); // false
isWcagAA("#ffffff", "#667eea", "large"); // true

// WCAG AAA: 7:1 normal, 4.5:1 large
isWcagAAA("#ffffff", "#000000"); // true

// Comprehensive assessment
accessibility("#ffffff", "#667eea");
// {
//   isReadable: false,
//   contrast: 2.5,
//   wcagAA: false,
//   wcagAAA: false,
//   recommendation: "Poor contrast - fails WCAG standards"
// }
```

### Automatic Contrast Adjustment

```typescript
import { ensureContrast, bestTextColor, optimalTextColor } from "@/core/utils/color";

// Adjust color to meet WCAG standards
ensureContrast("#888888", "#ffffff", "AA"); // Returns readable color

// Get best text color (black or white)
bestTextColor("#667eea"); // "#ffffff"
bestTextColor("#ffff00"); // "#000000"

// Get optimal readable color
optimalTextColor("#667eea", "#ff0000", "AA");
```

### Color Blindness Simulation

```typescript
import {
  simulateProtanopia,
  simulateDeuteranopia,
  simulateTritanopia,
} from "@/core/utils/color";

simulateProtanopia("#667eea"); // Red-blind
simulateDeuteranopia("#667eea"); // Green-blind
simulateTritanopia("#667eea"); // Blue-blind
```

## Gradients

### Basic Gradients

```typescript
import { gradient, smoothGradient, gradientAt } from "@/core/utils/color";

// Linear gradient
gradient("#667eea", "#764ba2", 10);

// Smooth easing
smoothGradient("#667eea", "#764ba2", 10);

// Get color at position
gradientAt("#667eea", "#764ba2", 0.5);
```

### Multi-Stop Gradients

```typescript
import { multiStopGradient, cssMultiStopGradient } from "@/core/utils/color";

const stops = [
  { color: "#667eea", position: 0 },
  { color: "#764ba2", position: 50 },
  { color: "#f093fb", position: 100 },
];

// Generate color array
multiStopGradient(stops, 100);

// Generate CSS
cssMultiStopGradient(stops, "to-right");
// "linear-gradient(to right, #667eea 0%, #764ba2 50%, #f093fb 100%)"
```

### CSS Gradients

```typescript
import {
  cssLinearGradient,
  cssRadialGradient,
  cssConicGradient,
} from "@/core/utils/color";

cssLinearGradient("#667eea", "#764ba2", "to-right");
cssRadialGradient("#667eea", "#764ba2", "circle");
cssConicGradient(["#667eea", "#764ba2", "#f093fb"], 0);
```

### Specialized Gradients

```typescript
import {
  heatmapGradient,
  rainbowGradient,
  coolGradient,
  warmGradient,
} from "@/core/utils/color";

heatmapGradient(100); // Blue -> Green -> Yellow -> Red
rainbowGradient(100); // Full spectrum
coolGradient(100); // Blue tones
warmGradient(100); // Warm tones
```

## Color Palettes

### Color Harmonies

```typescript
import {
  analogous,
  complementary,
  triadic,
  splitComplementary,
  harmony,
} from "@/core/utils/color";

// Adjacent colors (harmonious)
analogous("#667eea", 3);

// Opposite color (high contrast)
complementary("#667eea");

// Three evenly spaced colors
triadic("#667eea");

// Split complementary
splitComplementary("#667eea");

// Generic harmony
harmony("#667eea", "triadic");
```

### Palette Generation

```typescript
import { monochrome, tints, shades, tones, fullPalette } from "@/core/utils/color";

// Monochromatic variations
monochrome("#667eea", { count: 5, variation: "mixed" });

// Tints (mix with white)
tints("#667eea", 5);

// Shades (mix with black)
shades("#667eea", 5);

// Tones (mix with gray)
tones("#667eea", 5);

// Complete palette (tints + base + shades)
fullPalette("#667eea", 9);
```

### Material Design Palette

```typescript
import { materialPalette } from "@/core/utils/color";

materialPalette("#667eea");
// {
//   50: "#...",
//   100: "#...",
//   200: "#...",
//   ...
//   900: "#..."
// }
```

### Categorical Colors

```typescript
import { categorical, sequential, diverging } from "@/core/utils/color";

// Distinct colors for categories
categorical(8); // 8 evenly distributed colors

// Sequential (ordered data)
sequential("#667eea", 9);

// Diverging (positive/negative)
diverging("#ef4444", "#ffffff", "#10b981", 11);
```

## Theme Generation

### Complete Theme

```typescript
import { generateDarkTheme, generateLightTheme } from "@/core/utils/color";

const darkTheme = generateDarkTheme("#667eea");
const lightTheme = generateLightTheme("#667eea");

// Theme includes:
// - colors: Complete color system
// - semantic: Semantic color scales (primary, secondary, etc.)
// - mode: "light" | "dark"
```

### Color Scales

```typescript
import { generateScale, generateSemanticColors } from "@/core/utils/color";

// Single color scale (50-950)
generateScale("#667eea");

// All semantic scales
generateSemanticColors("#667eea");
// {
//   primary: ColorScale,
//   secondary: ColorScale,
//   accent: ColorScale,
//   success: ColorScale,
//   ...
// }
```

### CSS Variables

```typescript
import { themeToCssVars, themeToCss } from "@/core/utils/color";

const theme = generateDarkTheme("#667eea");

// Get CSS variables object
const vars = themeToCssVars(theme);

// Get CSS string
const css = themeToCss(theme, ":root");
// :root {
//   --color-background: #0a0a0a;
//   --color-foreground: #ffffff;
//   ...
// }
```

### Theme Variants

```typescript
import {
  generateThemeWithBackground,
  generateGlassTheme,
  ensureAccessibleTheme,
  blendThemes,
} from "@/core/utils/color";

// Custom background
generateThemeWithBackground("#667eea", "#1a1a1a", "dark");

// Glassmorphism
generateGlassTheme("#667eea");

// Ensure accessibility
ensureAccessibleTheme(theme);

// Blend two themes
blendThemes(theme1, theme2, 0.5);
```

## Advanced Usage

### Color Class

```typescript
import { Color } from "@/core/utils/color";

const c = new Color("#667eea");

// Chainable operations
c.lighten(0.2).saturate(0.1).withAlpha(0.8).toHex();

// Properties
c.luminance(); // 0.3
c.brightness(); // 180
c.isLight(); // false
c.isDark(); // true

// Conversions
c.toHex();
c.toRgb();
c.toHsl();
c.toString("rgba");
```

## Best Practices

1. **Use semantic names**: Prefer `primary`, `success` over hex codes in UI
2. **Check accessibility**: Always verify text contrast meets WCAG AA
3. **Use color harmonies**: Generate palettes from harmonies for cohesive design
4. **Test color blindness**: Use simulation functions to verify accessibility
5. **Generate themes**: Use theme generation for consistent dark/light modes
6. **Prefer immutable operations**: All functions return new values

## Performance

- Bundle size: **1.7KB gzipped**
- Tree-shakeable: Import only what you need
- No dependencies: Pure JavaScript implementation
- Immutable: Predictable, cacheable operations

## Migration from Legacy

```typescript
// Old (custom implementation)
import { hexToRgb, rgbToHex, withOpacity } from "./old-utils";

// New (colord-powered)
import { toRgb, toHex, toRgbaString } from "@/core/utils/color";

// API is backward compatible
toRgb("#667eea"); // Same output
toHex({ r: 102, g: 126, b: 234 }); // Same output
toRgbaString("#667eea", 0.5); // Replaces withOpacity
```

## References

- [colord documentation](https://github.com/omgovich/colord)
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Color Theory Basics](https://www.interaction-design.org/literature/topics/color-theory)
