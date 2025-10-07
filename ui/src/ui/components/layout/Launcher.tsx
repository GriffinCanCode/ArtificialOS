/**
 * App Launcher Component
 * Grid-based launcher for installed apps from the registry
 */

import React, { useState, useCallback } from "react";
import { Rocket, Plus, AlertTriangle, Trash2, Play } from "lucide-react";
import { useRegistryApps, useRegistryMutations } from "../../../core/hooks/useRegistryQueries";
import {
  cardVariants,
  categoryButtonVariants,
  cn,
} from "../../../core/utils/animation/componentVariants";
import { Tooltip, ContextMenu } from "../../../features/floating";
import type { DropdownItem } from "../../../features/floating";
import "./Launcher.css";

interface LauncherProps {
  onAppLaunch?: (appId: string, uiSpec: Record<string, any>) => void;
  onCreateNew?: () => void;
}

export const Launcher: React.FC<LauncherProps> = React.memo(({ onAppLaunch, onCreateNew }) => {
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);

  // Use TanStack Query for data fetching with automatic caching
  const {
    data,
    isLoading,
    error: queryError,
    refetch,
  } = useRegistryApps(selectedCategory || undefined);

  // Use mutation hooks for app actions
  const { launchApp, deleteApp } = useRegistryMutations();

  const apps = data?.apps ?? [];
  const error = queryError?.message ?? (launchApp.error?.message || deleteApp.error?.message);

  const handleLaunchApp = useCallback(
    async (packageId: string) => {
      launchApp.mutate(packageId, {
        onSuccess: (response) => {
          if (onAppLaunch) {
            onAppLaunch(response.app_id, response.blueprint);
          }
        },
      });
    },
    [launchApp, onAppLaunch]
  );

  const handleDeleteApp = useCallback(
    async (packageId: string, event?: React.MouseEvent) => {
      if (event) {
        event.stopPropagation();
      }

      if (!confirm("Are you sure you want to delete this app?")) {
        return;
      }

      deleteApp.mutate(packageId);
    },
    [deleteApp]
  );

  const getAppContextMenuItems = useCallback(
    (_packageId: string): DropdownItem[] => [
      { value: "launch", label: "Launch", icon: <Play size={16} /> },
      { divider: true, value: "divider-1", label: "" },
      { value: "delete", label: "Delete", icon: <Trash2 size={16} /> },
    ],
    []
  );

  const handleAppContextMenu = useCallback(
    (packageId: string, action: string) => {
      switch (action) {
        case "launch":
          handleLaunchApp(packageId);
          break;
        case "delete":
          handleDeleteApp(packageId);
          break;
      }
    },
    [handleLaunchApp, handleDeleteApp]
  );

  const categories = ["all", "productivity", "utilities", "games", "creative", "general"];

  return (
    <div className="launcher">
      <div className="launcher-header">
        <h1 className="launcher-title">
          <Rocket
            size={28}
            style={{ display: "inline-block", marginRight: "12px", verticalAlign: "middle" }}
          />
          App Launcher
        </h1>
        <p className="launcher-subtitle">
          {apps.length} {apps.length === 1 ? "app" : "apps"} installed
        </p>
      </div>

      <div className="launcher-categories">
        {categories.map((cat) => (
          <button
            key={cat}
            className={cn(
              categoryButtonVariants({
                active: selectedCategory === (cat === "all" ? null : cat),
              })
            )}
            onClick={() => setSelectedCategory(cat === "all" ? null : cat)}
          >
            {cat}
          </button>
        ))}
      </div>

      {error && (
        <div className="launcher-error">
          <span>
            <AlertTriangle size={16} style={{ marginRight: "6px", verticalAlign: "middle" }} />
            {error}
          </span>
          <button onClick={() => refetch()}>Retry</button>
        </div>
      )}

      {isLoading ? (
        <div className="launcher-loading">
          <div className="spinner" />
          <p>Loading apps...</p>
        </div>
      ) : (
        <div className="app-grid">
          {apps.map((app) => (
            <div key={app.id} className="app-grid-item">
              <ContextMenu
                items={getAppContextMenuItems(app.id)}
                onSelect={(action) => handleAppContextMenu(app.id, action)}
              >
                <div
                  className={cn(
                    "app-card",
                    cardVariants({
                      variant: "default",
                      padding: "medium",
                      interactive: true,
                    })
                  )}
                  onClick={() => handleLaunchApp(app.id)}
                  title={`${app.name} - ${app.description}`}
                >
                  <Tooltip content="Delete app" delay={500}>
                    <button
                      className="app-delete"
                      onClick={(e) => handleDeleteApp(app.id, e)}
                      aria-label="Delete app"
                    >
                      ×
                    </button>
                  </Tooltip>
                  <div className="app-icon">{app.icon}</div>
                  <div className="app-name">{app.name}</div>
                  <div className="app-description">{app.description}</div>
                  <div className="app-meta">
                    <span className="app-category">{app.category}</span>
                    <span className="app-version">v{app.version}</span>
                  </div>
                </div>
              </ContextMenu>
            </div>
          ))}

          <div
            className={cn(
              "app-card app-card-new",
              cardVariants({
                variant: "outlined",
                padding: "medium",
                interactive: true,
              })
            )}
            onClick={onCreateNew}
          >
            <div className="app-icon">
              <Plus size={32} />
            </div>
            <div className="app-name">Create New App</div>
            <div className="app-description">Generate a new app with AI</div>
          </div>
        </div>
      )}
    </div>
  );
});

Launcher.displayName = "Launcher";
