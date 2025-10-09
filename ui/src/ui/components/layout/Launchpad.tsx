/**
 * Launchpad Component
 * Desktop-integrated app grid that replaces desktop icons
 */

import React, { useState, useEffect, useCallback } from "react";
import { Search } from "lucide-react";
import "./Launchpad.css";

interface LaunchpadApp {
  id: string;
  name: string;
  icon: string;
  description: string;
  category: string;
  type: string;
}

interface LaunchpadProps {
  isVisible: boolean;
  onLaunchApp: (appId: string) => void;
}

export const Launchpad: React.FC<LaunchpadProps> = ({ isVisible, onLaunchApp }) => {
  const [apps, setApps] = useState<LaunchpadApp[]>([]);
  const [filteredApps, setFilteredApps] = useState<LaunchpadApp[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [loading, setLoading] = useState(true);

  // Fetch apps from registry
  useEffect(() => {
    if (isVisible) {
      fetchApps();
      setSearchQuery("");
    }
  }, [isVisible]);

  const fetchApps = async () => {
    try {
      setLoading(true);
      const response = await fetch("http://localhost:8000/registry/apps");
      const data = await response.json();
      setApps(data.apps || []);
      setFilteredApps(data.apps || []);
    } catch (error) {
      console.error("[Launchpad] Failed to fetch apps:", error);
      setApps([]);
      setFilteredApps([]);
    } finally {
      setLoading(false);
    }
  };

  // Filter apps by search query
  useEffect(() => {
    if (!searchQuery.trim()) {
      setFilteredApps(apps);
      return;
    }

    const query = searchQuery.toLowerCase();
    const filtered = apps.filter(
      (app) =>
        app.name.toLowerCase().includes(query) ||
        app.description.toLowerCase().includes(query) ||
        app.category.toLowerCase().includes(query)
    );
    setFilteredApps(filtered);
  }, [searchQuery, apps]);

  // Handle app launch
  const handleLaunch = useCallback(
    (appId: string) => {
      onLaunchApp(appId);
      // Optionally close launchpad after launch
      // onClose();
    },
    [onLaunchApp]
  );

  if (!isVisible) return null;

  return (
    <div className={`launchpad-container ${isVisible ? "visible" : ""}`}>
      {/* Search Bar */}
      <div className="launchpad-search">
        <Search size={20} className="launchpad-search-icon" />
        <input
          type="text"
          placeholder="Search apps..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="launchpad-search-input"
        />
      </div>

      {/* Apps Grid */}
      <div className="launchpad-grid">
        {loading ? (
          <div className="launchpad-loading">
            <div className="launchpad-spinner" />
            <p>Loading apps...</p>
          </div>
        ) : filteredApps.length === 0 ? (
          <div className="launchpad-empty">
            <div className="launchpad-empty-icon">📦</div>
            <p>
              {searchQuery ? "No apps found matching your search" : "No apps available"}
            </p>
          </div>
        ) : (
          filteredApps.map((app, index) => (
            <div
              key={app.id}
              className="launchpad-app"
              onClick={() => handleLaunch(app.id)}
              style={{
                animationDelay: `${index * 0.02}s`,
              }}
            >
              <div className="launchpad-app-icon">{app.icon}</div>
              <div className="launchpad-app-name">{app.name}</div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

