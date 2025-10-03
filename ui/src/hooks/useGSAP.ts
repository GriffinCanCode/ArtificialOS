/**
 * React Hooks for GSAP Animations
 * Easy-to-use hooks for integrating GSAP animations into React components
 */

import { useEffect, useRef, useCallback, RefObject } from 'react';
import gsap from 'gsap';
import * as animations from '../utils/animation/gsapAnimations';
import { ANIMATION_TIMING } from '../utils/animation/animationConfig';

/**
 * Hook to animate element on mount
 */
export function useAnimateOnMount<T extends HTMLElement>(
  animationFn: (element: HTMLElement) => gsap.core.Tween | gsap.core.Timeline,
  deps: any[] = []
) {
  const elementRef = useRef<T>(null);

  useEffect(() => {
    if (elementRef.current) {
      const animation = animationFn(elementRef.current);
      return () => {
        animation.kill();
      };
    }
  }, deps);

  return elementRef;
}

/**
 * Hook for fade in animation on mount
 */
export function useFadeIn<T extends HTMLElement>(
  options?: Parameters<typeof animations.fadeIn>[1]
) {
  return useAnimateOnMount<T>(
    (el) => animations.fadeIn(el, options),
    [options?.duration, options?.delay]
  );
}

/**
 * Hook for slide in up animation on mount
 */
export function useSlideInUp<T extends HTMLElement>(
  options?: Parameters<typeof animations.slideInUp>[1]
) {
  return useAnimateOnMount<T>(
    (el) => animations.slideInUp(el, options),
    [options?.duration, options?.delay, options?.distance]
  );
}

/**
 * Hook for slide in right animation on mount
 */
export function useSlideInRight<T extends HTMLElement>(
  options?: Parameters<typeof animations.slideInRight>[1]
) {
  return useAnimateOnMount<T>(
    (el) => animations.slideInRight(el, options),
    [options?.duration, options?.delay, options?.distance]
  );
}

/**
 * Hook for scale in animation on mount
 */
export function useScaleIn<T extends HTMLElement>(
  options?: Parameters<typeof animations.scaleIn>[1]
) {
  return useAnimateOnMount<T>(
    (el) => animations.scaleIn(el, options),
    [options?.duration, options?.delay, options?.scale]
  );
}

/**
 * Hook for app appear animation on mount
 */
export function useAppAppear<T extends HTMLElement>(
  options?: { delay?: number }
) {
  return useAnimateOnMount<T>(
    (el) => animations.appAppear(el, options),
    [options?.delay]
  );
}

/**
 * Hook for infinite animations (float, pulse, spin)
 */
export function useInfiniteAnimation<T extends HTMLElement>(
  animationFn: (element: HTMLElement) => gsap.core.Tween | gsap.core.Timeline,
  enabled: boolean = true
) {
  const elementRef = useRef<T>(null);
  const animationRef = useRef<gsap.core.Tween | gsap.core.Timeline | null>(null);

  useEffect(() => {
    if (elementRef.current && enabled) {
      animationRef.current = animationFn(elementRef.current);
      return () => {
        animationRef.current?.kill();
      };
    } else if (animationRef.current && !enabled) {
      animationRef.current.kill();
      animationRef.current = null;
    }
  }, [enabled]);

  return elementRef;
}

/**
 * Hook for float animation
 */
export function useFloat<T extends HTMLElement>(
  enabled: boolean = true,
  options?: Parameters<typeof animations.floatAnimation>[1]
) {
  return useInfiniteAnimation<T>(
    (el) => animations.floatAnimation(el, options),
    enabled
  );
}

/**
 * Hook for pulse animation
 */
export function usePulse<T extends HTMLElement>(
  enabled: boolean = true,
  options?: Parameters<typeof animations.pulseAnimation>[1]
) {
  return useInfiniteAnimation<T>(
    (el) => animations.pulseAnimation(el, options),
    enabled
  );
}

/**
 * Hook for spin animation
 */
export function useSpin<T extends HTMLElement>(
  enabled: boolean = true,
  options?: Parameters<typeof animations.spinAnimation>[1]
) {
  return useInfiniteAnimation<T>(
    (el) => animations.spinAnimation(el, options),
    enabled
  );
}

/**
 * Hook for build pulse animation
 */
export function useBuildPulse<T extends HTMLElement>(enabled: boolean = true) {
  return useInfiniteAnimation<T>(
    (el) => animations.buildPulseAnimation(el),
    enabled
  );
}

/**
 * Hook for glow pulse animation
 */
