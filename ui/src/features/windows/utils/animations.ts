/**
 * Animation Utilities
 * Window animation helpers
 */

import type { Bounds } from "../core/types";
import { ANIMATION_DURATION } from "../core/types";

export interface AnimationConfig {
  duration?: number;
  easing?: string;
}

const DEFAULT_CONFIG: AnimationConfig = {
  duration: ANIMATION_DURATION,
  easing: "cubic-bezier(0.4, 0, 0.2, 1)",
};

/**
 * Animate bounds transition
 */
export function animateBounds(
  element: HTMLElement,
  from: Bounds,
  to: Bounds,
  config: AnimationConfig = {}
): Promise<void> {
  const { duration, easing } = { ...DEFAULT_CONFIG, ...config };

  return new Promise((resolve) => {
    const animation = element.animate(
      [
        {
          left: `${from.position.x}px`,
          top: `${from.position.y}px`,
          width: `${from.size.width}px`,
          height: `${from.size.height}px`,
        },
        {
          left: `${to.position.x}px`,
          top: `${to.position.y}px`,
          width: `${to.size.width}px`,
          height: `${to.size.height}px`,
        },
      ],
      {
        duration,
        easing,
        fill: "forwards",
      }
    );

    animation.onfinish = () => resolve();
  });
}

/**
 * Fade in animation
 */
export function fadeIn(element: HTMLElement, config: AnimationConfig = {}): Promise<void> {
  const { duration, easing } = { ...DEFAULT_CONFIG, ...config };

  return new Promise((resolve) => {
    const animation = element.animate(
      [
        { opacity: 0, transform: "scale(0.95)" },
        { opacity: 1, transform: "scale(1)" },
      ],
      { duration, easing, fill: "forwards" }
    );

    animation.onfinish = () => resolve();
  });
}

/**
 * Fade out animation
 */
export function fadeOut(element: HTMLElement, config: AnimationConfig = {}): Promise<void> {
  const { duration, easing } = { ...DEFAULT_CONFIG, ...config };

  return new Promise((resolve) => {
    const animation = element.animate(
      [
        { opacity: 1, transform: "scale(1)" },
        { opacity: 0, transform: "scale(0.95)" },
      ],
      { duration, easing, fill: "forwards" }
    );

    animation.onfinish = () => resolve();
  });
}

/**
 * Minimize animation (to taskbar)
 */
export function animateMinimize(
  element: HTMLElement,
  target: { x: number; y: number },
  config: AnimationConfig = {}
): Promise<void> {
  const { duration, easing } = { ...DEFAULT_CONFIG, ...config };
  const rect = element.getBoundingClientRect();
  const current = { x: rect.left, y: rect.top };

  return new Promise((resolve) => {
    const animation = element.animate(
      [
        {
          left: `${current.x}px`,
          top: `${current.y}px`,
          opacity: 1,
          transform: "scale(1)",
        },
        {
          left: `${target.x}px`,
          top: `${target.y}px`,
          opacity: 0,
          transform: "scale(0.1)",
        },
      ],
      { duration, easing, fill: "forwards" }
    );

    animation.onfinish = () => resolve();
  });
}

/**
 * Restore animation (from taskbar)
 */
export function animateRestore(
  element: HTMLElement,
  from: { x: number; y: number },
  to: Bounds,
  config: AnimationConfig = {}
): Promise<void> {
  const { duration, easing } = { ...DEFAULT_CONFIG, ...config };

  return new Promise((resolve) => {
    const animation = element.animate(
      [
        {
          left: `${from.x}px`,
          top: `${from.y}px`,
          opacity: 0,
          transform: "scale(0.1)",
        },
        {
          left: `${to.position.x}px`,
          top: `${to.position.y}px`,
          opacity: 1,
          transform: "scale(1)",
        },
      ],
      { duration, easing, fill: "forwards" }
    );

    animation.onfinish = () => resolve();
  });
}
