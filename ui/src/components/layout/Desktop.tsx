/**
 * Desktop Component
 * Main OS desktop with app shortcuts and system tray
 */

import React, { useEffect, useState } from "react";
import "./Desktop.css";

interface DesktopApp {
  id: string;
  name: string;
  icon: string;
  category: string;
}

interface DesktopProps {
  onLaunchApp: (appId: string) => void;
  onOpenHub: () => void;
  onOpenCreator: () => void;
}

export const Desktop: React.FC<DesktopProps> = ({
  onLaunchApp,
  onOpenHub,
  onOpenCreator,
}) => {
  const [apps, setApps] = useState<DesktopApp[]>([]);
  const [time, setTime] = useState(new Date());

  // Load pinned apps
  useEffect(() => {
    const loadApps = async () => {
      try {
        const response = await fetch("http://localhost:8000/registry/apps");
        const data = await response.json();
        // Show first 6 apps on desktop
        const pinnedApps = (data.apps || []).slice(0, 6).map((app: any) => ({
          id: app.id,
          name: app.name,
          icon: app.icon,
          category: app.category,
        }));
        setApps(pinnedApps);
      } catch (error) {
        console.error("Failed to load apps:", error);
      }
    };
    loadApps();
  }, []);

  // Update clock
  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
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
            <div className="clock-date">{formatDate()}</div>
          </div>
        </div>
      </div>

      {/* Desktop Icons */}
      <div className="desktop-icons">
        {apps.map((app) => (
          <button
            key={app.id}
            className="desktop-icon"
            onClick={() => onLaunchApp(app.id)}
          >
            <div className="desktop-icon-symbol">{app.icon}</div>
            <div className="desktop-icon-name">{app.name}</div>
          </button>
        ))}
      </div>

      {/* Dock */}
      <div className="desktop-dock">
        <div className="dock-container">
          <button className="dock-item" onClick={onOpenHub} title="Hub">
            <span className="dock-icon">🚀</span>
          </button>
          <button
            className="dock-item"
            onClick={() => onLaunchApp("file-explorer")}
            title="Files"
          >
            <span className="dock-icon">📁</span>
          </button>
          <button
            className="dock-item"
            onClick={() => onLaunchApp("calculator")}
            title="Calculator"
          >
            <span className="dock-icon">🧮</span>
          </button>
          <button
            className="dock-item"
            onClick={() => onLaunchApp("notes")}
            title="Notes"
          >
            <span className="dock-icon">📝</span>
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

