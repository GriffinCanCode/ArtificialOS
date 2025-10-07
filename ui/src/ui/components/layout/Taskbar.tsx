/**
 * Taskbar Component
 * Shows open and minimized windows with sortable reordering
 * Optimized for performance with many windows
 */

import React, { useMemo, useCallback } from "react";
import { useStore, useActions } from "../../../features/windows";
import { Sortable } from "../../../features/dnd";
import type { SortResult } from "../../../features/dnd";
import "./Taskbar.css";

// Maximum number of taskbar items to show before truncating
const MAX_VISIBLE_ITEMS = 12;

// Memoized taskbar item component to prevent unnecessary re-renders
const TaskbarItem = React.memo<{
  title: string;
  icon?: string;
  isFocused: boolean;
  isMinimized: boolean;
  onClick: () => void;
}>(({ title, icon, isFocused, isMinimized, onClick }) => (
  <button
    className={`taskbar-item ${isFocused ? "active" : ""} ${isMinimized ? "minimized" : ""}`}
    onClick={onClick}
    title={title}
  >
    {icon && <span className="taskbar-item-icon">{icon}</span>}
    <span className="taskbar-item-title">{title}</span>
    {isMinimized && <span className="taskbar-item-indicator">●</span>}
  </button>
));

TaskbarItem.displayName = "TaskbarItem";

export const Taskbar: React.FC = () => {
  const windows = useStore((state) => state.windows);
  const { restore, focus } = useActions();

  const handleWindowClick = useCallback(
    (windowId: string, isMinimized: boolean) => {
      if (isMinimized) {
        restore(windowId);
      } else {
        focus(windowId);
      }
    },
    [restore, focus]
  );

  const handleSort = useCallback((result: SortResult) => {
    // Optional: Add store action to persist window order if needed
    console.log("Windows reordered:", result);
  }, []);

  // Limit visible windows to prevent memory issues
  const visibleWindows = useMemo(() => {
    if (windows.length <= MAX_VISIBLE_ITEMS) {
      return windows;
    }
    // Show focused/active windows and most recent non-minimized ones
    const focused = windows.filter((w) => w.isFocused);
    const nonMinimized = windows.filter((w) => !w.isMinimized && !w.isFocused);
    const minimized = windows.filter((w) => w.isMinimized);

    const visible = [
      ...focused,
      ...nonMinimized.slice(0, MAX_VISIBLE_ITEMS - focused.length - 1),
      ...minimized.slice(0, 1), // Show at least one minimized as indicator
    ];

    return visible.slice(0, MAX_VISIBLE_ITEMS);
  }, [windows]);

  // Convert windows to sortable items
  const sortableWindows = useMemo(
    () => visibleWindows.map((w) => ({ ...w, id: w.id })),
    [visibleWindows]
  );

  // Early return AFTER all hooks have been called
  if (windows.length === 0) {
    return null;
  }

  const hasMoreWindows = windows.length > MAX_VISIBLE_ITEMS;
  const hiddenCount = windows.length - visibleWindows.length;

  return (
    <div className="taskbar">
      <Sortable
        items={sortableWindows}
        onSort={handleSort}
        strategy="horizontal"
        renderItem={(window) => (
          <TaskbarItem
            key={window.id}
            title={window.title}
            icon={window.icon}
            isFocused={window.isFocused}
            isMinimized={window.isMinimized}
            onClick={() => handleWindowClick(window.id, window.isMinimized)}
          />
        )}
        className="taskbar-container"
      />
      {hasMoreWindows && (
        <div
          className="taskbar-overflow"
          title={`${hiddenCount} more window${hiddenCount > 1 ? "s" : ""}`}
        >
          +{hiddenCount}
        </div>
      )}
    </div>
  );
};
