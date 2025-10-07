/**
 * BrandLogo Component
 * Sophisticated branded logo with custom typography
 */

import React from "react";
import "./BrandLogo.css";

export interface BrandLogoProps {
  size?: "small" | "medium" | "large";
  animated?: boolean;
  className?: string;
  onClick?: () => void;
}

export const BrandLogo: React.FC<BrandLogoProps> = ({
  size = "medium",
  animated = true,
  className = "",
  onClick,
}) => {
  const sizeClass = `brand-logo-${size}`;
  const animatedClass = animated ? "brand-logo-animated" : "";

  return (
    <button
      className={`brand-logo ${sizeClass} ${animatedClass} ${className}`}
      onClick={onClick}
      aria-label="AgentOS"
    >
      <span className="brand-logo-icon">✨</span>
      <span className="brand-logo-text">
        <span className="brand-logo-agent">Agent</span>
        <span className="brand-logo-os">OS</span>
      </span>
    </button>
  );
};