export function useGlowPulse<T extends HTMLElement>(
  enabled: boolean = true,
  options?: { color?: string; intensity?: number }
) {
  return useInfiniteAnimation<T>(
    (el) => animations.glowPulse(el, options),
    enabled
  );
}

/**
 * Hook for hover animations
 */
export function useHoverAnimation<T extends HTMLElement>(
  hoverInFn: (element: HTMLElement) => gsap.core.Tween,
  hoverOutFn: (element: HTMLElement) => gsap.core.Tween
) {
  const elementRef = useRef<T>(null);

  const handleMouseEnter = useCallback(() => {
    if (elementRef.current) {
      hoverInFn(elementRef.current);
    }
  }, [hoverInFn]);

  const handleMouseLeave = useCallback(() => {
    if (elementRef.current) {
      hoverOutFn(elementRef.current);
    }
  }, [hoverOutFn]);

  return {
    elementRef,
    hoverProps: {
      onMouseEnter: handleMouseEnter,
      onMouseLeave: handleMouseLeave,
    },
  };
}

/**
 * Hook for button hover animation
 */
export function useButtonHover<T extends HTMLElement>() {
  return useHoverAnimation<T>(
    animations.buttonHoverIn,
    animations.buttonHoverOut
  );
}

/**
 * Hook for scale hover animation
 */
export function useScaleHover<T extends HTMLElement>(scale?: number) {
  return useHoverAnimation<T>(
    (el) => animations.scaleHoverIn(el, scale),
    animations.scaleHoverOut
  );
}

/**
 * Hook for imperative animations (call when needed)
 */
export function useImperativeAnimation<T extends HTMLElement>() {
  const elementRef = useRef<T>(null);

  const animate = useCallback(
    (
      animationFn: (element: HTMLElement) => gsap.core.Tween | gsap.core.Timeline
    ) => {
      if (elementRef.current) {
        return animationFn(elementRef.current);
      }
      return null;
    },
    []
  );

  const fadeIn = useCallback(
    (options?: Parameters<typeof animations.fadeIn>[1]) => {
      return animate((el) => animations.fadeIn(el, options));
    },
    [animate]
  );

  const fadeOut = useCallback(
    (options?: Parameters<typeof animations.fadeOut>[1]) => {
      return animate((el) => animations.fadeOut(el, options));
    },
    [animate]
  );

  const shake = useCallback(() => {
    return animate(animations.errorShake);
  }, [animate]);

  const flash = useCallback(() => {
    return animate(animations.completionFlash);
  }, [animate]);

  const slideInUp = useCallback(
    (options?: Parameters<typeof animations.slideInUp>[1]) => {
      return animate((el) => animations.slideInUp(el, options));
    },
    [animate]
  );

  const scaleIn = useCallback(
    (options?: Parameters<typeof animations.scaleIn>[1]) => {
      return animate((el) => animations.scaleIn(el, options));
    },
    [animate]
  );

  const scaleOut = useCallback(
    (options?: Parameters<typeof animations.scaleOut>[1]) => {
      return animate((el) => animations.scaleOut(el, options));
    },
    [animate]
  );

  return {
    elementRef,
    fadeIn,
    fadeOut,
    shake,
    flash,
    slideInUp,
    scaleIn,
    scaleOut,
    animate,
  };
}

/**
 * Hook for stagger animations on multiple elements
 */
export function useStaggerAnimation<T extends HTMLElement>(
  selector: string,
  animationFn: (elements: NodeListOf<Element>) => gsap.core.Tween | gsap.core.Timeline,
  deps: any[] = []
) {
  const containerRef = useRef<T>(null);

  useEffect(() => {
    if (containerRef.current) {
      const elements = containerRef.current.querySelectorAll(selector);
      if (elements.length > 0) {
        const animation = animationFn(elements);
        return () => {
          animation.kill();
        };
      }
    }
  }, deps);

  return containerRef;
}

/**
 * Hook for stagger fade in on children
 */
export function useStaggerFadeIn<T extends HTMLElement>(
  selector: string = '> *',
  options?: Parameters<typeof animations.staggerFadeIn>[1]
) {
  return useStaggerAnimation<T>(
    selector,
    (elements) => animations.staggerFadeIn(elements, options),
    [options?.stagger, options?.duration]
  );
}

/**
 * Hook for stagger slide up on children
 */
export function useStaggerSlideUp<T extends HTMLElement>(
  selector: string = '> *',
  options?: Parameters<typeof animations.staggerSlideUp>[1]
) {
  return useStaggerAnimation<T>(
    selector,
    (elements) => animations.staggerSlideUp(elements, options),
    [options?.stagger, options?.duration, options?.distance]
  );
}

