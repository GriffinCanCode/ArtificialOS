/**
 * React Spring Animation Hooks
 * Physics-based, natural motion animations that complement GSAP
 * 
 * Use these for:
 * - Interactive, gesture-driven animations
 * - Component lifecycle animations
 * - Mouse/touch-following effects
 * - Natural spring physics
 */

import { useSpring, useTrail, useSprings, animated } from '@react-spring/web';
import { useDrag } from '@use-gesture/react';
import { useState, useCallback, useEffect, useRef } from 'react';
import { springConfigs } from '../utils/gsapAnimations';

// Re-export animated for convenience
export { animated };

// ============================================================================
// ENTRANCE ANIMATIONS
// ============================================================================

/**
 * Spring-based fade in with scale
 */
export function useSpringFadeIn(active: boolean = true, springConfig = springConfigs.bouncy) {
  const spring = useSpring({
    from: { opacity: 0, scale: 0.9 },
    to: { 
      opacity: active ? 1 : 0,
      scale: active ? 1 : 0.9,
    },
    config: springConfig,
  });
  
  return spring;
}

/**
 * Spring-based slide in from any direction
 */
export function useSpringSlideIn(
  active: boolean = true,
  direction: 'left' | 'right' | 'top' | 'bottom' = 'bottom',
  distance: number = 50,
  springConfig = springConfigs.bouncy
) {
  const getInitialPos = () => {
    switch (direction) {
      case 'left': return { x: -distance, y: 0 };
      case 'right': return { x: distance, y: 0 };
      case 'top': return { y: -distance, x: 0 };
      case 'bottom': return { y: distance, x: 0 };
    }
  };
  
  const spring = useSpring({
    from: { ...getInitialPos(), opacity: 0 },
    to: { 
      x: active ? 0 : getInitialPos().x,
      y: active ? 0 : getInitialPos().y,
      opacity: active ? 1 : 0,
    },
    config: springConfig,
  });
  
  return spring;
}

/**
 * Spring-based rotate in
 */
export function useSpringRotateIn(active: boolean = true, springConfig = springConfigs.wobbly) {
  const spring = useSpring({
    from: { opacity: 0, scale: 0, rotate: -180 },
    to: { 
      opacity: active ? 1 : 0,
      scale: active ? 1 : 0,
      rotate: active ? 0 : -180,
    },
    config: springConfig,
  });
  
  return spring;
}

// ============================================================================
// INTERACTIVE ANIMATIONS
// ============================================================================

/**
 * Magnetic hover effect - element follows cursor with spring physics
 */
export function useMagneticHover(strength: number = 0.2, springConfig = springConfigs.bouncy) {
  const [{ x, y }, api] = useSpring(() => ({ x: 0, y: 0, config: springConfig }));
  const ref = useRef<HTMLElement>(null);
  
  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!ref.current) return;
    const rect = ref.current.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;
    
    const deltaX = (e.clientX - centerX) * strength;
    const deltaY = (e.clientY - centerY) * strength;
    
    api.start({ x: deltaX, y: deltaY });
  }, [api, strength]);
  
  const handleMouseLeave = useCallback(() => {
    api.start({ x: 0, y: 0 });
  }, [api]);
  
  return { style: { x, y }, ref, handlers: { onMouseMove: handleMouseMove, onMouseLeave: handleMouseLeave } };
}

/**
 * Drag with spring physics and rubber band effect
 */
export function useDraggableSpring(
  onDragEnd?: (info: { offset: [number, number], velocity: [number, number] }) => void,
  springConfig = springConfigs.rubber
) {
  const [{ x, y }, api] = useSpring(() => ({ x: 0, y: 0, config: springConfig }));
  
  const bind = useDrag(({ offset: [ox, oy], velocity: [vx, vy], down }) => {
    api.start({ x: ox, y: oy, immediate: down });
    
    if (!down && onDragEnd) {
      onDragEnd({ offset: [ox, oy], velocity: [vx, vy] });
    }
  }, {
    from: () => [x.get(), y.get()],
  });
  
  return { style: { x, y, touchAction: 'none' }, bind };
}

