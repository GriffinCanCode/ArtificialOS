/**
 * GSAP Animation Utilities
 * Reusable animation functions for consistent animations throughout the app
 *
 * Performance optimizations:
 * - Using force3D for GPU acceleration
 * - Adding will-change hints
 * - Optimized easing functions
 */

import gsap from "gsap";
import { ANIMATION_TIMING, EASING, STAGGER } from "./animationConfig";

// Global GSAP performance settings
gsap.config({
  force3D: true,
  nullTargetWarn: false,
});

// ============================================================================
// BASIC ANIMATIONS
// ============================================================================

/**
 * Fade in animation - optimized for performance
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
      ease: options.ease ?? EASING.smoothOut,
      delay: options.delay ?? 0,
      onComplete: options.onComplete,
      force3D: true,
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
 * Slide in from bottom with fade - GPU accelerated
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
      ease: options.ease ?? EASING.smoothOut,
      delay: options.delay ?? 0,
      force3D: true,
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
 * Scale in with fade (modal, popup) - smoother with elastic easing
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
      scale: options.scale ?? 0.9,
    },
    {
      opacity: 1,
      scale: 1,
      duration: options.duration ?? ANIMATION_TIMING.moderate,
      ease: options.ease ?? EASING.bounceSoft,
      delay: options.delay ?? 0,
      force3D: true,
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
 * App appear animation (rendered app container) - enhanced with smoother transition
 */
export const appAppear = (element: gsap.TweenTarget, options: { delay?: number } = {}) => {
  const tl = gsap.timeline({ delay: options.delay ?? 0 });

  tl.fromTo(
    element,
    {
      opacity: 0,
      scale: 0.85,
      y: 40,
      filter: "blur(15px)",
      rotateX: 10,
    },
    {
      opacity: 1,
      scale: 1,
      y: 0,
      filter: "blur(0px)",
      rotateX: 0,
      duration: ANIMATION_TIMING.slow,
      ease: "power4.out",
      force3D: true,
    }
  ).to(
    element,
    {
      boxShadow: "0 30px 90px rgba(0, 0, 0, 0.5), 0 0 1px rgba(255, 255, 255, 0.1) inset",
      duration: 0.3,
      ease: "power2.inOut",
    },
    "-=0.3"
  );

  return tl;
};

/**
 * Build container animation - optimized for smooth performance
 */
export const buildContainerAppear = (
  element: gsap.TweenTarget,
  options: { delay?: number } = {}
) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0,
      scale: 0.96,
      y: 15,
    },
    {
      opacity: 1,
      scale: 1,
      y: 0,
      duration: ANIMATION_TIMING.moderate,
      ease: EASING.smoothOut,
      delay: options.delay ?? 0,
      force3D: true,
      willChange: "transform, opacity",
    }
  );
};

/**
 * Component appear animation (for building components) - optimized for smooth performance
 */
export const componentAppear = (element: gsap.TweenTarget, options: { delay?: number } = {}) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0,
      y: 20,
      scale: 0.96,
    },
    {
      opacity: 1,
      y: 0,
      scale: 1,
      duration: ANIMATION_TIMING.moderate,
      ease: EASING.smoothOut,
      delay: options.delay ?? 0,
      force3D: true,
      willChange: "transform, opacity",
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
      filter: "blur(4px) brightness(1.2)",
    },
    {
      opacity: 1,
      filter: "blur(0px) brightness(1)",
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
    boxShadow: "0 0 0 0 rgba(255, 255, 255, 0.7)",
    duration: 0,
  })
    .to(element, {
      boxShadow: "0 0 40px 10px rgba(138, 101, 255, 0.6)",
      duration: 0.4,
      ease: EASING.ease,
    })
    .to(element, {
      boxShadow: "0 8px 32px rgba(0, 0, 0, 0.3)",
      duration: 0.4,
      ease: EASING.ease,
    });

  return tl;
};

// ============================================================================
// CONTINUOUS/INFINITE ANIMATIONS
// ============================================================================

