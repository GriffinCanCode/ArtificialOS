/**
 * Custom Title Bar Component
 * Provides window controls for frameless Electron window
 */

import React, { useState, useCallback } from "react";
import { Save, FolderOpen, X, Trash2 } from "lucide-react";
import { useSessions, useDeleteSession } from "../../hooks/useSessionQueries";
import { useLogger } from "../../utils/monitoring/useLogger";
import { SaveSessionDialog } from "../dialogs/SaveSessionDialog";
import { controlButtonVariants, cn } from "../../utils/animation/componentVariants";
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

const TitleBar: React.FC<TitleBarProps> = React.memo(({ sessionManager }) => {
  const log = useLogger("TitleBar");
  const [showSessionMenu, setShowSessionMenu] = useState(false);
  const [showSaveDialog, setShowSaveDialog] = useState(false);

  // Use TanStack Query for sessions
  const {
    data: sessionsData,
    refetch: refetchSessions,
    isLoading: isLoadingSessions,
  } = useSessions();
  const deleteSessionMutation = useDeleteSession();

  const sessions = sessionsData?.sessions ?? [];

  const handleMinimize = useCallback(() => {
    if (window.electron) {
      window.electron.minimize();
    }
  }, []);

  const handleMaximize = useCallback(() => {
    if (window.electron) {
      window.electron.maximize();
    }
  }, []);

  const handleClose = useCallback(() => {
    if (window.electron) {
      window.electron.close();
    }
  }, []);

  const handleSave = useCallback(() => {
    if (!sessionManager) return;
    setShowSaveDialog(true);
  }, [sessionManager]);

  const handleSaveSubmit = useCallback(
    async (data: { name: string; description?: string }) => {
      if (!sessionManager) return;

      try {
        await sessionManager.save(data.name, data.description);
        log.info("Session saved successfully", { name: data.name });
      } catch (err) {
        log.error("Failed to save session", err as Error);
        throw err; // Let the dialog handle error display
      }
    },
    [sessionManager, log]
  );

  const handleShowSessions = useCallback(() => {
    setShowSessionMenu(!showSessionMenu);
    if (!showSessionMenu) {
      // Refetch to ensure fresh data when opening menu
      refetchSessions();
    }
  }, [showSessionMenu, refetchSessions]);

  const handleDeleteSession = useCallback(
    async (sessionId: string, event: React.MouseEvent) => {
      event.stopPropagation();

      if (!confirm("Are you sure you want to delete this session?")) {
        return;
      }

      try {
        await deleteSessionMutation.mutateAsync(sessionId);
        log.info("Session deleted successfully", { sessionId });
      } catch (err) {
        log.error("Failed to delete session", err as Error);
      }
    },
    [deleteSessionMutation, log]
  );

  const handleRestore = useCallback(
    async (sessionId: string) => {
      if (!sessionManager) return;

      try {
        await sessionManager.restore(sessionId);
        setShowSessionMenu(false);
        log.info("Session restored successfully", { sessionId });
      } catch (err) {
        log.error("Failed to restore session", err as Error);
      }
    },
    [sessionManager, log]
  );

  const closeSaveDialog = useCallback(() => {
    setShowSaveDialog(false);
  }, []);

  const closeSessionMenu = useCallback(() => {
    setShowSessionMenu(false);
  }, []);

  return (
    <>
      <SaveSessionDialog
        isOpen={showSaveDialog}
        onClose={closeSaveDialog}
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
              <button onClick={closeSessionMenu}>
                <X size={16} />
              </button>
            </div>
            <div className="session-list">
              {isLoadingSessions ? (
                <p className="no-sessions">Loading sessions...</p>
              ) : sessions.length === 0 ? (
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
                    <button
                      className="session-delete"
                      onClick={(e) => handleDeleteSession(session.id, e)}
                      title="Delete session"
                      disabled={deleteSessionMutation.isPending}
                    >
                      <Trash2 size={14} />
                    </button>
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
});

TitleBar.displayName = "TitleBar";

export default TitleBar;
