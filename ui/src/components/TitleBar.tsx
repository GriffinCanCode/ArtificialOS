/**
 * Custom Title Bar Component
 * Provides window controls for frameless Electron window
 */

import React, { useState } from "react";
import { Save, FolderOpen, X } from "lucide-react";
import { SessionClient } from "../utils/sessionClient";
import type { SessionMetadata } from "../types/session";
import { useLogger } from "../utils/useLogger";
import { SaveSessionDialog } from "./SaveSessionDialog";
import { controlButtonVariants, cn } from "../utils/componentVariants";
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
  const log = useLogger("TitleBar");
  const [showSessionMenu, setShowSessionMenu] = useState(false);
  const [showSaveDialog, setShowSaveDialog] = useState(false);
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

  const handleSave = () => {
    if (!sessionManager) return;
    setShowSaveDialog(true);
  };

  const handleSaveSubmit = async (data: { name: string; description?: string }) => {
    if (!sessionManager) return;
    
    try {
      await sessionManager.save(data.name, data.description);
      log.info("Session saved successfully", { name: data.name });
    } catch (err) {
      log.error("Failed to save session", err as Error);
      throw err; // Let the dialog handle error display
    }
  };

  const handleShowSessions = async () => {
    setShowSessionMenu(!showSessionMenu);
    if (!showSessionMenu) {
      try {
        const result = await SessionClient.listSessions();
        setSessions(result.sessions);
      } catch (err) {
        log.error("Failed to load sessions", err as Error);
      }
    }
  };

  const handleRestore = async (sessionId: string) => {
    if (!sessionManager) return;

    try {
      await sessionManager.restore(sessionId);
      setShowSessionMenu(false);
      log.info("Session restored successfully", { sessionId });
    } catch (err) {
      log.error("Failed to restore session", err as Error);
    }
  };

  return (
    <>
      <SaveSessionDialog
        isOpen={showSaveDialog}
        onClose={() => setShowSaveDialog(false)}
        onSave={handleSaveSubmit}
        isLoading={sessionManager?.isSaving}
      />
      {/* Minimal title bar - just window controls on hover */}
      <div className="title-bar minimal">
        <div className="title-bar-drag" />

        <div className="window-controls">
          <button
            className={cn("control-btn", controlButtonVariants({ type: "minimize" }))}
            onClick={handleMinimize}
            aria-label="Minimize"
          >
            <span></span>
          </button>
          <button
            className={cn("control-btn", controlButtonVariants({ type: "maximize" }))}
            onClick={handleMaximize}
            aria-label="Maximize"
          >
            <span></span>
          </button>
          <button
            className={cn("control-btn", controlButtonVariants({ type: "close" }))}
            onClick={handleClose}
            aria-label="Close"
          >
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
              <button onClick={() => setShowSessionMenu(false)}><X size={16} /></button>
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
            <Save size={18} />
          </button>
          <button
            className="session-btn"
            onClick={handleShowSessions}
            disabled={sessionManager.isRestoring}
            title="Load saved session (⌘O)"
          >
            <FolderOpen size={18} />
          </button>
        </div>
      )}
    </>
  );
};

export default TitleBar;
