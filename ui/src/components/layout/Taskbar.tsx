/**
 * Taskbar Component
 * Shows open and minimized windows, allows restoring
 */

import React from "react";
import { useWindows, useWindowActions } from "../../store/windowStore";
import "./Taskbar.css";

export const Taskbar: React.FC = () => {
  const windows = useWindows();
  const { restoreWindow, focusWindow } = useWindowActions();

  const handleWindowClick = (windowId: string, isMinimized: boolean) => {
    if (isMinimized) {
      restoreWindow(windowId);
    } else {
      focusWindow(windowId);
    }
  };

  if (windows.length === 0) {
    return null;
  }

  return (
    <div className="taskbar">
      <div className="taskbar-container">
        {windows.map((window) => (
          <button
            key={window.id}
            className={`taskbar-item ${window.isFocused ? "active" : ""} ${
              window.isMinimized ? "minimized" : ""
            }`}
            onClick={() => handleWindowClick(window.id, window.isMinimized)}
            title={window.title}
          >
            {window.icon && <span className="taskbar-item-icon">{window.icon}</span>}
            <span className="taskbar-item-title">{window.title}</span>
            {window.isMinimized && (
              <span className="taskbar-item-indicator">●</span>
            )}
          </button>
        ))}
      </div>
    </div>
  );
};

