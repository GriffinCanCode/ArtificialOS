/**
 * Desktop Component
 * Main OS desktop with menu bar and dock
 */

import React, { useEffect, useState } from "react";
import { shouldIgnoreKeyboardEvent } from "../../../features/input";
import "./Desktop.css";

interface DesktopProps {
  onLaunchApp: (appId: string) => void;
  onOpenHub: () => void;
  onOpenCreator: () => void;
}

export const Desktop: React.FC<DesktopProps> = ({ onLaunchApp, onOpenHub, onOpenCreator }) => {
  const [time, setTime] = useState(new Date());

  // Update clock
  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't trigger shortcuts if user is typing in an input/textarea
      if (shouldIgnoreKeyboardEvent(e)) {
        return; // Let the input handle the keystroke
      }

      // Cmd+K or Ctrl+K to open creator
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        onOpenCreator();
      }
      // Cmd+Space to open hub
      if ((e.metaKey || e.ctrlKey) && e.key === " ") {
        e.preventDefault();
        onOpenHub();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onOpenCreator, onOpenHub]);

  const formatTime = () => {
    return time.toLocaleTimeString("en-US", {
      hour: "numeric",
      minute: "2-digit",
      hour12: true,
    });
  };

  const formatDate = () => {
    return time.toLocaleDateString("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
  };

  return (
    <div className="desktop">
      {/* Desktop Background */}
      <div className="desktop-background">
        <div className="desktop-gradient" />
      </div>

      {/* Top Menu Bar */}
      <div className="desktop-menubar">
        <div className="menubar-left">
          <button className="menubar-item logo" onClick={onOpenHub}>
            ✨ AgentOS
          </button>
          <button className="menubar-item" onClick={onOpenHub}>
            Hub
          </button>
        </div>
        <div className="menubar-right">
          <div className="menubar-clock">
            <div className="clock-time">{formatTime()}</div>
            <div className="clock-separator">•</div>
            <div className="clock-date">{formatDate()}</div>
          </div>
        </div>
      </div>

      {/* Dock */}
      <div className="desktop-dock">
        <div className="dock-container">
          <button className="dock-item" onClick={onOpenHub} title="Hub">
            <span className="dock-icon">🚀</span>
          </button>
          <button className="dock-item" onClick={() => onLaunchApp("file-explorer")} title="Files">
            <span className="dock-icon">📁</span>
          </button>
          <button className="dock-item" onClick={() => onLaunchApp("browser")} title="Browser">
            <span className="dock-icon">🌐</span>
          </button>
          <button
            className="dock-item"
            onClick={() => onLaunchApp("calculator")}
            title="Calculator"
          >
            <span className="dock-icon">🧮</span>
          </button>
          <button className="dock-item" onClick={() => onLaunchApp("notes")} title="Notes">
            <span className="dock-icon">📝</span>
          </button>
          <button
            className="dock-item"
            onClick={() => onLaunchApp("system-analysis")}
            title="System Analysis"
          >
            <span className="dock-icon">📊</span>
          </button>
          <div className="dock-separator" />
          <button className="dock-item" onClick={onOpenCreator} title="Create (⌘K)">
            <span className="dock-icon">✨</span>
          </button>
        </div>
      </div>

      {/* Hint Overlay */}
      <div className="desktop-hint">
        Press <kbd>⌘K</kbd> to create something
      </div>
    </div>
  );
};
