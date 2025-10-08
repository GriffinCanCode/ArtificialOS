/**
 * useShortcuts Hook
 * React hook for registering multiple keyboard shortcuts
 */

import { useEffect, useRef } from "react";
import { useActions } from "../shortcuts/store/store";
import type { ShortcutConfig } from "../shortcuts/core/types";

// ============================================================================
// Hook Implementation
// ============================================================================

/**
 * Register multiple keyboard shortcuts that are automatically cleaned up on unmount
 */
export function useShortcuts(configs: ShortcutConfig[]): void {
  const { registerMany, unregister } = useActions();
  const configsRef = useRef(configs);
  const idsRef = useRef<string[]>([]);

  // Keep configs ref updated
  useEffect(() => {
    configsRef.current = configs;
  }, [configs]);

  // Register shortcuts on mount
  useEffect(() => {
    // Wrap handlers to use ref
    const wrappedConfigs = configs.map((config) => {
      const originalHandler = config.handler;
      return {
        ...config,
        handler: (event: KeyboardEvent, context: any) => {
          // Find current config
          const current = configsRef.current.find((c) => c.id === config.id);
          if (current) {
            return current.handler(event, context);
          }
          return originalHandler(event, context);
        },
      };
    });

    const ids = registerMany(wrappedConfigs);
    idsRef.current = ids;

    // Cleanup on unmount
    return () => {
      for (const id of idsRef.current) {
        unregister(id);
      }
    };
  }, []); // Empty deps - only register once

  // Update shortcuts when configs change (without re-registering)
  useEffect(() => {
    // The handler wrapper above ensures we always use the latest handler
  }, [configs]);
}

/**
 * Register shortcuts from a map
 */
export function useShortcutMap(
  shortcuts: Record<string, ShortcutConfig>
): void {
  const configs = Object.values(shortcuts);
  useShortcuts(configs);
}

/**
 * Register shortcuts for a specific scope
 */
export function useScopedShortcuts(
  scope: ShortcutConfig["scope"],
  configs: Omit<ShortcutConfig, "scope">[]
): void {
  const scopedConfigs = configs.map((config) => ({
    ...config,
    scope,
  }));

  useShortcuts(scopedConfigs as ShortcutConfig[]);
}