/**
 * Hook to create and manage a GSAP timeline
 */
export function useTimeline(options?: gsap.TimelineVars) {
  const timelineRef = useRef<gsap.core.Timeline | null>(null);

  useEffect(() => {
    timelineRef.current = gsap.timeline(options);
    return () => {
      timelineRef.current?.kill();
    };
  }, []);

  return timelineRef;
}

// ============================================================================
// ANIMATION HOOKS
// ============================================================================

/**
 * Hook for blur reveal animation
 */
export function useBlurReveal<T extends HTMLElement>(
  options?: Parameters<typeof animations.blurReveal>[1]
) {
  return useAnimateOnMount<T>(
    (el) => animations.blurReveal(el, options),
    [options?.duration, options?.delay]
  );
}

/**
 * Hook for elastic bounce in animation
 */
export function useElasticBounceIn<T extends HTMLElement>(
  options?: Parameters<typeof animations.elasticBounceIn>[1]
) {
  return useAnimateOnMount<T>(
    (el) => animations.elasticBounceIn(el, options),
    [options?.delay, options?.from]
  );
}

/**
 * Hook for ripple effect animation
 */
export function useRippleEffect<T extends HTMLElement>() {
  return useAnimateOnMount<T>(
    (el) => animations.rippleEffect(el),
    []
  );
}

/**
 * Hook for 3D flip animation on mount
 */
export function useFlip3D<T extends HTMLElement>(
  options?: Parameters<typeof animations.flip3D>[1]
) {
  return useAnimateOnMount<T>(
    (el) => animations.flip3D(el, options),
    [options?.direction, options?.degrees, options?.duration]
  );
}

/**
 * Hook for text reveal animation
 */
export function useTextReveal<T extends HTMLElement>(
  options?: Parameters<typeof animations.textReveal>[1]
) {
  return useAnimateOnMount<T>(
    (el) => animations.textReveal(el as HTMLElement, options),
    [options?.stagger, options?.duration]
  );
}

/**
 * Hook for wobble animation (imperative)
 */
export function useWobble<T extends HTMLElement>() {
  const elementRef = useRef<T>(null);

  const wobble = useCallback(() => {
    if (elementRef.current) {
      return animations.wobble(elementRef.current);
    }
    return null;
  }, []);

  return {
    elementRef,
    wobble,
  };
}

/**
 * Hook for glitch effect (imperative)
 */
export function useGlitch<T extends HTMLElement>() {
  const elementRef = useRef<T>(null);

  const glitch = useCallback(() => {
    if (elementRef.current) {
      return animations.glitchEffect(elementRef.current);
    }
    return null;
  }, []);

  return {
    elementRef,
    glitch,
  };
}

/**
 * Hook for particle burst effect (imperative)
 */
export function useParticleBurst<T extends HTMLElement>() {
  const elementRef = useRef<T>(null);

  const burst = useCallback(
    (options?: Parameters<typeof animations.particleBurst>[1]) => {
      if (elementRef.current) {
        return animations.particleBurst(elementRef.current, options);
      }
      return null;
    },
    []
  );

  return {
    elementRef,
    burst,
  };
}

/**
 * Hook for magnetic hover effect
 */
export function useMagneticHover<T extends HTMLElement>(
  options?: Parameters<typeof animations.magneticHover>[1]
) {
  const elementRef = useRef<T>(null);

  useEffect(() => {
    if (elementRef.current) {
      const cleanup = animations.magneticHover(elementRef.current, options);
      return cleanup;
    }
  }, [options?.strength, options?.speed]);

  return elementRef;
}

/**
 * Hook for smooth morph animation
 */
export function useSmoothMorph<T extends HTMLElement>() {
  const elementRef = useRef<T>(null);

  const morph = useCallback(
    (
      toProps: gsap.TweenVars,
      options?: Parameters<typeof animations.smoothMorph>[2]
    ) => {
      if (elementRef.current) {
        return animations.smoothMorph(elementRef.current, toProps, options);
      }
      return null;
    },
    []
  );

  return {
    elementRef,
    morph,
  };
}

/**
 * Hook for cleanup on unmount
 */
export function useAnimationCleanup<T extends HTMLElement>() {
  const elementRef = useRef<T>(null);

  useEffect(() => {
    return () => {
      if (elementRef.current) {
        gsap.killTweensOf(elementRef.current);
      }
    };
  }, []);

  return elementRef;
}