/**
 * Float animation (infinite) - GPU accelerated
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
    ease: "sine.inOut",
    repeat: -1,
    yoyo: true,
    force3D: true,
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
    ease: "sine.inOut",
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
    ease: "sine.inOut",
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
    ease: "sine.out",
  })
    .to(element, {
      scale: 1.12,
      rotation: 0,
      opacity: 0.9,
      duration: 0.5,
      ease: "sine.inOut",
    })
    .to(element, {
      scale: 1.08,
      rotation: 3,
      duration: 0.5,
      ease: "sine.inOut",
    })
    .to(element, {
      scale: 1,
      rotation: 0,
      opacity: 1,
      duration: 0.5,
      ease: "sine.in",
    });

  return tl;
};

/**
 * Progress shimmer animation (infinite)
 */
export const progressShimmerAnimation = (element: gsap.TweenTarget) => {
  return gsap.fromTo(
    element,
    { backgroundPosition: "200% 0" },
    {
      backgroundPosition: "-200% 0",
      duration: ANIMATION_TIMING.glow,
      ease: "none",
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
    { backgroundPosition: "0 0" },
    {
      backgroundPosition: "20px 20px",
      duration: ANIMATION_TIMING.gridScan,
      ease: "none",
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
    borderColor: "rgba(99, 102, 241, 0.4)",
    backgroundColor: "rgba(99, 102, 241, 0.05)",
    duration: 0.5,
    ease: "sine.inOut",
  }).to(element, {
    borderColor: "rgba(168, 85, 247, 0.4)",
    backgroundColor: "rgba(168, 85, 247, 0.05)",
    duration: 0.5,
    ease: "sine.inOut",
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
 * Button hover animation - optimized for smooth performance
 */
export const buttonHoverIn = (element: gsap.TweenTarget) => {
  return gsap.to(element, {
    y: -2,
    scale: 1.01,
    duration: ANIMATION_TIMING.quick,
    ease: EASING.smoothOut,
    force3D: true,
    willChange: "transform",
  });
};

export const buttonHoverOut = (element: gsap.TweenTarget) => {
  return gsap.to(element, {
    y: 0,
    scale: 1,
    duration: ANIMATION_TIMING.quick,
    ease: EASING.smoothOut,
    force3D: true,
    clearProps: "willChange",
  });
};

/**
 * Scale hover animation - optimized for smooth performance
 */
export const scaleHoverIn = (element: gsap.TweenTarget, scale: number = 1.02) => {
  return gsap.to(element, {
    scale,
    duration: ANIMATION_TIMING.quick,
    ease: EASING.smoothOut,
    force3D: true,
    willChange: "transform",
  });
};

export const scaleHoverOut = (element: gsap.TweenTarget) => {
  return gsap.to(element, {
    scale: 1,
    duration: ANIMATION_TIMING.quick,
    ease: EASING.smoothOut,
    force3D: true,
    clearProps: "willChange",
  });
};

// ============================================================================
// CUSTOM ANIMATIONS
// ============================================================================

/**
 * Magnetic hover effect - element follows cursor
 */
export const magneticHover = (
  element: HTMLElement,
  options: {
    strength?: number;
    speed?: number;
  } = {}
) => {
  const strength = options.strength ?? 0.3;
  const speed = options.speed ?? 0.4;

  const handleMouseMove = (e: MouseEvent) => {
    const rect = element.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    const deltaX = (e.clientX - centerX) * strength;
    const deltaY = (e.clientY - centerY) * strength;

    gsap.to(element, {
      x: deltaX,
      y: deltaY,
      duration: speed,
      ease: "power2.out",
      force3D: true,
    });
  };

  const handleMouseLeave = () => {
    gsap.to(element, {
      x: 0,
      y: 0,
      duration: speed,
      ease: "elastic.out(1, 0.3)",
      force3D: true,
    });
  };

  element.addEventListener("mousemove", handleMouseMove);
  element.addEventListener("mouseleave", handleMouseLeave);

  return () => {
    element.removeEventListener("mousemove", handleMouseMove);
    element.removeEventListener("mouseleave", handleMouseLeave);
  };
};

/**
 * Glitch effect animation
 */
export const glitchEffect = (element: gsap.TweenTarget) => {
  const tl = gsap.timeline();

  tl.to(element, { x: -2, duration: 0.05, ease: "power2.in" })
    .to(element, { x: 2, skewX: 2, duration: 0.05 })
    .to(element, { x: -2, skewX: -2, duration: 0.05 })
    .to(element, { x: 2, duration: 0.05 })
    .to(element, { x: 0, skewX: 0, duration: 0.1, ease: "power2.out" });

  return tl;
};

/**
 * Particle burst effect
 */
export const particleBurst = (
  container: HTMLElement,
  options: {
    count?: number;
    colors?: string[];
    duration?: number;
  } = {}
) => {
  const count = options.count ?? 12;
  const colors = options.colors ?? ["#6366f1", "#8b5cf6", "#a855f7", "#c084fc"];
  const duration = options.duration ?? 1.2;

  const tl = gsap.timeline({
    onComplete: () => {
      particles.forEach((p) => p.remove());
    },
  });

  const particles: HTMLDivElement[] = [];

  for (let i = 0; i < count; i++) {
    const particle = document.createElement("div");
    particle.style.cssText = `
      position: absolute;
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: ${colors[i % colors.length]};
      pointer-events: none;
      top: 50%;
      left: 50%;
    `;
    container.appendChild(particle);
    particles.push(particle);

    const angle = (360 / count) * i;
    const distance = 60 + Math.random() * 40;
    const x = Math.cos((angle * Math.PI) / 180) * distance;
    const y = Math.sin((angle * Math.PI) / 180) * distance;

    tl.to(
      particle,
      {
        x,
        y,
        opacity: 0,
        scale: 0.5 + Math.random() * 0.5,
        duration,
        ease: "power2.out",
        force3D: true,
      },
      0
    );
  }

  return tl;
};

/**
 * Ripple effect from center
 */
export const rippleEffect = (element: gsap.TweenTarget) => {
  const tl = gsap.timeline();

  tl.fromTo(
    element,
    { scale: 0.8, opacity: 0 },
    {
      scale: 1,
      opacity: 1,
      duration: 0.4,
      ease: "back.out(2)",
      force3D: true,
    }
  ).to(
    element,
    {
      boxShadow: "0 0 0 20px rgba(99, 102, 241, 0)",
      duration: 0.6,
      ease: "power2.out",
    },
    0.2
  );

  return tl;
};

/**
 * Smooth reveal with blur
 */
export const blurReveal = (
  element: gsap.TweenTarget,
  options: { delay?: number; duration?: number } = {}
) => {
  return gsap.fromTo(
    element,
    {
      opacity: 0,
      filter: "blur(20px)",
      scale: 1.1,
    },
    {
      opacity: 1,
      filter: "blur(0px)",
      scale: 1,
      duration: options.duration ?? ANIMATION_TIMING.slow,
      ease: "power3.out",
      delay: options.delay ?? 0,
      force3D: true,
    }
  );
};

/**
 * Text reveal with split
 */
export const textReveal = (
  element: HTMLElement,
  options: { stagger?: number; duration?: number } = {}
) => {
  const text = element.textContent || "";
  const chars = text.split("");

  element.innerHTML = chars
    .map((char) => `<span style="display:inline-block;">${char === " " ? "&nbsp;" : char}</span>`)
    .join("");

  const spans = element.querySelectorAll("span");

  return gsap.fromTo(
    spans,
    {
      opacity: 0,
      y: 20,
      rotateX: -90,
    },
    {
      opacity: 1,
      y: 0,
      rotateX: 0,
      duration: options.duration ?? 0.6,
      ease: "back.out(1.7)",
      stagger: options.stagger ?? 0.03,
      force3D: true,
    }
  );
};

/**
 * 3D flip animation
 */
export const flip3D = (
  element: gsap.TweenTarget,
  options: { direction?: "x" | "y"; degrees?: number; duration?: number } = {}
) => {
  const direction = options.direction ?? "y";
  const degrees = options.degrees ?? 180;
  const prop = direction === "x" ? "rotateX" : "rotateY";

  return gsap.fromTo(
    element,
    {
      [prop]: 0,
      opacity: 1,
    },
    {
      [prop]: degrees,
      duration: options.duration ?? ANIMATION_TIMING.standard,
      ease: "power2.inOut",
      force3D: true,
      transformPerspective: 1000,
    }
  );
};

/**
 * Elastic bounce in - optimized for smooth performance
 */
export const elasticBounceIn = (
  element: gsap.TweenTarget,
  options: { delay?: number; from?: "top" | "bottom" | "left" | "right" } = {}
) => {
  const from = options.from ?? "bottom";
  const initialProps: gsap.TweenVars = { opacity: 0, scale: 0.92 };

  switch (from) {
    case "top":
      initialProps.y = -30;
      break;
    case "bottom":
      initialProps.y = 30;
      break;
    case "left":
      initialProps.x = -30;
      break;
    case "right":
      initialProps.x = 30;
      break;
  }

  return gsap.fromTo(element, initialProps, {
    opacity: 1,
    scale: 1,
    x: 0,
    y: 0,
    duration: ANIMATION_TIMING.moderate,
    ease: EASING.bounceSoft,
    delay: options.delay ?? 0,
    force3D: true,
    willChange: "transform, opacity",
  });
};

/**
 * Wobble animation
 */
export const wobble = (element: gsap.TweenTarget) => {
  const tl = gsap.timeline();

  tl.to(element, { rotation: -5, duration: 0.1, ease: "power1.inOut" })
    .to(element, { rotation: 5, duration: 0.2, ease: "power1.inOut" })
    .to(element, { rotation: -3, duration: 0.15, ease: "power1.inOut" })
    .to(element, { rotation: 3, duration: 0.15, ease: "power1.inOut" })
    .to(element, { rotation: 0, duration: 0.2, ease: "elastic.out(1, 0.5)" });

  return tl;
};

/**
 * Glow pulse animation
 */
export const glowPulse = (
  element: gsap.TweenTarget,
  options: { color?: string; intensity?: number } = {}
) => {
  const color = options.color ?? "139, 92, 246";
  const intensity = options.intensity ?? 20;

  return gsap.to(element, {
    boxShadow: `0 0 ${intensity}px rgba(${color}, 0.8), 0 0 ${intensity * 2}px rgba(${color}, 0.4)`,
    duration: ANIMATION_TIMING.pulse / 2,
    ease: "sine.inOut",
    repeat: -1,
    yoyo: true,
  });
};

/**
 * Smooth morph between states
 */
export const smoothMorph = (
  element: gsap.TweenTarget,
  toProps: gsap.TweenVars,
  options: { duration?: number; ease?: string } = {}
) => {
  return gsap.to(element, {
    ...toProps,
    duration: options.duration ?? ANIMATION_TIMING.standard,
    ease: options.ease ?? "power3.inOut",
    force3D: true,
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

// ============================================================================
// ADVANCED CREATIVE EFFECTS - Bespoke & Original
// ============================================================================

/**
 * Liquid morph effect - smooth organic transformation
 */
export const liquidMorph = (
  element: gsap.TweenTarget,
  options: {
    duration?: number;
    intensity?: number;
  } = {}
) => {
  const intensity = options.intensity ?? 15;
  const tl = gsap.timeline();

  tl.to(element, {
    filter: `blur(${intensity * 0.3}px) contrast(1.2)`,
    scale: 1.05,
    duration: options.duration ? options.duration * 0.3 : 0.3,
    ease: "power2.out",
  })
    .to(element, {
      filter: `blur(${intensity * 0.6}px) contrast(1.3) saturate(1.2)`,
      scale: 0.98,
      skewX: 2,
      duration: options.duration ? options.duration * 0.4 : 0.4,
      ease: "sine.inOut",
    })
    .to(element, {
      filter: "blur(0px) contrast(1) saturate(1)",
      scale: 1,
      skewX: 0,
      duration: options.duration ? options.duration * 0.3 : 0.3,
      ease: "power3.out",
    });

  return tl;
};

/**
 * Chromatic aberration effect - RGB split like a glitch
 */
export const chromaticAberration = (
  element: gsap.TweenTarget,
  options: { intensity?: number; duration?: number } = {}
) => {
  const intensity = options.intensity ?? 8;
  const tl = gsap.timeline();

  tl.to(element, {
    filter: `drop-shadow(${intensity}px 0 0 rgba(255, 0, 0, 0.8)) drop-shadow(-${intensity}px 0 0 rgba(0, 255, 255, 0.8))`,
    duration: 0.1,
    ease: "power2.in",
  })
    .to(element, {
      filter: `drop-shadow(${intensity * 1.5}px 0 0 rgba(255, 0, 0, 0.6)) drop-shadow(-${intensity * 1.5}px 0 0 rgba(0, 255, 255, 0.6))`,
      duration: 0.08,
      ease: "none",
    })
    .to(element, {
      filter: "drop-shadow(0px 0 0 rgba(255, 0, 0, 0)) drop-shadow(0px 0 0 rgba(0, 255, 255, 0))",
      duration: options.duration ?? 0.4,
      ease: "power3.out",
    });

  return tl;
};

/**
 * Perspective flip with depth - 3D card flip effect
 */
export const perspectiveFlip = (
  element: gsap.TweenTarget,
  options: {
    direction?: "x" | "y";
    distance?: number;
    duration?: number;
  } = {}
) => {
  const direction = options.direction ?? "y";
  const distance = options.distance ?? 1200;
  const tl = gsap.timeline();

  // Set perspective container
  gsap.set(element, {
    transformPerspective: distance,
    transformStyle: "preserve-3d",
  });

  tl.to(element, {
    [direction === "x" ? "rotateX" : "rotateY"]: 90,
    z: -100,
    opacity: 0.8,
    duration: (options.duration ?? 0.8) * 0.5,
    ease: "power2.in",
  }).to(element, {
    [direction === "x" ? "rotateX" : "rotateY"]: 180,
    z: 0,
    opacity: 1,
    duration: (options.duration ?? 0.8) * 0.5,
    ease: "power2.out",
  });

  return tl;
};

/**
 * Organic breathing animation - subtle, natural pulsing
 */
export const organicBreathe = (
  element: gsap.TweenTarget,
  options: {
    scale?: number;
    duration?: number;
  } = {}
) => {
  return gsap.to(element, {
    scale: options.scale ?? 1.03,
    filter: "brightness(1.08) saturate(1.1)",
    duration: (options.duration ?? 3) / 2,
    ease: "sine.inOut",
    repeat: -1,
    yoyo: true,
    force3D: true,
  });
};

/**
 * Kinetic typography - letters scatter and reform
 */
export const kineticType = (
  element: HTMLElement,
  options: {
    scatter?: number;
    duration?: number;
    stagger?: number;
  } = {}
) => {
  const text = element.textContent || "";
  const chars = text.split("");

  element.innerHTML = chars
    .map(
      (char) =>
        `<span style="display:inline-block;white-space:pre;">${char === " " ? "&nbsp;" : char}</span>`
    )
    .join("");

  const spans = element.querySelectorAll("span");
  const scatter = options.scatter ?? 100;
  const tl = gsap.timeline();

  // Scatter
  tl.to(spans, {
    x: () => (Math.random() - 0.5) * scatter,
    y: () => (Math.random() - 0.5) * scatter,
    rotation: () => (Math.random() - 0.5) * 180,
    opacity: 0,
    duration: 0.01,
  })
    // Reform
    .to(spans, {
      x: 0,
      y: 0,
      rotation: 0,
      opacity: 1,
      duration: options.duration ?? 1.2,
      ease: "elastic.out(1, 0.5)",
      stagger: {
        each: options.stagger ?? 0.03,
        from: "random",
      },
      force3D: true,
    });

  return tl;
};

/**
 * Ink blot reveal - organic spreading effect
 */
export const inkBlotReveal = (
  element: gsap.TweenTarget,
  options: { duration?: number; from?: "center" | "corner" } = {}
) => {
  const tl = gsap.timeline();
  const from = options.from ?? "center";

  tl.fromTo(
    element,
    {
      clipPath: from === "center" ? "circle(0% at 50% 50%)" : "circle(0% at 0% 0%)",
      opacity: 0,
      scale: 1.2,
      filter: "blur(20px) contrast(0.8)",
    },
    {
      clipPath: "circle(100% at 50% 50%)",
      opacity: 1,
      scale: 1,
      filter: "blur(0px) contrast(1)",
      duration: options.duration ?? 1.2,
      ease: "power3.out",
      force3D: true,
    }
  );

  return tl;
};

/**
 * Holographic shimmer - futuristic iridescent effect
 */
export const holographicShimmer = (
  element: gsap.TweenTarget,
  options: { duration?: number } = {}
) => {
  const tl = gsap.timeline({ repeat: -1 });

  tl.to(element, {
    filter: "hue-rotate(0deg) saturate(1.3) brightness(1.1)",
    duration: 0,
  })
    .to(element, {
      filter: "hue-rotate(180deg) saturate(1.5) brightness(1.2)",
      duration: (options.duration ?? 4) / 2,
      ease: "sine.inOut",
    })
    .to(element, {
      filter: "hue-rotate(360deg) saturate(1.3) brightness(1.1)",
      duration: (options.duration ?? 4) / 2,
      ease: "sine.inOut",
    });

  return tl;
};

/**
 * Quantum flicker - subtle reality-bending effect
 */
export const quantumFlicker = (element: gsap.TweenTarget, options: { intensity?: number } = {}) => {
  const intensity = options.intensity ?? 0.05;

  return gsap.to(element, {
    opacity: 1 - intensity,
    x: () => (Math.random() - 0.5) * 2,
    y: () => (Math.random() - 0.5) * 2,
    filter: "blur(0.5px)",
    duration: 0.05,
    repeat: -1,
    yoyo: true,
    ease: "none",
  });
};

/**
 * Elastic morph - bouncy shape transformation
 */
export const elasticMorph = (
  element: gsap.TweenTarget,
  toProps: { scaleX?: number; scaleY?: number; skewX?: number; skewY?: number },
  options: { duration?: number } = {}
) => {
  const tl = gsap.timeline();

  tl.to(element, {
    ...toProps,
    duration: (options.duration ?? 0.8) * 0.6,
    ease: "elastic.out(1, 0.4)",
    force3D: true,
  }).to(element, {
    scaleX: 1,
    scaleY: 1,
    skewX: 0,
    skewY: 0,
    duration: (options.duration ?? 0.8) * 0.4,
    ease: "elastic.out(1, 0.6)",
  });

  return tl;
};

/**
 * Vortex spiral in - element spirals into existence
 */
export const vortexSpiral = (
  element: gsap.TweenTarget,
  options: { duration?: number; clockwise?: boolean } = {}
) => {
  const direction = options.clockwise ? 1 : -1;

  return gsap.fromTo(
    element,
    {
      opacity: 0,
      scale: 0,
      rotation: direction * 720,
      filter: "blur(20px)",
    },
    {
      opacity: 1,
      scale: 1,
      rotation: 0,
      filter: "blur(0px)",
      duration: options.duration ?? 1,
      ease: "power4.out",
      force3D: true,
    }
  );
};

/**
 * Energy pulse - radiating waves effect
 */
export const energyPulse = (
  element: gsap.TweenTarget,
  options: { color?: string; count?: number } = {}
) => {
  const color = options.color ?? "139, 92, 246";
  const tl = gsap.timeline({ repeat: options.count ? options.count - 1 : -1 });

  tl.fromTo(
    element,
    {
      boxShadow: `0 0 0 0 rgba(${color}, 0.7)`,
    },
    {
      boxShadow: `0 0 0 40px rgba(${color}, 0), 0 0 0 80px rgba(${color}, 0)`,
      duration: 1.5,
      ease: "power2.out",
    }
  );

  return tl;
};

// ============================================================================
// REACT-SPRING CONFIGURATION PRESETS
// Physics-based animation configs for natural motion
// ============================================================================

export const springConfigs = {
  // Gentle, subtle spring
  gentle: { tension: 120, friction: 14, precision: 0.01 },

  // Default spring - balanced
  default: { tension: 170, friction: 26, precision: 0.01 },

  // Slow, smooth spring
  slow: { tension: 100, friction: 30, precision: 0.01 },

  // Fast, snappy spring
  fast: { tension: 300, friction: 30, precision: 0.01 },

  // Bouncy, playful spring
  bouncy: { tension: 200, friction: 12, precision: 0.01 },

  // Molasses - very slow
  molasses: { tension: 50, friction: 20, precision: 0.01 },

  // Stiff - minimal oscillation
  stiff: { tension: 300, friction: 40, precision: 0.01 },

  // Wobbly - lots of oscillation
  wobbly: { tension: 180, friction: 8, precision: 0.01 },

  // Rubber band effect
  rubber: { tension: 150, friction: 10, clamp: false, precision: 0.01 },

  // Jello - very bouncy
  jello: { tension: 200, friction: 6, precision: 0.01 },
};

// Pre-configured animation states for common patterns
export const springStates = {
  fadeIn: { opacity: 1, scale: 1 },
  fadeOut: { opacity: 0, scale: 0.95 },
  slideInFromLeft: { x: 0, opacity: 1 },
  slideInFromRight: { x: 0, opacity: 1 },
  slideInFromTop: { y: 0, opacity: 1 },
  slideInFromBottom: { y: 0, opacity: 1 },
  scaleIn: { scale: 1, opacity: 1 },
  scaleOut: { scale: 0, opacity: 0 },
  rotate: { rotate: 0 },
  reset: { x: 0, y: 0, scale: 1, opacity: 1, rotate: 0 },
};