/**
 * Parallax effect on mouse move
 */
export function useParallax(depth: number = 0.05, springConfig = springConfigs.gentle) {
  const [{ x, y }, api] = useSpring(() => ({ x: 0, y: 0, config: springConfig }));
  
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      const x = (e.clientX - window.innerWidth / 2) * depth;
      const y = (e.clientY - window.innerHeight / 2) * depth;
      api.start({ x, y });
    };
    
    window.addEventListener('mousemove', handleMouseMove);
    return () => window.removeEventListener('mousemove', handleMouseMove);
  }, [api, depth]);
  
  return { x, y };
}

// ============================================================================
// CONTINUOUS ANIMATIONS
// ============================================================================

/**
 * Floating animation with spring physics
 */
export function useFloatSpring(
  amplitude: number = 10,
  duration: number = 3000,
  springConfig = springConfigs.gentle
) {
  const [flip, setFlip] = useState(false);
  
  const spring = useSpring({
    from: { y: 0 },
    to: { y: flip ? -amplitude : amplitude },
    config: springConfig,
    onRest: () => setFlip(!flip),
  });
  
  return spring;
}

/**
 * Breathing animation with spring physics
 */
export function useBreatheSpring(
  scaleRange: [number, number] = [1, 1.05],
  duration: number = 3000
) {
  const [flip, setFlip] = useState(false);
  
  const spring = useSpring({
    from: { scale: scaleRange[0] },
    to: { scale: flip ? scaleRange[1] : scaleRange[0] },
    config: { duration: duration },
    onRest: () => setFlip(!flip),
  });
  
  return spring;
}

// ============================================================================
// SEQUENTIAL & TRAIL ANIMATIONS
// ============================================================================

/**
 * Stagger animation using trails - great for lists
 */
export function useSpringTrail<T>(
  items: T[],
  active: boolean = true,
  springConfig = springConfigs.bouncy
) {
  const trail = useTrail(items.length, {
    from: { opacity: 0, x: -20, scale: 0.9 },
    to: { 
      opacity: active ? 1 : 0,
      x: active ? 0 : -20,
      scale: active ? 1 : 0.9,
    },
    config: springConfig,
  });
  
  return trail;
}

/**
 * Chain animation - elements follow each other
 */
export function useSpringChain(
  count: number,
  leadPosition: { x: number; y: number },
  spacing: number = 50,
  springConfig = springConfigs.bouncy
) {
  const [springs, api] = useSprings(count, () => ({
    x: 0,
    y: 0,
    config: springConfig,
  }));
  
  useEffect(() => {
    api.start((index) => {
      const offset = index * spacing;
      return {
        x: leadPosition.x - offset,
        y: leadPosition.y,
        delay: index * 50,
      };
    });
  }, [leadPosition, spacing, api]);
  
  return springs;
}

// ============================================================================
// GESTURE-BASED ANIMATIONS
// ============================================================================

/**
 * Swipe to dismiss with spring physics
 */
export function useSwipeToDismiss(
  onDismiss: () => void,
  threshold: number = 100,
  springConfig = springConfigs.stiff
) {
  const [{ x, opacity }, api] = useSpring(() => ({ 
    x: 0, 
    opacity: 1,
    config: springConfig,
  }));
  
  const bind = useDrag(({ offset: [ox], velocity: [vx], down, direction: [dirX] }) => {
    const trigger = Math.abs(ox) > threshold || (Math.abs(vx) > 0.5 && Math.abs(ox) > threshold * 0.5);
    
    if (trigger && !down) {
      api.start({ 
        x: dirX > 0 ? window.innerWidth : -window.innerWidth,
        opacity: 0,
        config: { ...springConfig, velocity: vx * 1000 },
        onRest: onDismiss,
      });
    } else {
      api.start({ 
        x: down ? ox : 0,
        opacity: 1 - Math.min(Math.abs(ox) / threshold, 0.7),
        immediate: down,
      });
    }
  }, {
    axis: 'x',
    from: () => [x.get(), 0],
  });
  
  return { style: { x, opacity, touchAction: 'pan-y' }, bind };
}

