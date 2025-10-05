/**
 * Session Manager Hook
 * Handles auto-save, manual save/restore, and session management
 * Now powered by TanStack Query for better caching and state management
 */

import { useEffect, useState, useCallback, useRef } from "react";
import { useAppStore } from "../store/appStore";
import { useWindowStore } from "../store/windowStore";
import type { ChatState, UIState } from "../types/session";
import { logger } from "../utils/monitoring/logger";
import {
  useSaveSession,
  useSaveDefaultSession,
  useRestoreSession,
  useSessions,
} from "./useSessionQueries";

interface UseSessionManagerOptions {
  autoSaveInterval?: number; // in seconds, default 30
  enableAutoSave?: boolean; // default true
  restoreOnMount?: boolean; // default true
}

export function useSessionManager(options: UseSessionManagerOptions = {}) {
  const { autoSaveInterval = 30, enableAutoSave = true, restoreOnMount = true } = options;

  // TanStack Query mutations
  const saveSessionMutation = useSaveSession();
  const saveDefaultMutation = useSaveDefaultSession();
  const restoreSessionMutation = useRestoreSession();
  const { data: sessionsData } = useSessions();

  const [lastSaveTime, setLastSaveTime] = useState<Date | null>(null);

  const autoSaveTimerRef = useRef<NodeJS.Timeout | null>(null);
  const hasRestoredRef = useRef(false);

  // Derive state from mutations
  const isSaving = saveSessionMutation.isPending || saveDefaultMutation.isPending;
  const isRestoring = restoreSessionMutation.isPending;
  const error =
    saveSessionMutation.error?.message ||
    saveDefaultMutation.error?.message ||
    restoreSessionMutation.error?.message ||
    null;

  /**
   * Capture current frontend state
   * Access store directly to avoid recreating this callback on every state change
   */
  const captureState = useCallback(() => {
    const state = useAppStore.getState();

    const chatState: ChatState = {
      messages: state.messages.map((m) => ({
        type: m.type,
        content: m.content,
        timestamp: m.timestamp,
      })),
      thoughts: state.thoughts.map((t) => ({
        content: t.content,
        timestamp: t.timestamp,
      })),
    };

    const uiState: UIState = {
      generation_thoughts: state.generationThoughts,
      generation_preview: state.generationPreview,
      is_loading: state.isLoading,
      error: state.error || undefined,
    };

    return { chatState, uiState };
  }, []); // No dependencies - reads from store directly

  /**
   * Save session with default name (for auto-save)
   */
  const saveDefault = useCallback(async () => {
    // Capture state (for potential future use)
    captureState();

    // Use TanStack Query mutation
    try {
      await saveDefaultMutation.mutateAsync(undefined);
      setLastSaveTime(new Date());
    } catch (error) {
      // Error already logged by mutation
      logger.warn("Auto-save failed", {
        component: "SessionManager",
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }, [captureState, saveDefaultMutation]);

  /**
   * Save session with custom name (for manual save)
   */
  const save = useCallback(
    async (name: string, description?: string) => {
      const { chatState, uiState } = captureState();

      const result = await saveSessionMutation.mutateAsync({
        name,
        description,
        chat_state: chatState,
        ui_state: uiState,
      });

      setLastSaveTime(new Date());
      return result;
    },
    [captureState, saveSessionMutation]
  );

  /**
   * Restore a session
   */
  const restore = useCallback(
    async (sessionId: string) => {
      const store = useAppStore.getState();
      const { openWindow, updateWindowPosition, updateWindowSize, clearAllWindows } = useWindowStore.getState();

      // Prevent restore during active generation to avoid interrupting streaming
      if (store.isStreaming || store.isLoading) {
        logger.warn("Cannot restore session while generation is in progress", {
          component: "SessionManager",
          isStreaming: store.isStreaming,
          isLoading: store.isLoading,
        });
        throw new Error("Cannot restore session during active generation");
      }

      const result = await restoreSessionMutation.mutateAsync(sessionId);

      // Wait a bit for backend to finish restoring apps before updating frontend state
      // This is non-blocking and won't interfere with any future streaming
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Restore frontend state
      const { workspace } = result;

      // Reset state first to ensure clean slate
      store.resetState();
      clearAllWindows();

      if (workspace.chat_state) {
        // Restore messages and thoughts
        workspace.chat_state.messages.forEach((msg) => {
          store.addMessage({
            type: msg.type as "user" | "assistant" | "system",
            content: msg.content,
            timestamp: msg.timestamp,
          });
        });

        workspace.chat_state.thoughts.forEach((thought) => {
          store.addThought({
            content: thought.content,
            timestamp: thought.timestamp,
          });
        });
      }

      if (workspace.ui_state) {
        // Only restore error state, never loading/streaming state during restore
        if (workspace.ui_state.error) {
          store.setError(workspace.ui_state.error);
        }
      }

      // Restore all apps as windows with saved positions
      if (workspace.apps && workspace.apps.length > 0) {
        workspace.apps.forEach((app: any) => {
          // Open window with saved or default position/size
          const windowId = openWindow(
            app.id,
            app.title,
            app.ui_spec,
            app.ui_spec?.icon || "📦"
          );

          // Restore window geometry if saved
          if (app.window_pos && app.window_size) {
            // Small delay to ensure window is created
            setTimeout(() => {
              updateWindowPosition(windowId, {
                x: app.window_pos.x,
                y: app.window_pos.y,
              });
              updateWindowSize(windowId, {
                width: app.window_size.width,
                height: app.window_size.height,
              });
            }, 50);
          }
        });

        logger.info("Restored session with windows", {
          component: "SessionManager",
          count: workspace.apps.length,
        });
      }

      // Note: We no longer restore focused app UI to fullscreen DynamicRenderer
      // since apps are now windowed by default

      return result;
    },
    [restoreSessionMutation]
  );

  /**
   * Restore default session on mount
   */
  useEffect(() => {
    if (restoreOnMount && !hasRestoredRef.current && sessionsData) {
      hasRestoredRef.current = true;

      // Try to restore the most recent session
      if (sessionsData.sessions.length > 0) {
        // Sessions are already sorted by the query hook
        const latestId = sessionsData.sessions[0].id;

        logger.info("Restoring latest session on mount", {
          component: "SessionManager",
          sessionId: latestId,
        });

        restore(latestId).catch((err) => {
          logger.warn("Failed to restore session on mount", {
            component: "SessionManager",
            error: err instanceof Error ? err.message : String(err),
          });
        });
      }
    }
  }, [restoreOnMount, restore, sessionsData]);

  /**
   * Setup auto-save interval
   */
  useEffect(() => {
    if (!enableAutoSave) return;

    // Clear existing timer
    if (autoSaveTimerRef.current) {
      clearInterval(autoSaveTimerRef.current);
    }

    // Setup new timer
    autoSaveTimerRef.current = setInterval(() => {
      // Only auto-save if there's something to save (has messages or an app loaded)
      // Access current state directly from store to avoid dependency issues
      const currentMessages = useAppStore.getState().messages;
      const currentUISpec = useAppStore.getState().blueprint;

      if (currentMessages.length > 0 || currentUISpec) {
        saveDefault();
      }
    }, autoSaveInterval * 1000);

    logger.info("Auto-save enabled", {
      component: "SessionManager",
      interval: autoSaveInterval,
    });

    return () => {
      if (autoSaveTimerRef.current) {
        clearInterval(autoSaveTimerRef.current);
        autoSaveTimerRef.current = null;
      }
    };
  }, [enableAutoSave, autoSaveInterval, saveDefault]); // Removed messages.length and blueprint from deps

  return {
    save,
    restore,
    saveDefault,
    isSaving,
    isRestoring,
    lastSaveTime,
    error,
  };
}
