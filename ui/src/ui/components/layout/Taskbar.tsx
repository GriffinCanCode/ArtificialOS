/**
 * Taskbar Component
 * Shows open and minimized windows with sortable reordering
 */

import React from "react";
import { useStore, useActions } from "../../../features/windows";
import { Sortable } from "../../../features/dnd";
import type { SortResult } from "../../../features/dnd";
import "./Taskbar.css";

export const Taskbar: React.FC = () => {
  const windows = useStore((state) => state.windows);
  const { restore, focus } = useActions();

  const handleWindowClick = (windowId: string, isMinimized: boolean) => {
    if (isMinimized) {
      restore(windowId);
    } else {
      focus(windowId);
    }
  };

  const handleSort = (result: SortResult) => {
    // Optional: Add store action to persist window order if needed
    console.log("Windows reordered:", result);
  };

  if (windows.length === 0) {
    return null;
  }

  // Convert windows to sortable items
  const sortableWindows = windows.map((w) => ({ ...w, id: w.id }));

  return (
    <div className="taskbar">
      <Sortable
        items={sortableWindows}
        onSort={handleSort}
        strategy="horizontal"
        renderItem={(window) => (
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
            {window.isMinimized && <span className="taskbar-item-indicator">●</span>}
          </button>
        )}
        className="taskbar-container"
      />
    </div>
  );
};
