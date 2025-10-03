/**
 * GSAP Animation Utilities
 * Reusable animation functions for consistent animations throughout the app
 */

import gsap from 'gsap';
import { ANIMATION_TIMING, EASING, STAGGER } from './animationConfig';

// ============================================================================
// BASIC ANIMATIONS
// ============================================================================

/**
 * Fade in animation
 */
export const fadeIn = (
  element: gsap.TweenTarget,
  options: {
    duration?: number;
    ease?: string;
    delay?: number;
    onComplete?: () => void;
  } = {}
) => {
  return gsap.fromTo(
    element,
    { opacity: 0 },
    {
      opacity: 1,
      duration: options.duration ?? ANIMATION_TIMING.normal,
      ease: options.ease ?? EASING.ease,
      delay: options.delay ?? 0,
      onComplete: options.onComplete,
    }
  );
};

/**
 * Fade out animation
 */
export const fadeOut = (
  element: gsap.TweenTarget,
  options: {
    duration?: number;
    ease?: string;
    delay?: number;
    onComplete?: () => void;
  } = {}
) => {
  return gsap.to(element, {
    opacity: 0,
    duration: options.duration ?? ANIMATION_TIMING.normal,
    ease: options.ease ?? EASING.ease,
    delay: options.delay ?? 0,
    onComplete: options.onComplete,
  });
};

/**
 * Slide in from bottom with fade
 */
export const slideInUp = (
  element: gsap.TweenTarget,
  options: {
    duration?: number;
    ease?: string;
    distance?: number;
    delay?: number;
  } = {}
) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0,
      y: options.distance ?? 20,
    },
    {
      opacity: 1,
      y: 0,
      duration: options.duration ?? ANIMATION_TIMING.quick,
      ease: options.ease ?? EASING.standard,
      delay: options.delay ?? 0,
    }
  );
};

/**
 * Slide in from right with fade
 */
export const slideInRight = (
  element: gsap.TweenTarget,
  options: {
    duration?: number;
    ease?: string;
    distance?: number;
    delay?: number;
  } = {}
) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0,
      x: options.distance ?? 20,
    },
    {
      opacity: 1,
      x: 0,
      duration: options.duration ?? ANIMATION_TIMING.quick,
      ease: options.ease ?? EASING.standard,
      delay: options.delay ?? 0,
    }
  );
};

/**
 * Slide in from top with fade
 */
export const slideInDown = (
  element: gsap.TweenTarget,
  options: {
    duration?: number;
    ease?: string;
    distance?: number;
    delay?: number;
  } = {}
) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0,
      y: -(options.distance ?? 10),
    },
    {
      opacity: 1,
      y: 0,
      duration: options.duration ?? ANIMATION_TIMING.quick,
      ease: options.ease ?? EASING.standard,
      delay: options.delay ?? 0,
    }
  );
};

/**
 * Scale in with fade (modal, popup)
 */
export const scaleIn = (
  element: gsap.TweenTarget,
  options: {
    duration?: number;
    ease?: string;
    scale?: number;
    delay?: number;
  } = {}
) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0,
      scale: options.scale ?? 0.95,
    },
    {
      opacity: 1,
      scale: 1,
      duration: options.duration ?? ANIMATION_TIMING.fast,
      ease: options.ease ?? EASING.standard,
      delay: options.delay ?? 0,
    }
  );
};

/**
 * Scale out with fade
 */
export const scaleOut = (
  element: gsap.TweenTarget,
  options: {
    duration?: number;
    ease?: string;
    scale?: number;
    delay?: number;
  } = {}
) => {
  return gsap.to(element, {
    opacity: 0,
    scale: options.scale ?? 0.95,
    duration: options.duration ?? ANIMATION_TIMING.fast,
    ease: options.ease ?? EASING.standard,
    delay: options.delay ?? 0,
  });
};

// ============================================================================
// COMPONENT-SPECIFIC ANIMATIONS
// ============================================================================

/**
 * App appear animation (rendered app container)
 */
export const appAppear = (
  element: gsap.TweenTarget,
  options: { delay?: number } = {}
) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0,
      scale: 0.92,
      y: 30,
      filter: 'blur(10px)',
    },
    {
      opacity: 1,
      scale: 1,
      y: 0,
      filter: 'blur(0px)',
      duration: ANIMATION_TIMING.standard,
      ease: EASING.bounce,
      delay: options.delay ?? 0,
    }
  );
};

/**
 * Build container animation
 */
