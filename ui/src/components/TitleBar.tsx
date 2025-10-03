/**
 * Custom Title Bar Component
 * Provides window controls for frameless Electron window
 */

import React, { useState } from "react";
import { SessionClient } from "../utils/sessionClient";
import type { SessionMetadata } from "../types/session";
import "./TitleBar.css";

interface TitleBarProps {
  sessionManager?: {
    save: (name: string, description?: string) => Promise<any>;
    restore: (sessionId: string) => Promise<any>;
    saveDefault: () => Promise<any>;
    isSaving: boolean;
    isRestoring: boolean;
    lastSaveTime: Date | null;
    error: string | null;
  };
}

const TitleBar: React.FC<TitleBarProps> = ({ sessionManager }) => {
  const [showSessionMenu, setShowSessionMenu] = useState(false);
  const [sessions, setSessions] = useState<SessionMetadata[]>([]);

  const handleMinimize = () => {
    if (window.electron) {
      window.electron.minimize();
    }
  };

  const handleMaximize = () => {
    if (window.electron) {
      window.electron.maximize();
    }
  };

  const handleClose = () => {
    if (window.electron) {
      window.electron.close();
    }
  };

  const handleSave = async () => {
    if (!sessionManager) return;

    const name = prompt("Enter session name:", "Work Session");
    if (name) {
      const description = prompt("Enter description (optional):");
      try {
        await sessionManager.save(name, description || undefined);
        alert("✅ Session saved!");
      } catch (err) {
        alert("Failed to save: " + (err instanceof Error ? err.message : String(err)));
      }
    }
  };

  const handleShowSessions = async () => {
    setShowSessionMenu(!showSessionMenu);
    if (!showSessionMenu) {
      try {
        const result = await SessionClient.listSessions();
        setSessions(result.sessions);
      } catch (err) {
        console.error("Failed to load sessions:", err);
      }
    }
  };

  const handleRestore = async (sessionId: string) => {
    if (!sessionManager) return;

    try {
      await sessionManager.restore(sessionId);
      setShowSessionMenu(false);
      alert("✅ Session restored!");
    } catch (err) {
      alert("Failed to restore: " + (err instanceof Error ? err.message : String(err)));
    }
  };

  return (
    <>
      {/* Minimal title bar - just window controls on hover */}
      <div className="title-bar minimal">
        <div className="title-bar-drag" />

        <div className="window-controls">
          <button className="control-btn minimize" onClick={handleMinimize} aria-label="Minimize">
            <span></span>
          </button>
          <button className="control-btn maximize" onClick={handleMaximize} aria-label="Maximize">
            <span></span>
          </button>
          <button className="control-btn close" onClick={handleClose} aria-label="Close">
            <span></span>
          </button>
        </div>
      </div>

      {/* Session menu - floating */}
      {showSessionMenu && (
        <div className="session-menu-overlay">
          <div className="session-menu">
            <div className="session-menu-header">
              <h3>Saved Sessions</h3>
              <button onClick={() => setShowSessionMenu(false)}>✕</button>
            </div>
            <div className="session-list">
              {sessions.length === 0 ? (
                <p className="no-sessions">No saved sessions</p>
              ) : (
                sessions.map((session) => (
                  <div
                    key={session.id}
                    className="session-item"
                    onClick={() => handleRestore(session.id)}
                  >
                    <div className="session-info">
                      <div className="session-name">{session.name}</div>
                      <div className="session-meta">
                        {session.app_count} apps •{" "}
                        {new Date(session.updated_at).toLocaleDateString()}
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      )}

      {/* Floating session controls - top-left corner */}
      {sessionManager && (
        <div className="session-controls floating">
          <button
            className="session-btn"
            onClick={handleSave}
            disabled={sessionManager.isSaving}
            title="Save current session (⌘S)"
          >
            💾
          </button>
          <button
            className="session-btn"
            onClick={handleShowSessions}
            disabled={sessionManager.isRestoring}
            title="Load saved session (⌘O)"
          >
            📂
          </button>
        </div>
      )}
    </>
  );
};

export default TitleBar;
