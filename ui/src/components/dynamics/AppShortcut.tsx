/**
 * AppShortcut Component
 * Displays an application icon/card that can be clicked to launch
 */

import React from "react";
import "./AppShortcut.css";

export interface AppShortcutProps {
  id: string;
  name: string;
  icon: string;
  description?: string;
  category?: string;
  variant?: "icon" | "card" | "list";
  onClick?: (appId: string) => void;
}

export const AppShortcut: React.FC<AppShortcutProps> = ({
  id,
  name,
  icon,
  description,
  category,
  variant = "card",
  onClick,
}) => {
  const handleClick = () => {
    if (onClick) {
      onClick(id);
    }
  };

  if (variant === "icon") {
    return (
      <div className="app-shortcut-icon" onClick={handleClick}>
        <div className="app-shortcut-icon-symbol">{icon}</div>
        <div className="app-shortcut-icon-name">{name}</div>
      </div>
    );
  }

  if (variant === "list") {
    return (
      <div className="app-shortcut-list" onClick={handleClick}>
        <div className="app-shortcut-list-icon">{icon}</div>
        <div className="app-shortcut-list-content">
          <div className="app-shortcut-list-name">{name}</div>
          {description && (
            <div className="app-shortcut-list-description">{description}</div>
          )}
        </div>
        {category && (
          <div className="app-shortcut-list-category">{category}</div>
        )}
      </div>
    );
  }

  // Card variant (default)
  return (
    <div className="app-shortcut-card" onClick={handleClick}>
      <div className="app-shortcut-card-icon">{icon}</div>
      <div className="app-shortcut-card-name">{name}</div>
      {description && (
        <div className="app-shortcut-card-description">{description}</div>
      )}
      {category && (
        <div className="app-shortcut-card-category">{category}</div>
      )}
    </div>
  );
};

