/**
 * useWebSocketConnection Hook
 * High-performance WebSocket connection state subscription using React 18's useSyncExternalStore
 * Provides concurrent-safe access to WebSocket connection status
 */

import { useSyncExternalStore, useCallback } from "react";
import type { WebSocketClient } from "../websocketClient";

/**
 * Subscribe to WebSocket connection state using React 18's efficient external store API
 * This hook ensures optimal performance and concurrent mode safety
 *
 * @param client - WebSocketClient instance
 * @returns Current connection state (true if connected, false if disconnected)
 *
 * @example
 * ```tsx
 * const isConnected = useWebSocketConnection(client);
 *
 * return (
 *   <div>
 *     Status: {isConnected ? 'Connected' : 'Disconnected'}
 *   </div>
 * );
 * ```
 */
export function useWebSocketConnection(client: WebSocketClient | null): boolean {
  // Subscribe callback - called by React when component mounts/updates
  const subscribe = useCallback(
    (callback: () => void) => {
      if (!client) {
        // No client, return no-op unsubscribe
        return () => {};
      }

      // Subscribe to connection changes and call React's callback
      const unsubscribe = client.onConnection(() => {
        callback();
      });

      return unsubscribe;
    },
    [client]
  );

  // Snapshot callback - called by React to get current value
  // CRITICAL: This must NOT use useCallback with dependencies!
  // Every call must return the CURRENT connection state
  const getSnapshot = () => {
    if (!client) return false;
    return client.isConnected();
  };

  // Server snapshot for SSR (always disconnected on server)
  const getServerSnapshot = () => {
    return false;
  };

  return useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot);
}
