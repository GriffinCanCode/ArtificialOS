# GSAP Animation System

## Overview

Your application now uses GSAP (GreenSock Animation Platform) for all animations, providing a centralized, performant, and consistent animation system.

## Architecture

### 1. **Animation Configuration** (`src/utils/animationConfig.ts`)
- **Centralized Timing Constants**: All animation durations in one place to prevent race conditions
- **Easing Presets**: Standard easing functions mapped to GSAP equivalents
- **Stagger Configurations**: For animating multiple elements in sequence
- **Component-Specific Timings**: Pre-configured timing for common components
- **Z-Index Layers**: Proper stacking order management

### 2. **Animation Utilities** (`src/utils/gsapAnimations.ts`)
Reusable animation functions:

**Basic Animations:**
- `fadeIn`, `fadeOut`
- `slideInUp`, `slideInRight`, `slideInDown`
- `scaleIn`, `scaleOut`

**Component-Specific:**
- `appAppear` - Rendered app container animation
- `buildContainerAppear` - Build preview container
- `componentAppear` - Individual component building
- `componentMaterialize` - Skeleton to real transition
- `errorShake` - Error notification shake
- `completionFlash` - Success completion flash

**Infinite Animations:**
- `floatAnimation` - Floating icon effect
- `pulseAnimation` - Pulsing opacity
- `spinAnimation` - Rotating spinner
- `borderGlowAnimation` - Border glow pulse
- `buildPulseAnimation` - Complex build indicator pulse
- `progressShimmerAnimation` - Progress bar shimmer
- `gridScanAnimation` - Grid background scan
- `assemblyPulseAnimation` - Component assembly indicator

**Stagger Animations:**
- `staggerFadeIn` - Fade in multiple elements
- `staggerSlideUp` - Slide up multiple elements

**Hover Animations:**
- `buttonHoverIn`, `buttonHoverOut`
- `scaleHoverIn`, `scaleHoverOut`

### 3. **React Hooks** (`src/hooks/useGSAP.ts`)
Easy-to-use React hooks for animations:

**Mount Animations:**
```typescript
const ref = useFadeIn<HTMLDivElement>({ duration: 0.5, delay: 0.2 });
const ref = useSlideInUp<HTMLDivElement>({ distance: 20 });
const ref = useScaleIn<HTMLDivElement>({ scale: 0.95 });
const ref = useAppAppear<HTMLDivElement>();
```

**Infinite Animations:**
```typescript
const ref = useFloat<HTMLDivElement>(enabled, { distance: 12, duration: 4 });
const ref = usePulse<HTMLDivElement>(enabled, { opacity: 0.7 });
const ref = useSpin<HTMLDivElement>(enabled);
const ref = useBuildPulse<HTMLDivElement>(enabled);
```

**Hover Animations:**
```typescript
const { elementRef, hoverProps } = useButtonHover<HTMLButtonElement>();
<button ref={elementRef} {...hoverProps}>Click Me</button>

const { elementRef, hoverProps } = useScaleHover<HTMLDivElement>(1.05);
```

**Imperative Animations:**
```typescript
const { elementRef, fadeIn, fadeOut, shake, flash, slideInUp, scaleIn, scaleOut } = 
  useImperativeAnimation<HTMLDivElement>();

// Trigger animations on demand
onClick={() => shake()}
```

**Stagger Animations:**
```typescript
const containerRef = useStaggerFadeIn<HTMLDivElement>('.child-class', { 
  stagger: 0.1, 
  duration: 0.4 
});

const containerRef = useStaggerSlideUp<HTMLDivElement>('> *', { 
  stagger: 0.15,
  distance: 20 
});
```

## Implementation Examples

### Example 1: Component with Mount Animation
```typescript
import { useFadeIn, useSlideInUp } from '../hooks/useGSAP';

function MyComponent() {
  const containerRef = useFadeIn<HTMLDivElement>({ duration: 0.5 });
  const titleRef = useSlideInUp<HTMLHeadingElement>({ distance: 30, delay: 0.2 });
  
  return (
    <div ref={containerRef}>
      <h1 ref={titleRef}>Hello World</h1>
    </div>
  );
}
```

### Example 2: Infinite Animation with Toggle
```typescript
import { useFloat, usePulse } from '../hooks/useGSAP';

function LoadingIcon({ isLoading }: { isLoading: boolean }) {
  const iconRef = useFloat<HTMLDivElement>(isLoading);
  const badgeRef = usePulse<HTMLSpanElement>(isLoading);
  
  return (
    <div ref={iconRef}>
      <Icon />
      {isLoading && <span ref={badgeRef}>Loading...</span>}
    </div>
  );
}
```

### Example 3: Hover Animation
```typescript
import { useButtonHover } from '../hooks/useGSAP';

function AnimatedButton() {
  const { elementRef, hoverProps } = useButtonHover<HTMLButtonElement>();
  
  return (
    <button ref={elementRef} {...hoverProps}>
      Hover Me
    </button>
  );
}
```

