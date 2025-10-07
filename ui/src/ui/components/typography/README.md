# Advanced Typography System

This typography system provides advanced text rendering capabilities using OpenType.js for font manipulation and custom effects.

## Components

### 1. CustomText
Renders text using OpenType.js with full control over typography features.

```tsx
import { CustomText } from '@/ui/components/typography';

<CustomText
  text="Beautiful Typography"
  preset="hero"
  gradient={{
    colors: ['#8B5CF6', '#EC4899', '#F59E0B'],
    angle: 45
  }}
  animate="fadeIn"
  shadow={{
    color: 'rgba(139, 92, 246, 0.3)',
    blur: 20,
    offsetY: 4
  }}
/>
```

**Props:**
- `text`: The text to render
- `fontName`: Font to use (must be loaded first)
- `preset`: Typography preset (hero, title, heading, body, caption, display)
- `fontSize`: Custom font size
- `letterSpacing`: Letter spacing adjustment
- `tracking`: Additional character spacing
- `gradient`: Gradient configuration
- `shadow`: Shadow configuration
- `glow`: Glow effect configuration
- `animate`: Animation type (fadeIn, slideUp, draw, morph, glitch, none)
- `animationDuration`: Animation duration in seconds

**Presets:**
- `hero`: 96px, tight spacing, all features
- `display`: 128px, extra tight spacing, decorative features
- `title`: 48px, tight spacing
- `heading`: 32px, medium spacing
- `body`: 16px, normal spacing
- `caption`: 14px, loose spacing

### 2. AnimatedTitle
High-level animated title component with preset effects.

```tsx
import { AnimatedTitle } from '@/ui/components/typography';

<AnimatedTitle
  text="Welcome to AgentOS"
  preset="hero"
  effect="gradient"
  animationDelay={200}
/>
```

**Effects:**
- `gradient`: Smooth gradient animation
- `glow`: Glowing pulsing effect
- `shimmer`: Shimmer/shine effect
- `wave`: Wave animation on characters
- `split`: Character-by-character reveal
- `typewriter`: Typewriter effect

### 3. BrandLogo
Sophisticated branded logo component with gradient text.

```tsx
import { BrandLogo } from '@/ui/components/typography';

<BrandLogo
  size="medium"
  animated={true}
  onClick={() => console.log('Logo clicked')}
/>
```

**Sizes:**
- `small`: Compact for menubar
- `medium`: Standard size
- `large`: Hero/landing page

### 4. WindowTitleBar
Animated window title with character-by-character reveal.

```tsx
import { WindowTitleBar } from '@/ui/components/typography';

<WindowTitleBar
  title="My Application"
  icon="📦"
  active={true}
/>
```

### 5. TypewriterText
Classic typewriter effect.

```tsx
import { TypewriterText } from '@/ui/components/typography';

<TypewriterText
  text="Type this out character by character"
  speed={50}
  delay={500}
  cursor={true}
  onComplete={() => console.log('Done!')}
/>
```

## Typography Utilities

### Loading Fonts

```ts
import { typography } from '@/core/utils/typography';

// Load a custom font
await typography.loadFont('MyFont', '/fonts/myfont.ttf');

// Generate SVG path from text
const pathData = typography.textToPath('Hello', 'MyFont', {
  fontSize: 72,
  letterSpacing: -1,
  features: { liga: true, kern: true }
});

// Measure text
const dimensions = typography.measureText('Hello', 'MyFont', 72);
console.log(dimensions.width, dimensions.height);

// Get glyph information
const glyphInfo = typography.getGlyphInfo('A', 'MyFont');

// Get available OpenType features
const features = typography.getFontFeatures('MyFont');
```

### Using the Hook

```tsx
import { useTypography } from '@/hooks/useTypography';

function MyComponent() {
  const { fontsLoaded, loadingError } = useTypography({
    fonts: [
      { name: 'CustomFont', url: '/fonts/custom.ttf' }
    ],
    preloadFonts: true
  });

  if (!fontsLoaded) return <div>Loading fonts...</div>;
  
  return <CustomText text="Hello" fontName="CustomFont" />;
}
```

## OpenType Features

The system supports advanced OpenType features:

- **liga**: Ligatures (e.g., "fi" → "ﬁ")
- **kern**: Kerning (spacing between character pairs)
- **calt**: Contextual alternates
- **swsh**: Swashes (decorative flourishes)
- **salt**: Stylistic alternates
- **dlig**: Discretionary ligatures
- **frac**: Fractions
- **ordn**: Ordinals

Enable features:

```tsx
<CustomText
  text="Official Typography"
  features={{
    liga: true,
    kern: true,
    swsh: true,
    salt: true
  }}
/>
```

## Visual Effects

### Gradients

```tsx
<CustomText
  text="Gradient Text"
  gradient={{
    colors: ['#8B5CF6', '#EC4899', '#EF4444'],
    angle: 45
  }}
/>
```

### Shadows

```tsx
<CustomText
  text="Shadow Text"
  shadow={{
    color: 'rgba(0, 0, 0, 0.5)',
    blur: 10,
    offsetX: 2,
    offsetY: 2
  }}
/>
```

### Glow

```tsx
<CustomText
  text="Glowing Text"
  glow={{
    color: '#8B5CF6',
    intensity: 6
  }}
/>
```

### Stroke

```tsx
<CustomText
  text="Outlined Text"
  fill="transparent"
  stroke="#8B5CF6"
  strokeWidth={2}
/>
```

## Animations

All animations use GPU-accelerated transforms for smooth 60fps performance.

```tsx
<CustomText
  text="Animated"
  animate="fadeIn"
  animationDuration={0.8}
/>
```

Available animations:
- `fadeIn`: Fade in with slight scale
- `slideUp`: Slide up from below
- `draw`: Draw stroke (requires stroke)
- `glitch`: Glitch effect
- `morph`: Morph between shapes

## Strategic Integration Points

The typography system is integrated at these key locations:

1. **Welcome Screen**: Hero title with gradient animation
2. **Desktop Menubar**: Branded logo with animated gradient
3. **Creator Overlay**: Glowing animated title
4. **Window Titles**: Character-by-character reveal
5. **App Content**: Available for blueprint-rendered apps

## Performance

- Font loading is cached and managed efficiently
- SVG paths are memoized to prevent unnecessary recalculation
- Animations use CSS transforms for hardware acceleration
- Components gracefully fall back to regular text while fonts load

## Browser Support

- Modern browsers with SVG support
- Chrome 90+, Firefox 88+, Safari 14+, Edge 90+
- Fallback to regular text on older browsers