export const buildContainerAppear = (
  element: gsap.TweenTarget,
  options: { delay?: number } = {}
) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0,
      scale: 0.95,
      y: 20,
    },
    {
      opacity: 1,
      scale: 1,
      y: 0,
      duration: ANIMATION_TIMING.moderate,
      ease: EASING.bounce,
      delay: options.delay ?? 0,
    }
  );
};

/**
 * Component appear animation (for building components)
 */
export const componentAppear = (
  element: gsap.TweenTarget,
  options: { delay?: number } = {}
) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0,
      y: 20,
      scale: 0.95,
    },
    {
      opacity: 1,
      y: 0,
      scale: 1,
      duration: ANIMATION_TIMING.moderate,
      ease: EASING.smoothOut,
      delay: options.delay ?? 0,
    }
  );
};

/**
 * Component materialize animation (skeleton to real)
 */
export const componentMaterialize = (
  element: gsap.TweenTarget,
  options: { delay?: number } = {}
) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0.6,
      filter: 'blur(4px) brightness(1.2)',
    },
    {
      opacity: 1,
      filter: 'blur(0px) brightness(1)',
      duration: ANIMATION_TIMING.standard,
      ease: EASING.smoothOut,
      delay: options.delay ?? 0,
    }
  );
};

/**
 * Error shake animation
 */
export const errorShake = (element: gsap.TweenTarget) => {
  return gsap.fromTo(
    element,
    { x: 0 },
    {
      x: -5,
      duration: 0.05,
      repeat: 9,
      yoyo: true,
      ease: EASING.linear,
    }
  );
};

/**
 * Completion flash animation
 */
export const completionFlash = (element: gsap.TweenTarget) => {
  const tl = gsap.timeline();
  
  tl.to(element, {
    boxShadow: '0 0 0 0 rgba(255, 255, 255, 0.7)',
    duration: 0,
  })
    .to(element, {
      boxShadow: '0 0 40px 10px rgba(138, 101, 255, 0.6)',
      duration: 0.4,
      ease: EASING.ease,
    })
    .to(element, {
      boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
      duration: 0.4,
      ease: EASING.ease,
    });
  
  return tl;
};

// ============================================================================
// CONTINUOUS/INFINITE ANIMATIONS
// ============================================================================

/**
 * Float animation (infinite)
 */
export const floatAnimation = (
  element: gsap.TweenTarget,
  options: {
    distance?: number;
    duration?: number;
  } = {}
) => {
  return gsap.to(element, {
    y: -(options.distance ?? 12),
    scale: 1.05,
    duration: options.duration ?? ANIMATION_TIMING.float / 2,
    ease: 'sine.inOut',
    repeat: -1,
    yoyo: true,
  });
};

/**
 * Pulse animation (infinite)
 */
export const pulseAnimation = (
  element: gsap.TweenTarget,
  options: {
    opacity?: number;
    duration?: number;
  } = {}
) => {
  return gsap.to(element, {
    opacity: options.opacity ?? 0.7,
    duration: options.duration ?? ANIMATION_TIMING.pulse / 2,
    ease: 'sine.inOut',
    repeat: -1,
    yoyo: true,
  });
};

/**
 * Spin animation (infinite)
 */
export const spinAnimation = (
  element: gsap.TweenTarget,
  options: {
    duration?: number;
    ease?: string;
  } = {}
) => {
  return gsap.to(element, {
    rotation: 360,
    duration: options.duration ?? ANIMATION_TIMING.spin,
    ease: options.ease ?? EASING.linear,
    repeat: -1,
  });
};

/**
 * Border glow animation (infinite pulse)
 */
export const borderGlowAnimation = (element: gsap.TweenTarget) => {
  return gsap.to(element, {
    opacity: 1,
    duration: ANIMATION_TIMING.glow / 2,
    ease: 'sine.inOut',
    repeat: -1,
    yoyo: true,
  });
};

/**
 * Build pulse animation (infinite)
 */
export const buildPulseAnimation = (element: gsap.TweenTarget) => {
  const tl = gsap.timeline({ repeat: -1 });
  
  tl.to(element, {
    scale: 1.08,
    rotation: -3,
    duration: 0.5,
    ease: 'sine.out',
  })
    .to(element, {
      scale: 1.12,
      rotation: 0,
      opacity: 0.9,
      duration: 0.5,
      ease: 'sine.inOut',
    })
    .to(element, {
      scale: 1.08,
      rotation: 3,
      duration: 0.5,
      ease: 'sine.inOut',
    })
    .to(element, {
      scale: 1,
      rotation: 0,
      opacity: 1,
      duration: 0.5,
      ease: 'sine.in',
    });
  
  return tl;
};

