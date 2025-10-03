/**
 * TanStack Query Hooks for Session API
 * Provides caching, refetching, and optimistic updates for session operations
 */

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { SessionClient } from "../utils/sessionClient";
import type {
  SessionMetadata,
  Session,
  SaveSessionRequest,
  SaveSessionResponse,
  RestoreSessionResponse,
  ListSessionsResponse,
} from "../types/session";
import { logger } from "../utils/logger";

// ============================================================================
// Query Keys - Centralized for consistency
// ============================================================================

export const sessionKeys = {
  all: ["sessions"] as const,
  lists: () => [...sessionKeys.all, "list"] as const,
  list: () => [...sessionKeys.lists()] as const,
  details: () => [...sessionKeys.all, "detail"] as const,
  detail: (id: string) => [...sessionKeys.details(), id] as const,
};

// ============================================================================
// Query Hooks
// ============================================================================

/**
 * Fetch list of all sessions
 * - Automatically cached and refetched
 * - Sorted by most recent first
 */
export function useSessions() {
  return useQuery({
    queryKey: sessionKeys.list(),
    queryFn: async () => {
      logger.info("Fetching sessions list", {
        component: "useSessions",
      });
      return SessionClient.listSessions();
    },
    staleTime: 10 * 1000, // Sessions list fresh for 10 seconds
    gcTime: 5 * 60 * 1000, // Keep in cache for 5 minutes
    retry: 2,
    select: (data) => {
      // Sort sessions by updated_at (most recent first)
      return {
        ...data,
        sessions: [...data.sessions].sort(
          (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
        ),
      };
    },
  });
}

/**
 * Fetch details of a specific session
 */
export function useSession(sessionId: string, options?: { enabled?: boolean }) {
  return useQuery({
    queryKey: sessionKeys.detail(sessionId),
    queryFn: async () => {
      logger.info("Fetching session details", {
        component: "useSession",
        sessionId,
      });
      return SessionClient.getSession(sessionId);
    },
    enabled: options?.enabled ?? true,
    staleTime: 30 * 1000, // Session details fresh for 30 seconds
    gcTime: 10 * 60 * 1000,
    retry: 2,
  });
}

// ============================================================================
// Mutation Hooks
// ============================================================================

/**
 * Save a session with custom name and details
 * - Invalidates session list after save
 */
export function useSaveSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: SaveSessionRequest) => {
      logger.info("Saving session", {
        component: "useSaveSession",
        name: request.name,
      });
      return SessionClient.saveSession(request);
    },
    onSuccess: (data, request) => {
      logger.info("Session saved successfully", {
        component: "useSaveSession",
        sessionId: data.session.id,
        name: request.name,
      });

      // Add the new session to the cache optimistically
      queryClient.setQueryData<ListSessionsResponse>(
        sessionKeys.list(),
        (old) => {
          if (!old) return old;
          
          // Check if session already exists (update case)
          const existingIndex = old.sessions.findIndex(s => s.id === data.session.id);
          if (existingIndex >= 0) {
            const updated = [...old.sessions];
            updated[existingIndex] = data.session;
            return { ...old, sessions: updated };
          }
          
          // New session - add to beginning
          return {
            ...old,
            sessions: [data.session, ...old.sessions],
            stats: {
              ...old.stats,
              total_sessions: old.stats.total_sessions + 1,
            },
          };
        }
      );

      // Invalidate to ensure fresh data
      queryClient.invalidateQueries({ queryKey: sessionKeys.lists() });
    },
    onError: (error, request) => {
      logger.error("Failed to save session", error as Error, {
        component: "useSaveSession",
        name: request.name,
      });
    },
  });
}

/**
 * Save session with default name (for auto-save)
 * - Lightweight operation
 * - Doesn't require cache updates as it's background
 */
export function useSaveDefaultSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      logger.info("Saving default session", {
        component: "useSaveDefaultSession",
      });
      return SessionClient.saveDefault();
    },
    onSuccess: (data) => {
      logger.info("Default session saved", {
        component: "useSaveDefaultSession",
        sessionId: data.session.id,
      });

      // Silently update cache in background
      queryClient.invalidateQueries({ 
        queryKey: sessionKeys.lists(),
        refetchType: "none", // Don't refetch, just mark as stale
      });
    },
    onError: (error) => {
      logger.error("Failed to save default session", error as Error, {
        component: "useSaveDefaultSession",
      });
    },
  });
}

/**
 * Restore a session
 * - Updates application state
 */
export function useRestoreSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (sessionId: string) => {
      logger.info("Restoring session", {
        component: "useRestoreSession",
        sessionId,
      });
      return SessionClient.restoreSession(sessionId);
    },
    onSuccess: (data, sessionId) => {
      logger.info("Session restored successfully", {
        component: "useRestoreSession",
        sessionId,
        appCount: data.workspace.apps.length,
      });

      // Update cache with restored session data
      queryClient.setQueryData(sessionKeys.detail(sessionId), data);
    },
    onError: (error, sessionId) => {
      logger.error("Failed to restore session", error as Error, {
        component: "useRestoreSession",
        sessionId,
      });
    },
  });
}

/**
 * Delete a session
 * - Optimistically removes from cache
 * - Rolls back on error
 */
export function useDeleteSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (sessionId: string) => {
      logger.info("Deleting session", {
        component: "useDeleteSession",
        sessionId,
      });
      return SessionClient.deleteSession(sessionId);
    },
    onMutate: async (sessionId) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: sessionKeys.lists() });

      // Snapshot previous values
      const previousSessions = queryClient.getQueryData(sessionKeys.list());

      // Optimistically update
      queryClient.setQueryData<ListSessionsResponse>(
        sessionKeys.list(),
        (old) => {
          if (!old) return old;
          return {
            ...old,
            sessions: old.sessions.filter((s) => s.id !== sessionId),
            stats: {
              ...old.stats,
              total_sessions: old.stats.total_sessions - 1,
            },
          };
        }
      );

      return { previousSessions };
    },
    onError: (error, sessionId, context) => {
      logger.error("Failed to delete session", error as Error, {
        component: "useDeleteSession",
        sessionId,
      });

      // Rollback to previous state
      if (context?.previousSessions) {
        queryClient.setQueryData(sessionKeys.list(), context.previousSessions);
      }
    },
    onSettled: () => {
      // Always refetch to ensure consistency
      queryClient.invalidateQueries({ queryKey: sessionKeys.lists() });
    },
  });
}

// ============================================================================
// Combined Hook for All Session Mutations
// ============================================================================

/**
 * Get all session mutations in one hook for convenience
 */
export function useSessionMutations() {
  const saveSession = useSaveSession();
  const saveDefault = useSaveDefaultSession();
  const restoreSession = useRestoreSession();
  const deleteSession = useDeleteSession();

  return {
    saveSession,
    saveDefault,
    restoreSession,
    deleteSession,
  };
}

