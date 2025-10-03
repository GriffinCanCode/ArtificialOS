/**
 * Session Manager Hook
 * Handles auto-save, manual save/restore, and session management
 */

import { useEffect, useState, useCallback, useRef } from 'react';
import { SessionClient } from '../utils/sessionClient';
import { useAppStore } from '../store/appStore';
import type { ChatState, UIState } from '../types/session';
import { logger } from '../utils/logger';

interface UseSessionManagerOptions {
  autoSaveInterval?: number; // in seconds, default 30
  enableAutoSave?: boolean;   // default true
  restoreOnMount?: boolean;   // default true
}

export function useSessionManager(options: UseSessionManagerOptions = {}) {
  const {
    autoSaveInterval = 30,
    enableAutoSave = true,
    restoreOnMount = true,
  } = options;

  const [isSaving, setIsSaving] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);
  const [lastSaveTime, setLastSaveTime] = useState<Date | null>(null);
  const [error, setError] = useState<string | null>(null);
  
  const autoSaveTimerRef = useRef<NodeJS.Timeout | null>(null);
  const hasRestoredRef = useRef(false);

  // Get state from store
  const messages = useAppStore((state) => state.messages);
  const thoughts = useAppStore((state) => state.thoughts);
  const uiSpec = useAppStore((state) => state.uiSpec);
  const isLoading = useAppStore((state) => state.isLoading);
  const storeError = useAppStore((state) => state.error);
  const generationThoughts = useAppStore((state) => state.generationThoughts);
  const generationPreview = useAppStore((state) => state.generationPreview);

  /**
   * Capture current frontend state
   */
  const captureState = useCallback(() => {
    const chatState: ChatState = {
      messages: messages.map(m => ({
        type: m.type,
        content: m.content,
        timestamp: m.timestamp,
      })),
      thoughts: thoughts.map(t => ({
        content: t.content,
        timestamp: t.timestamp,
      })),
    };

    const uiState: UIState = {
      generation_thoughts: generationThoughts,
      generation_preview: generationPreview,
      is_loading: isLoading,
      error: storeError || undefined,
    };

    return { chatState, uiState };
  }, [messages, thoughts, generationThoughts, generationPreview, isLoading, storeError]);

  /**
   * Save session with default name (for auto-save)
   */
  const saveDefault = useCallback(async () => {
    try {
      setIsSaving(true);
      setError(null);

      // Capture state (for potential future use)
      captureState();

      // For auto-save, we use the simplified endpoint
      await SessionClient.saveDefault();

      setLastSaveTime(new Date());
      logger.info('Auto-saved session', { component: 'SessionManager' });
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to save session';
      setError(message);
      logger.error('Auto-save failed', err, { component: 'SessionManager' });
    } finally {
      setIsSaving(false);
    }
  }, [captureState]);

  /**
   * Save session with custom name (for manual save)
   */
  const save = useCallback(async (name: string, description?: string) => {
    try {
      setIsSaving(true);
      setError(null);

      const { chatState, uiState } = captureState();

      const result = await SessionClient.saveSession({
        name,
        description,
        chat_state: chatState,
        ui_state: uiState,
      });

      setLastSaveTime(new Date());
      logger.info('Manually saved session', { 
        component: 'SessionManager',
        sessionId: result.session.id,
      });

      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to save session';
      setError(message);
      logger.error('Manual save failed', err, { component: 'SessionManager' });
      throw err;
    } finally {
      setIsSaving(false);
    }
  }, [captureState]);

  /**
   * Restore a session
   */
  const restore = useCallback(async (sessionId: string) => {
    try {
      setIsRestoring(true);
      setError(null);

      const result = await SessionClient.restoreSession(sessionId);

      // Restore frontend state
      const { workspace } = result;
      
      if (workspace.chat_state) {
        // Restore messages and thoughts
        const store = useAppStore.getState();
        store.resetState();
        
        workspace.chat_state.messages.forEach(msg => {
          store.addMessage({
            type: msg.type as 'user' | 'assistant' | 'system',
            content: msg.content,
            timestamp: msg.timestamp,
          });
        });

        workspace.chat_state.thoughts.forEach(thought => {
          store.addThought({
            content: thought.content,
            timestamp: thought.timestamp,
          });
        });
      }

      if (workspace.ui_state) {
        // Restore UI state
        const store = useAppStore.getState();
        store.setLoading(workspace.ui_state.is_loading);
        if (workspace.ui_state.error) {
          store.setError(workspace.ui_state.error);
        }
      }

      // Restore focused app UI
      if (workspace.focused_app_id && workspace.apps.length > 0) {
        const focusedApp = workspace.apps.find(app => app.id === workspace.focused_app_id);
        if (focusedApp) {
          useAppStore.getState().setUISpec(focusedApp.ui_spec as any, focusedApp.id);
        }
      }

      logger.info('Restored session', { 
        component: 'SessionManager',
        sessionId,
        appCount: workspace.apps.length,
      });

      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to restore session';
      setError(message);
      logger.error('Restore failed', err, { component: 'SessionManager' });
      throw err;
    } finally {
      setIsRestoring(false);
    }
  }, []);

  /**
   * Restore default session on mount
   */
  useEffect(() => {
    if (restoreOnMount && !hasRestoredRef.current) {
      hasRestoredRef.current = true;

      // Try to restore the most recent session
      SessionClient.listSessions()
        .then(({ sessions }) => {
          if (sessions.length > 0) {
            // Sort by updated_at and get most recent
            const sorted = sessions.sort((a, b) => 
              new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
            );
            const latestId = sorted[0].id;
            
            logger.info('Restoring latest session on mount', { 
              component: 'SessionManager',
              sessionId: latestId,
            });

            return restore(latestId);
          }
        })
        .catch(err => {
          logger.warn('No session to restore on mount', { 
            component: 'SessionManager',
            error: err instanceof Error ? err.message : String(err),
          });
        });
    }
  }, [restoreOnMount, restore]);

  /**
   * Setup auto-save interval
   */
  useEffect(() => {
    if (enableAutoSave) {
      // Clear existing timer
      if (autoSaveTimerRef.current) {
        clearInterval(autoSaveTimerRef.current);
      }

      // Setup new timer
      autoSaveTimerRef.current = setInterval(() => {
        // Only auto-save if there's something to save (has messages or an app loaded)
        if (messages.length > 0 || uiSpec) {
          saveDefault();
        }
      }, autoSaveInterval * 1000);

      logger.info('Auto-save enabled', { 
        component: 'SessionManager',
        interval: autoSaveInterval,
      });

      return () => {
        if (autoSaveTimerRef.current) {
          clearInterval(autoSaveTimerRef.current);
          autoSaveTimerRef.current = null;
        }
      };
    }
  }, [enableAutoSave, autoSaveInterval, saveDefault, messages.length, uiSpec]);

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