/**
 * Progress shimmer animation (infinite)
 */
export const progressShimmerAnimation = (element: gsap.TweenTarget) => {
  return gsap.fromTo(
    element,
    { backgroundPosition: '200% 0' },
    {
      backgroundPosition: '-200% 0',
      duration: ANIMATION_TIMING.glow,
      ease: 'none',
      repeat: -1,
    }
  );
};

/**
 * Grid scan animation (infinite)
 */
export const gridScanAnimation = (element: gsap.TweenTarget) => {
  return gsap.fromTo(
    element,
    { backgroundPosition: '0 0' },
    {
      backgroundPosition: '20px 20px',
      duration: ANIMATION_TIMING.gridScan,
      ease: 'none',
      repeat: -1,
    }
  );
};

/**
 * Assembly pulse animation (infinite)
 */
export const assemblyPulseAnimation = (element: gsap.TweenTarget) => {
  const tl = gsap.timeline({ repeat: -1 });
  
  tl.to(element, {
    borderColor: 'rgba(99, 102, 241, 0.4)',
    backgroundColor: 'rgba(99, 102, 241, 0.05)',
    duration: 0.5,
    ease: 'sine.inOut',
  }).to(element, {
    borderColor: 'rgba(168, 85, 247, 0.4)',
    backgroundColor: 'rgba(168, 85, 247, 0.05)',
    duration: 0.5,
    ease: 'sine.inOut',
  });
  
  return tl;
};

// ============================================================================
// STAGGER ANIMATIONS
// ============================================================================

/**
 * Stagger fade in animation for multiple elements
 */
export const staggerFadeIn = (
  elements: gsap.TweenTarget,
  options: {
    stagger?: number;
    duration?: number;
    ease?: string;
  } = {}
) => {
  return gsap.fromTo(
    elements,
    { opacity: 0 },
    {
      opacity: 1,
      duration: options.duration ?? ANIMATION_TIMING.normal,
      ease: options.ease ?? EASING.ease,
      stagger: options.stagger ?? STAGGER.normal,
    }
  );
};

/**
 * Stagger slide up animation for multiple elements
 */
export const staggerSlideUp = (
  elements: gsap.TweenTarget,
  options: {
    stagger?: number;
    duration?: number;
    ease?: string;
    distance?: number;
  } = {}
) => {
  return gsap.fromTo(
    elements,
    {
      opacity: 0,
      y: options.distance ?? 20,
    },
    {
      opacity: 1,
      y: 0,
      duration: options.duration ?? ANIMATION_TIMING.moderate,
      ease: options.ease ?? EASING.smoothOut,
      stagger: options.stagger ?? STAGGER.normal,
    }
  );
};

// ============================================================================
// HOVER ANIMATIONS
// ============================================================================

/**
 * Button hover animation
 */
export const buttonHoverIn = (element: gsap.TweenTarget) => {
  return gsap.to(element, {
    y: -3,
    scale: 1.01,
    duration: ANIMATION_TIMING.quick,
    ease: EASING.standard,
  });
};

export const buttonHoverOut = (element: gsap.TweenTarget) => {
  return gsap.to(element, {
    y: 0,
    scale: 1,
    duration: ANIMATION_TIMING.quick,
    ease: EASING.standard,
  });
};

/**
 * Scale hover animation
 */
export const scaleHoverIn = (element: gsap.TweenTarget, scale: number = 1.05) => {
  return gsap.to(element, {
    scale,
    duration: ANIMATION_TIMING.fast,
    ease: EASING.standard,
  });
};

export const scaleHoverOut = (element: gsap.TweenTarget) => {
  return gsap.to(element, {
    scale: 1,
    duration: ANIMATION_TIMING.fast,
    ease: EASING.standard,
  });
};

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Kill all animations on an element
 */
export const killAnimations = (element: gsap.TweenTarget) => {
  gsap.killTweensOf(element);
};

/**
 * Set element properties without animation
 */
export const setProps = (element: gsap.TweenTarget, props: gsap.TweenVars) => {
  gsap.set(element, props);
};

/**
 * Create a timeline for complex animations
 */
export const createTimeline = (options?: gsap.TimelineVars) => {
  return gsap.timeline(options);
};

