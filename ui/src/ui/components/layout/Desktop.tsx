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
import { Launchpad } from "./Launchpad";
import { Grid as IconGrid, useIconActions, useIcons as useIconsData, useSelectedIcons } from "../../../features/icons";
import type { IconType } from "../../../features/icons";
import { useScope, useShortcuts, createSelectionCommands } from "../../../features/input";
import { ConnectionStatus } from "../../../features/connection";
import "./Desktop.css";

interface DesktopProps {
  onLaunchApp: (appId: string) => void;
  onOpenHub: () => void;
  onOpenCreator: () => void;
  onOpenAbout: () => void;
  showLaunchpad: boolean;
  onToggleLaunchpad: () => void;
}

export const Desktop: React.FC<DesktopProps> = ({ onLaunchApp, onOpenHub, onOpenCreator, onOpenAbout, showLaunchpad, onToggleLaunchpad }) => {
  const [time, setTime] = useState(new Date());
  const dockItems = useDockItems();
  const { reorder, toggle, remove } = useDockActions();

  // Icon grid actions and data for wiring up selection commands
  const iconActions = useIconActions();
  const icons = useIconsData();
  const selectedIcons = useSelectedIcons();

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
    {
      id: "desktop.launchpad.toggle",
      sequence: "$mod+l",
      label: "Toggle Launchpad",
      description: "Show or hide the launchpad app grid",
      category: "system",
      scope: "desktop",
      priority: "high",
      handler: () => onToggleLaunchpad(),
    },
  ]);

  // Register selection commands using factory pattern
  // This establishes single source of truth while allowing context-specific actions
  useShortcuts(
    createSelectionCommands(
      {
        selectAll: iconActions.selectAll,
        clearSelection: iconActions.clearSelection,
        invertSelection: () => {
          // Invert selection logic for icon grid
          const allIds = icons.map((i: IconType) => i.id);
          const selectedIds = selectedIcons.map((i: IconType) => i.id);
          const selectedSet = new Set(selectedIds);
          const invertedIds = allIds.filter((id: string) => !selectedSet.has(id));

          iconActions.clearSelection();
          invertedIds.forEach((id: string) => iconActions.select(id, true));
        },
      },
      "desktop" // Scope for these shortcuts
    )
  );

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

      {/* Desktop Icons Grid - hidden when launchpad is visible */}
      <div className={`desktop-icons ${showLaunchpad ? "hidden" : ""}`}>
        <IconGrid
          onIconDoubleClick={handleIconDoubleClick}
          onBackgroundClick={() => {}}
        />
      </div>

      {/* Launchpad - replaces desktop icons when visible */}
      <Launchpad isVisible={showLaunchpad} onLaunchApp={onLaunchApp} />

      {/* Top Menu Bar */}
      <div className="desktop-menubar">
        <div className="menubar-left">
          <BrandLogo size="small" onClick={onOpenAbout} />
          <button className="menubar-item" onClick={onOpenHub}>
            Hub
          </button>
          <Tooltip content="Launchpad (⌘L)" delay={500}>
            <button
              className={`menubar-item menubar-launchpad ${showLaunchpad ? "active" : ""}`}
              onClick={onToggleLaunchpad}
            >
              <span className="launchpad-icon">⚡</span>
            </button>
          </Tooltip>
        </div>
        <div className="menubar-right">
          <ConnectionStatus />
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
