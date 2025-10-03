/**
 * App Launcher Component
 * Grid-based launcher for installed apps from the registry
 */

import React, { useEffect, useState } from "react";
import { RegistryClient } from "../utils/registryClient";
import type { PackageMetadata } from "../types/registry";
import "./Launcher.css";

interface LauncherProps {
  onAppLaunch?: (appId: string, uiSpec: Record<string, any>) => void;
  onCreateNew?: () => void;
}

export const Launcher: React.FC<LauncherProps> = ({ onAppLaunch, onCreateNew }) => {
  const [apps, setApps] = useState<PackageMetadata[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);

  useEffect(() => {
    loadApps();
  }, [selectedCategory]);

  const loadApps = async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await RegistryClient.listApps(selectedCategory || undefined);
      setApps(response.apps);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load apps");
      setApps([]);
    } finally {
      setLoading(false);
    }
  };

  const handleLaunchApp = async (packageId: string) => {
    try {
      const response = await RegistryClient.launchApp(packageId);
      if (onAppLaunch) {
        onAppLaunch(response.app_id, response.ui_spec);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to launch app");
    }
  };

  const handleDeleteApp = async (packageId: string, event: React.MouseEvent) => {
    event.stopPropagation();

    if (!confirm("Are you sure you want to delete this app?")) {
      return;
    }

    try {
      await RegistryClient.deleteApp(packageId);
      loadApps();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete app");
    }
  };

  const categories = ["all", "productivity", "utilities", "games", "creative", "general"];

  return (
    <div className="launcher">
      <div className="launcher-header">
        <h1 className="launcher-title">🚀 App Launcher</h1>
        <p className="launcher-subtitle">
          {apps.length} {apps.length === 1 ? "app" : "apps"} installed
        </p>
      </div>

      <div className="launcher-categories">
        {categories.map((cat) => (
          <button
            key={cat}
            className={`category-btn ${selectedCategory === (cat === "all" ? null : cat) ? "active" : ""}`}
            onClick={() => setSelectedCategory(cat === "all" ? null : cat)}
          >
            {cat}
          </button>
        ))}
      </div>

      {error && (
        <div className="launcher-error">
          <span>⚠️ {error}</span>
          <button onClick={loadApps}>Retry</button>
        </div>
      )}

      {loading ? (
        <div className="launcher-loading">
          <div className="spinner" />
          <p>Loading apps...</p>
        </div>
      ) : (
        <div className="app-grid">
          {apps.map((app) => (
            <div key={app.id} className="app-card" onClick={() => handleLaunchApp(app.id)}>
              <button
                className="app-delete"
                onClick={(e) => handleDeleteApp(app.id, e)}
                title="Delete app"
              >
                ×
              </button>
              <div className="app-icon">{app.icon}</div>
              <div className="app-name">{app.name}</div>
              <div className="app-description">{app.description}</div>
              <div className="app-meta">
                <span className="app-category">{app.category}</span>
                <span className="app-version">v{app.version}</span>
              </div>
            </div>
          ))}

          <div className="app-card app-card-new" onClick={onCreateNew}>
            <div className="app-icon">➕</div>
            <div className="app-name">Create New App</div>
            <div className="app-description">Generate a new app with AI</div>
          </div>
        </div>
      )}
    </div>
  );
};
