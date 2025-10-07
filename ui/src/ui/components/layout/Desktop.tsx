/**
 * Desktop Component
 * Main OS desktop with menu bar and sortable dock
 */

import React, { useEffect, useState } from "react";
import { shouldIgnoreKeyboardEvent } from "../../../features/input";
import { formatDate, formatTime } from "../../../core/utils/dates";
import { Sortable, useDockItems, useDockActions } from "../../../features/dnd";
import type { SortResult } from "../../../features/dnd";
import { DockItem } from "./DockItem";
import "./Desktop.css";

interface DesktopProps {
  onLaunchApp: (appId: string) => void;
  onOpenHub: () => void;
  onOpenCreator: () => void;
}

export const Desktop: React.FC<DesktopProps> = ({ onLaunchApp, onOpenHub, onOpenCreator }) => {
  const [time, setTime] = useState(new Date());
  const dockItems = useDockItems();
  const { reorder } = useDockActions();

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

  // Handle dock reorder
  const handleDockSort = (result: SortResult) => {
    reorder(result.oldIndex, result.newIndex);
  };

  // Handle dock item click
  const handleDockItemClick = (action: string) => {
    if (action === "hub") {
      onOpenHub();
    } else {
      onLaunchApp(action);
    }
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
            <div className="clock-time">{formatTime(time, false)}</div>
            <div className="clock-separator">•</div>
            <div className="clock-date">{formatDate(time, "EEE, MMM d")}</div>
          </div>
        </div>
      </div>

      {/* Dock - Sortable */}
      <div className="desktop-dock">
        <Sortable
          items={dockItems}
          onSort={handleDockSort}
          strategy="horizontal"
          renderItem={(item) => (
            <DockItem key={item.id} item={item} onClick={handleDockItemClick} />
          )}
          className="dock-container"
        />
        <div className="dock-separator" />
        <button className="dock-item dock-creator" onClick={onOpenCreator} title="Create (⌘K)">
          <span className="dock-icon">✨</span>
        </button>
      </div>

      {/* Hint Overlay */}
      <div className="desktop-hint">
        Press <kbd>⌘K</kbd> to create something
      </div>
    </div>
  );
};
