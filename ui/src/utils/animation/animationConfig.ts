/**
 * Centralized Animation Configuration
 * All timing constants and easing functions in one place
 * Ensures consistency across the application and prevents race conditions
 */

// ============================================================================
// TIMING CONSTANTS - All animation durations in seconds
// ============================================================================

export const ANIMATION_TIMING = {
  // Quick interactions (hover, focus, clicks)
  instant: 0.15,
  fast: 0.2,
  quick: 0.3,
  
  // Standard animations (default for most UI transitions)
  normal: 0.4,
  moderate: 0.5,
  standard: 0.6,
  
  // Slower, more dramatic animations
  slow: 0.8,
  slower: 1.0,
  dramatic: 1.2,
  
  // Continuous/infinite animations
  pulse: 1.5,
  spin: 1.2,
  float: 4.0,
  shimmer: 2.0,
  wave: 2.0,
  glow: 3.0,
  borderPulse: 3.0,
  gridScan: 3.0,
  buildPulse: 2.0,
  assemblyPulse: 1.0,
} as const;

// ============================================================================
// EASING PRESETS - Standard easing functions
// ============================================================================

export const EASING = {
  // Standard CSS equivalents
  linear: 'none',
  ease: 'power1.inOut',
  easeIn: 'power1.in',
  easeOut: 'power1.out',
  
  // Common cubic-bezier equivalents
  standard: 'power2.inOut', // cubic-bezier(0.4, 0, 0.2, 1)
  decelerate: 'power2.out',
  accelerate: 'power2.in',
  
  // Smooth, natural animations
  smooth: 'power3.inOut',
  smoothOut: 'power3.out',
  smoothIn: 'power3.in',
  
  // Bouncy, playful animations
  bounce: 'back.out(1.7)',
  bounceSoft: 'back.out(1.2)',
  bounceHard: 'back.out(2.0)',
  
  // Elastic animations
  elastic: 'elastic.out(1, 0.5)',
  elasticSoft: 'elastic.out(1, 0.3)',
  
  // Expo for dramatic effects
  expoOut: 'expo.out',
  expoIn: 'expo.in',
  expoInOut: 'expo.inOut',
} as const;

// ============================================================================
// STAGGER CONFIGURATIONS - For animating multiple elements
// ============================================================================

export const STAGGER = {
  quick: 0.03,
  fast: 0.05,
  normal: 0.08,
  slow: 0.12,
  dramatic: 0.2,
} as const;

// ============================================================================
// ANIMATION STATES - Common initial/final states
// ============================================================================

export const ANIMATION_STATES = {
  hidden: {
    opacity: 0,
  },
  visible: {
    opacity: 1,
  },
  fadeInUp: {
    initial: { opacity: 0, y: 20 },
    animate: { opacity: 1, y: 0 },
  },
  fadeInDown: {
    initial: { opacity: 0, y: -20 },
    animate: { opacity: 1, y: 0 },
  },
  fadeInLeft: {
    initial: { opacity: 0, x: -20 },
    animate: { opacity: 1, x: 0 },
  },
  fadeInRight: {
    initial: { opacity: 0, x: 20 },
    animate: { opacity: 1, x: 0 },
  },
  scaleIn: {
    initial: { opacity: 0, scale: 0.9 },
    animate: { opacity: 1, scale: 1 },
  },
  scaleOut: {
    initial: { opacity: 1, scale: 1 },
    animate: { opacity: 0, scale: 0.9 },
  },
} as const;

// ============================================================================
// COMPONENT-SPECIFIC TIMINGS
// ============================================================================

export const COMPONENT_TIMINGS = {
  // Rendered App animations
  appAppear: {
    duration: ANIMATION_TIMING.standard,
    ease: EASING.bounce,
  },
  
  // Build preview animations
  buildContainer: {
    duration: ANIMATION_TIMING.moderate,
    ease: EASING.smoothOut,
  },
  
  // Component building animations
  componentAppear: {
    duration: ANIMATION_TIMING.moderate,
    ease: EASING.smoothOut,
    stagger: STAGGER.fast,
  },
  
  // Thought stream animations
  thoughtSlide: {
    duration: ANIMATION_TIMING.quick,
    ease: EASING.standard,
  },
  
  // Button interactions
  buttonHover: {
    duration: ANIMATION_TIMING.quick,
    ease: EASING.smoothOut,
  },
  
  // Input focus
  inputFocus: {
    duration: ANIMATION_TIMING.quick,
    ease: EASING.standard,
  },
  
  // Modal animations
  modalIn: {
    duration: ANIMATION_TIMING.fast,
    ease: EASING.standard,
  },
  
  // Spotlight input
  spotlightFocus: {
    duration: ANIMATION_TIMING.quick,
    ease: EASING.standard,
  },
  
  // Error shake
  errorShake: {
    duration: ANIMATION_TIMING.moderate,
    ease: EASING.elastic,
  },
} as const;

// ============================================================================
// Z-INDEX LAYERS - For proper stacking order
// ============================================================================

export const Z_INDEX = {
  base: 0,
  background: -1,
  content: 1,
  overlay: 900,
  modal: 998,
  panel: 997,
  dropdown: 999,
  tooltip: 1000,
  notification: 1001,
} as const;

export default {
  ANIMATION_TIMING,
  EASING,
  STAGGER,
  ANIMATION_STATES,
  COMPONENT_TIMINGS,
  Z_INDEX,
};

