/**
 * useSyncState Hook
 * High-performance external state subscription using React 18's useSyncExternalStore
 * Eliminates forceUpdate anti-pattern and reduces re-renders
 */

import { useSyncExternalStore, useCallback } from "react";
import type { ComponentState } from "../state/state";

/**
 * Subscribe to ComponentState using React 18's efficient external store API
 * This hook ensures optimal performance by only re-rendering when subscribed value changes
 *
 * @param state - ComponentState instance
 * @param key - State key to subscribe to
 * @param defaultValue - Default value if key doesn't exist
 * @returns Current value from state
 *
 * @example
 * ```tsx
 * const value = useSyncState(state, component.id, "");
 * ```
 */
export function useSyncState<T = any>(state: ComponentState, key: string, defaultValue?: T): T {
  // Subscribe callback - called by React when component mounts/updates
  const subscribe = useCallback(
    (callback: () => void) => {
      // Subscribe to state changes and call React's callback
      const unsubscribe = state.subscribe(key, () => {
        callback();
      });

      return unsubscribe;
    },
    [state, key]
  );

  // Snapshot callback - called by React to get current value
  // CRITICAL: This must NOT use useCallback with dependencies!
  // Every call must return the CURRENT value from state
  const getSnapshot = () => {
    return state.get(key, defaultValue);
  };

  // Server snapshot for SSR (return same as client)
  const getServerSnapshot = () => {
    return defaultValue as T;
  };

  return useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot);
}
