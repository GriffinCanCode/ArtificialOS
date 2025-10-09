/**
 * Developer Settings Page
 */

import { useState, useEffect } from 'react';
import { SettingRow } from '../components/SettingRow';
import './Page.css';

interface DeveloperPageProps {
  executor: any;
  state: any;
}

export function DeveloperPage({ executor }: DeveloperPageProps) {
  const [debugMode, setDebugMode] = useState(false);
  const [showFPS, setShowFPS] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const debugResult = await executor.execute('settings.get', { key: 'developer.debug_mode' });
      if (debugResult?.value !== undefined) setDebugMode(debugResult.value);

      const fpsResult = await executor.execute('settings.get', { key: 'developer.show_fps' });
      if (fpsResult?.value !== undefined) setShowFPS(fpsResult.value);
    } catch (error) {
      console.error('Failed to load developer settings:', error);
    }
  };

  const handleDebugToggle = async () => {
    const newValue = !debugMode;
    setDebugMode(newValue);
    try {
      await executor.execute('settings.set', { key: 'developer.debug_mode', value: newValue });
    } catch (error) {
      console.error('Failed to update debug mode:', error);
    }
  };

  const handleFPSToggle = async () => {
    const newValue = !showFPS;
    setShowFPS(newValue);
    try {
      await executor.execute('settings.set', { key: 'developer.show_fps', value: newValue });
    } catch (error) {
      console.error('Failed to update FPS display:', error);
    }
  };

  const handleExportSettings = async () => {
    try {
      const result = await executor.execute('settings.export', {});
      const blob = new Blob([JSON.stringify(result.settings, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'settings-export.json';
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export settings:', error);
    }
  };

  return (
    <div className="settings-page">
      <div className="page-header">
        <h1>Developer Options</h1>
        <p className="page-description">Advanced settings for development and debugging</p>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Debug Options</h2>
        <SettingRow
          label="Debug Mode"
          description="Enable verbose logging and debug information"
        >
          <div
            className={`toggle-switch ${debugMode ? 'active' : ''}`}
            onClick={handleDebugToggle}
          />
        </SettingRow>
        <SettingRow
          label="Show FPS Counter"
          description="Display frames per second in the interface"
        >
          <div
            className={`toggle-switch ${showFPS ? 'active' : ''}`}
            onClick={handleFPSToggle}
          />
        </SettingRow>
      </div>

      <div className="settings-section">
        <h2 className="section-title">Data Management</h2>
        <SettingRow
          label="Export Settings"
          description="Download all settings as JSON"
        >
          <button className="btn" onClick={handleExportSettings}>
            Export
          </button>
        </SettingRow>
      </div>

      <div className="settings-section">
        <h2 className="section-title">System Information</h2>
        <div className="info-grid">
          <div className="info-item">
            <div className="info-label">Platform</div>
            <div className="info-value">{navigator.platform}</div>
          </div>
          <div className="info-item">
            <div className="info-label">User Agent</div>
            <div className="info-value">{navigator.userAgent}</div>
          </div>
          <div className="info-item">
            <div className="info-label">Language</div>
            <div className="info-value">{navigator.language}</div>
          </div>
          <div className="info-item">
            <div className="info-label">Online</div>
            <div className="info-value">{navigator.onLine ? 'Yes' : 'No'}</div>
          </div>
        </div>
      </div>
    </div>
  );
}

