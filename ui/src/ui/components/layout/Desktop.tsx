/**
 * Desktop Component
 * Main OS desktop with menu bar and sortable dock
 */

import React, { useEffect, useState, useCallback } from "react";
import { formatDate, formatTime } from "../../../core/utils/dates";
import { Sortable, useDockItems, useDockActions } from "../../../features/dnd";
import type { SortResult } from "../../../features/dnd";
import { Tooltip } from "../../../features/floating";
import { BrandLogo } from "../typography/BrandLogo";
import { DockItem } from "./DockItem";
import { Grid as IconGrid } from "../../../features/icons";
import type { IconType } from "../../../features/icons";
import { useScope, useShortcuts } from "../../../features/input";
import "./Desktop.css";

interface DesktopProps {
  onLaunchApp: (appId: string) => void;
  onOpenHub: () => void;
  onOpenCreator: () => void;
  onOpenAbout: () => void;
}

export const Desktop: React.FC<DesktopProps> = ({ onLaunchApp, onOpenHub, onOpenCreator, onOpenAbout }) => {
  const [time, setTime] = useState(new Date());
  const dockItems = useDockItems();
  const { reorder, toggle, remove } = useDockActions();

  // Activate desktop scope
  useScope("desktop");

  // Register desktop shortcuts
  // Note: Cmd+K is handled globally in App.tsx
  useShortcuts([
    {
      id: "desktop.hub.open",
      sequence: "$mod+Space",
      label: "Open Hub",
      description: "Open the application hub",
      category: "system",
      scope: "desktop",
      priority: "high",
      handler: () => onOpenHub(),
    },
  ]);

  // Update clock
  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  // Handle dock reorder
  const handleDockSort = (result: SortResult) => {
    reorder(result.oldIndex, result.newIndex);
  };

  // Handle dock item click
  const handleDockItemClick = useCallback(
    (action: string) => {
      if (action === "hub") {
        onOpenHub();
      } else {
        onLaunchApp(action);
      }
    },
    [onOpenHub, onLaunchApp]
  );

  // Handle context menu actions
  const handleDockContextMenu = useCallback(
    (action: string, itemId: string) => {
      switch (action) {
        case "toggle-pin":
          toggle(itemId);
          break;
        case "remove":
          remove(itemId);
          break;
      }
    },
    [toggle, remove]
  );

  // Handle icon double-click (launch app)
  const handleIconDoubleClick = useCallback(
    (icon: IconType) => {
      if (icon.type === "app" && icon.metadata.type === "app") {
        onLaunchApp(icon.metadata.appId);
      } else if (icon.type === "native" && icon.metadata.type === "native") {
        onLaunchApp(icon.metadata.packageId);
      }
    },
    [onLaunchApp]
  );

  return (
    <div className="desktop">
      {/* Desktop Background */}
      <div className="desktop-background">
        <div className="desktop-gradient" />
      </div>

      {/* Desktop Icons Grid */}
      <IconGrid
        onIconDoubleClick={handleIconDoubleClick}
        onBackgroundClick={() => {}}
      />

      {/* Top Menu Bar */}
      <div className="desktop-menubar">
        <div className="menubar-left">
          <BrandLogo size="small" onClick={onOpenAbout} />
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
            <DockItem
              key={item.id}
              item={item}
              onClick={handleDockItemClick}
              onContextMenuAction={handleDockContextMenu}
            />
          )}
          className="dock-container"
        />
        <div className="dock-separator" />
        <Tooltip content="Create (⌘K)" delay={500}>
          <button className="dock-item dock-creator" onClick={onOpenCreator} aria-label="Create">
            <span className="dock-icon">✨</span>
          </button>
        </Tooltip>
      </div>
    </div>
  );
};
