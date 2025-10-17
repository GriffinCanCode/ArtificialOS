/**
 * Clipboard Hook
 * React hook for clipboard operations with service integration
 */

import { useState, useEffect, useCallback, useSyncExternalStore } from "react";
import type { ClipboardEntry, ClipboardOptions, ClipboardStats, ClipboardState } from "../core/types";
import { clipboardManager } from "../core/manager";
import { logger } from "@/core/utils/monitoring/logger";

interface UseClipboardOptions {
  /**
   * Service client for backend communication
   */
  service?: any;
  /**
   * Auto-load history on mount
   */
  autoLoad?: boolean;
}

export function useClipboard(options: UseClipboardOptions = {}) {
  const { service, autoLoad = true } = options;
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Subscribe to clipboard state
  const state = useSyncExternalStore<ClipboardState>(
    clipboardManager.subscribe.bind(clipboardManager),
    clipboardManager.getState.bind(clipboardManager),
    clipboardManager.getState.bind(clipboardManager)
  );

  /**
   * Copy text to clipboard
   */
  const copy = useCallback(
    async (text: string, opts: ClipboardOptions = {}): Promise<number> => {
      setLoading(true);
      setError(null);

      try {
        // Use browser clipboard first for immediate feedback
        if (navigator.clipboard) {
          await navigator.clipboard.writeText(text);
        }

        // Then sync with backend if service available
        if (service) {
          const result = await service.execute("clipboard.copy", {
            data: text,
            format: opts.format || "text",
            global: opts.global || false,
          });

          if (result?.success && result.data) {
            const entryId = result.data.entry_id;
            logger.debug("Copied to clipboard", { entryId, component: "useClipboard" });
            return entryId;
          }
        }

        return Date.now(); // Fallback ID
      } catch (err) {
        const message = err instanceof Error ? err.message : "Copy failed";
        setError(message);
        logger.error("Clipboard copy failed", err as Error, { component: "useClipboard" });
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [service]
  );

  /**
   * Paste from clipboard
   */
  const paste = useCallback(
    async (opts: ClipboardOptions = {}): Promise<ClipboardEntry | null> => {
      setLoading(true);
      setError(null);

      try {
        // Try backend first
        if (service) {
          const result = await service.execute("clipboard.paste", {
            global: opts.global || false,
          });

          if (result?.success && result.data) {
            const entry = result.data as ClipboardEntry;
            clipboardManager.setCurrent(entry);
            logger.debug("Pasted from clipboard", { entryId: entry.id, component: "useClipboard" });
            return entry;
          }
        }

        // Fallback to browser clipboard
        if (navigator.clipboard) {
          const text = await navigator.clipboard.readText();
          const entry: ClipboardEntry = {
            id: Math.floor(Math.random() * Number.MAX_SAFE_INTEGER),
            data: { type: "Text", data: text },
            source_pid: 0,
            timestamp: Date.now(),
          };
          clipboardManager.setCurrent(entry);
          return entry;
        }

        return null;
      } catch (err) {
        const message = err instanceof Error ? err.message : "Paste failed";
        setError(message);
        logger.error("Clipboard paste failed", err as Error, { component: "useClipboard" });
        return null;
      } finally {
        setLoading(false);
      }
    },
    [service]
  );

  /**
   * Get clipboard history
   */
  const getHistory = useCallback(
    async (opts: ClipboardOptions = {}): Promise<ClipboardEntry[]> => {
      if (!service) return [];

      setLoading(true);
      setError(null);

      try {
        const result = await service.execute("clipboard.history", {
          limit: opts.limit,
          global: opts.global || false,
        });

        if (result?.success && result.data?.entries) {
          const entries = result.data.entries as ClipboardEntry[];
          clipboardManager.setHistory(entries);
          logger.debug("Fetched clipboard history", { count: entries.length, component: "useClipboard" });
          return entries;
        }

        return [];
      } catch (err) {
        const message = err instanceof Error ? err.message : "History fetch failed";
        setError(message);
        logger.error("Clipboard history failed", err as Error, { component: "useClipboard" });
        return [];
      } finally {
        setLoading(false);
      }
    },
    [service]
  );

  /**
   * Get specific entry by ID
   */
  const getEntry = useCallback(
    async (entryId: number): Promise<ClipboardEntry | null> => {
      if (!service) return null;

      setLoading(true);
      setError(null);

      try {
        const result = await service.execute("clipboard.get_entry", { entry_id: entryId });

        if (result?.success && result.data) {
          return result.data as ClipboardEntry;
        }

        return null;
      } catch (err) {
        const message = err instanceof Error ? err.message : "Entry fetch failed";
        setError(message);
        logger.error("Clipboard get entry failed", err as Error, { component: "useClipboard" });
        return null;
      } finally {
        setLoading(false);
      }
    },
    [service]
  );

  /**
   * Clear clipboard
   */
  const clear = useCallback(
    async (global = false) => {
      setLoading(true);
      setError(null);

      try {
        if (service) {
          await service.execute("clipboard.clear", { global });
        }

        if (!global) {
          clipboardManager.clearState();
        }

        logger.debug("Cleared clipboard", { global, component: "useClipboard" });
      } catch (err) {
        const message = err instanceof Error ? err.message : "Clear failed";
        setError(message);
        logger.error("Clipboard clear failed", err as Error, { component: "useClipboard" });
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [service]
  );

  /**
   * Subscribe to clipboard changes
   */
  const subscribe = useCallback(
    async (formats?: string[]) => {
      if (!service) return;

      setLoading(true);
      setError(null);

      try {
        await service.execute("clipboard.subscribe", { formats: formats || [] });
        clipboardManager.setSubscribed(true);
        logger.debug("Subscribed to clipboard", { formats, component: "useClipboard" });
      } catch (err) {
        const message = err instanceof Error ? err.message : "Subscribe failed";
        setError(message);
        logger.error("Clipboard subscribe failed", err as Error, { component: "useClipboard" });
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [service]
  );

  /**
   * Unsubscribe from clipboard changes
   */
  const unsubscribe = useCallback(async () => {
    if (!service) return;

    setLoading(true);
    setError(null);

    try {
      await service.execute("clipboard.unsubscribe", {});
      clipboardManager.setSubscribed(false);
      logger.debug("Unsubscribed from clipboard", { component: "useClipboard" });
    } catch (err) {
      const message = err instanceof Error ? err.message : "Unsubscribe failed";
      setError(message);
      logger.error("Clipboard unsubscribe failed", err as Error, { component: "useClipboard" });
      throw err;
    } finally {
      setLoading(false);
    }
  }, [service]);

  /**
   * Get clipboard statistics
   */
  const getStats = useCallback(async (): Promise<ClipboardStats | null> => {
    if (!service) return null;

    setLoading(true);
    setError(null);

    try {
      const result = await service.execute("clipboard.stats", {});

      if (result?.success && result.data) {
        const stats = result.data as ClipboardStats;
        clipboardManager.setStats(stats);
        logger.debug("Fetched clipboard stats", { stats, component: "useClipboard" });
        return stats;
      }

      return null;
    } catch (err) {
      const message = err instanceof Error ? err.message : "Stats fetch failed";
      setError(message);
      logger.error("Clipboard stats failed", err as Error, { component: "useClipboard" });
      return null;
    } finally {
      setLoading(false);
    }
  }, [service]);

  // Auto-load history on mount
  useEffect(() => {
    if (autoLoad && service) {
      getHistory({ limit: 10 });
    }
  }, [autoLoad, service, getHistory]);

  return {
    // State
    current: state.current,
    history: state.history,
    stats: state.stats,
    subscribed: state.subscribed,
    loading,
    error,

    // Actions
    copy,
    paste,
    getHistory,
    getEntry,
    clear,
    subscribe,
    unsubscribe,
    getStats,
  };
}

