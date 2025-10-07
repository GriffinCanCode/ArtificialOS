/**
 * WindowTitleBar Component
 * Styled window title with typography effects
 */

import React from "react";
import "./WindowTitleBar.css";

export interface WindowTitleBarProps {
  title: string;
  icon?: string;
  active?: boolean;
  className?: string;
}

export const WindowTitleBar: React.FC<WindowTitleBarProps> = ({
  title,
  icon,
  active = false,
  className = "",
}) => {
  return (
    <div className={`window-title-bar ${active ? "active" : ""} ${className}`}>
      {icon && <span className="window-title-icon">{icon}</span>}
      <span className="window-title-text">
        {title.split("").map((char, index) => (
          <span
            key={index}
            className="window-title-char"
            style={{
              animationDelay: `${index * 0.02}s`,
            }}
          >
            {char === " " ? "\u00A0" : char}
          </span>
        ))}
      </span>
    </div>
  );
};
