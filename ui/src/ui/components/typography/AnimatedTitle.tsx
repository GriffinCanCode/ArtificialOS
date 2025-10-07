/**
 * AnimatedTitle Component
 * Advanced animated title with various effects
 */

import React, { useEffect, useState } from "react";
import { CustomText } from "./CustomText";
import "./AnimatedTitle.css";

export interface AnimatedTitleProps {
  text: string;
  fontName?: string;
  preset?: "hero" | "title" | "heading" | "display";
  effect?: "gradient" | "glow" | "shimmer" | "wave" | "typewriter" | "split";
  className?: string;
  style?: React.CSSProperties;
  animationDelay?: number;
}

export const AnimatedTitle: React.FC<AnimatedTitleProps> = ({
  text,
  fontName = "system",
  preset = "title",
  effect = "gradient",
  className = "",
  style = {},
  animationDelay = 0,
}) => {
  const [displayText, setDisplayText] = useState(effect === "typewriter" ? "" : text);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    // Delay initial animation
    const timer = setTimeout(() => {
      setIsVisible(true);
    }, animationDelay);

    return () => clearTimeout(timer);
  }, [animationDelay]);

  useEffect(() => {
    if (effect === "typewriter" && isVisible) {
      let index = 0;
      const interval = setInterval(() => {
        if (index <= text.length) {
          setDisplayText(text.slice(0, index));
          index++;
        } else {
          clearInterval(interval);
        }
      }, 100);

      return () => clearInterval(interval);
    }
  }, [text, effect, isVisible]);

  // Gradient configurations
  const gradients = {
    gradient: {
      colors: ["#8B5CF6", "#EC4899", "#EF4444", "#F59E0B"],
      angle: 45,
    },
    glow: {
      colors: ["#60A5FA", "#A78BFA", "#F472B6"],
      angle: 90,
    },
    shimmer: {
      colors: ["#FCD34D", "#FBBF24", "#F59E0B", "#FBBF24", "#FCD34D"],
      angle: 135,
    },
  };

  // Apply effect
  const effectProps =
    effect === "gradient" || effect === "glow" || effect === "shimmer"
      ? { gradient: gradients[effect] }
      : {};

  const glowProps =
    effect === "glow"
      ? {
          glow: {
            color: "#A78BFA",
            intensity: 6,
          },
        }
      : {};

  const shadowProps =
    effect === "gradient"
      ? {
          shadow: {
            color: "rgba(139, 92, 246, 0.3)",
            blur: 20,
            offsetY: 4,
          },
        }
      : {};

  if (effect === "split") {
    // Split each character for wave animation
    return (
      <div className={`animated-title split ${className}`} style={style}>
        {text.split("").map((char, index) => (
          <span
            key={index}
            className="char"
            style={{
              animationDelay: `${index * 0.05 + animationDelay / 1000}s`,
            }}
          >
            {char === " " ? "\u00A0" : char}
          </span>
        ))}
      </div>
    );
  }

  if (effect === "wave") {
    return (
      <div className={`animated-title wave ${className}`} style={style}>
        {text.split("").map((char, index) => (
          <span
            key={index}
            className="char"
            style={{
              animationDelay: `${index * 0.1 + animationDelay / 1000}s`,
            }}
          >
            {char === " " ? "\u00A0" : char}
          </span>
        ))}
      </div>
    );
  }

  return (
    <div
      className={`animated-title ${effect} ${isVisible ? "visible" : ""} ${className}`}
      style={style}
    >
      <CustomText
        text={displayText}
        fontName={fontName}
        preset={preset}
        animate={isVisible ? "fadeIn" : "none"}
        animationDuration={0.8}
        {...effectProps}
        {...glowProps}
        {...shadowProps}
      />
    </div>
  );
};