/**
 * Pull to refresh with spring physics
 */
export function usePullToRefresh(
  onRefresh: () => Promise<void>,
  threshold: number = 80,
  springConfig = springConfigs.rubber
) {
  const [refreshing, setRefreshing] = useState(false);
  const [{ y, rotate }, api] = useSpring(() => ({ 
    y: 0,
    rotate: 0,
    config: springConfig,
  }));
  
  const bind = useDrag(({ offset: [, oy], down, direction: [, dirY] }) => {
    if (refreshing) return;
    
    const pullDistance = Math.max(0, oy);
    const trigger = pullDistance > threshold && !down && dirY > 0;
    
    if (trigger) {
      setRefreshing(true);
      api.start({ y: threshold, rotate: 360 });
      onRefresh().finally(() => {
        setRefreshing(false);
        api.start({ y: 0, rotate: 0 });
      });
    } else {
      const dampedY = down ? pullDistance * 0.5 : 0;
      api.start({ 
        y: dampedY,
        rotate: (pullDistance / threshold) * 180,
        immediate: down,
      });
    }
  }, {
    axis: 'y',
    bounds: { top: 0 },
    rubberband: true,
    from: () => [0, y.get()],
  });
  
  return { style: { y, rotate }, bind, refreshing };
}

// ============================================================================
// ATTENTION-GRABBING ANIMATIONS
// ============================================================================

/**
 * Jello wobble - great for errors or notifications
 */
export function useJelloWobble(trigger: boolean) {
  const [springs, api] = useSprings(1, () => ({
    from: { skewX: 0, skewY: 0, scaleX: 1, scaleY: 1 },
    config: springConfigs.jello,
  }));
  
  useEffect(() => {
    if (trigger) {
      const sequence = async () => {
        await api.start({ skewX: -12.5, skewY: -12.5, scaleX: 1.05, scaleY: 0.95 });
        await api.start({ skewX: 6.25, skewY: 6.25, scaleX: 0.95, scaleY: 1.05 });
        await api.start({ skewX: -3.125, skewY: -3.125, scaleX: 1.02, scaleY: 0.98 });
        await api.start({ skewX: 1.5625, skewY: 1.5625, scaleX: 0.98, scaleY: 1.02 });
        await api.start({ skewX: 0, skewY: 0, scaleX: 1, scaleY: 1 });
      };
      sequence();
    }
  }, [trigger, api]);
  
  return springs[0];
}

/**
 * Bounce attention - bounces up and down
 */
export function useBounceAttention(active: boolean) {
  const spring = useSpring({
    from: { y: 0 },
    to: async (next) => {
      if (active) {
        await next({ y: -20 });
        await next({ y: 0 });
        await next({ y: -10 });
        await next({ y: 0 });
      }
    },
    config: springConfigs.bouncy,
  });
  
  return spring;
}

// ============================================================================
// UTILITY HOOKS
// ============================================================================

/**
 * Spring value that follows a target with physics
 */
export function useFollowValue(target: number, springConfig = springConfigs.default) {
  const spring = useSpring({
    from: { value: target },
    to: { value: target },
    config: springConfig,
  });
  
  useEffect(() => {
    spring.value.start(target);
  }, [target, spring]);
  
  return spring.value;
}

export default {
  useSpringFadeIn,
  useSpringSlideIn,
  useSpringRotateIn,
  useMagneticHover,
  useDraggableSpring,
  useParallax,
  useFloatSpring,
  useBreatheSpring,
  useSpringTrail,
  useSpringChain,
  useSwipeToDismiss,
  usePullToRefresh,
  useJelloWobble,
  useBounceAttention,
  useFollowValue,
};