### Example 4: Imperative Animation (on demand)
```typescript
import { useImperativeAnimation } from '../hooks/useGSAP';

function ErrorMessage() {
  const { elementRef, shake, fadeOut } = useImperativeAnimation<HTMLDivElement>();
  
  useEffect(() => {
    if (error) {
      shake();
    }
  }, [error]);
  
  return (
    <div ref={elementRef}>
      Error occurred!
      <button onClick={() => fadeOut({ onComplete: () => clearError() })}>
        Dismiss
      </button>
    </div>
  );
}
```

### Example 5: Stagger Animation for List
```typescript
import { useStaggerSlideUp } from '../hooks/useGSAP';

function ItemList({ items }: { items: Item[] }) {
  const listRef = useStaggerSlideUp<HTMLUListElement>('.list-item', { 
    stagger: 0.1,
    duration: 0.4,
    distance: 20
  });
  
  return (
    <ul ref={listRef}>
      {items.map(item => (
        <li key={item.id} className="list-item">{item.name}</li>
      ))}
    </ul>
  );
}
```

## Components Updated

### ✅ DynamicRenderer
- Desktop icon float animation
- Error shake animation
- Build container appear animation
- Rendered app appear animation
- Progress bar smooth transitions
- Component stagger animations during build
- Spinner animations
- Thought item stagger animations

### ✅ ThoughtStream
- Toggle button pulse when new thoughts arrive
- Panel slide-in animation
- Backdrop fade-in
- Thought items stagger animation

### ✅ App (Main)
- Spotlight input fade-in
- Hint text fade-in
- Session status slide-up

### ✅ TitleBar
- Session menu overlay fade-in
- Session menu scale-in

## Benefits

1. **Consistency**: All animations use centralized timing and easing
2. **Performance**: GSAP is highly optimized and uses native browser APIs
3. **No Race Conditions**: Centralized timing prevents conflicting animations
4. **Better Control**: Easy to pause, reverse, or chain animations
5. **Maintainability**: All animation logic in one place
6. **TypeScript Support**: Full type safety with hooks
7. **React Integration**: Clean hooks API that follows React patterns
8. **Flexibility**: Easy to create complex animation sequences with timelines

## Timing Constants Reference

```typescript
ANIMATION_TIMING = {
  // Quick interactions
  instant: 0.15,
  fast: 0.2,
  quick: 0.3,
  
  // Standard animations
  normal: 0.4,
  moderate: 0.5,
  standard: 0.6,
  
  // Slower animations
  slow: 0.8,
  slower: 1.0,
  dramatic: 1.2,
  
  // Continuous
  pulse: 1.5,
  spin: 1.2,
  float: 4.0,
  shimmer: 2.0,
  wave: 2.0,
  glow: 3.0,
}
```

## Easing Reference

```typescript
EASING = {
  // Standard
  linear: 'none',
  ease: 'power1.inOut',
  easeIn: 'power1.in',
  easeOut: 'power1.out',
  
  // Common
  standard: 'power2.inOut', // cubic-bezier(0.4, 0, 0.2, 1)
  decelerate: 'power2.out',
  accelerate: 'power2.in',
  
  // Smooth
  smooth: 'power3.inOut',
  smoothOut: 'power3.out',
  smoothIn: 'power3.in',
  
  // Bouncy
  bounce: 'back.out(1.7)',
  bounceSoft: 'back.out(1.2)',
  bounceHard: 'back.out(2.0)',
  
  // Elastic
  elastic: 'elastic.out(1, 0.5)',
  elasticSoft: 'elastic.out(1, 0.3)',
  
  // Dramatic
  expoOut: 'expo.out',
  expoIn: 'expo.in',
  expoInOut: 'expo.inOut',
}
```

## CSS Animations

The CSS files still contain `@keyframes` and `animation` properties as fallbacks. These will work alongside GSAP animations, but GSAP animations take precedence in components where they're applied. This ensures graceful degradation if JavaScript is disabled or GSAP fails to load.

## Future Enhancements

1. **ScrollTrigger**: Add scroll-based animations when needed
2. **Custom Easings**: Add custom easing curves for brand-specific animations
3. **Animation Presets**: Create common animation sequences as presets
4. **Performance Monitoring**: Add FPS monitoring for animation performance
5. **Accessibility**: Add reduced motion support based on user preferences

## Best Practices

1. **Use Centralized Timing**: Always use `ANIMATION_TIMING` constants
2. **Consistent Easing**: Use `EASING` presets for consistent feel
3. **Avoid Overanimation**: Not everything needs to animate
4. **Performance**: Use `will-change` CSS property for frequently animated elements
5. **Cleanup**: Animations are automatically cleaned up on unmount
6. **Testing**: Test animations on different devices and browsers

## Documentation

- [GSAP Documentation](https://greensock.com/docs/)
- [GSAP React Guide](https://greensock.com/react/)
- [GSAP Easing Visualizer](https://greensock.com/ease-visualizer/)

## Support

For questions or issues with animations, refer to:
- Animation configuration: `src/utils/animationConfig.ts`
- Animation utilities: `src/utils/gsapAnimations.ts`
- React hooks: `src/hooks/useGSAP.ts`

