/**
 * Sync Utilities
 * Backend synchronization helpers
 */

import type { Position, Size } from "../core/types";

export interface SyncPayload {
  window_id: string;
  position: Position;
  size: Size;
}

/**
 * Sync window state to backend
 */
export async function syncWindow(
  appId: string,
  windowId: string,
  position: Position,
  size: Size
): Promise<void> {
  // Backend expects integer values, so round all numbers
  const payload: SyncPayload = {
    window_id: windowId,
    position: {
      x: Math.round(position.x),
      y: Math.round(position.y),
    },
    size: {
      width: Math.round(size.width),
      height: Math.round(size.height),
    },
  };

  try {
    const response = await fetch(`http://localhost:8000/apps/${appId}/window`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      // 404 means app was already closed - this is expected during cleanup
      if (response.status === 404) {
        return;
      }
      console.error(`Failed to sync window state: ${response.status} ${response.statusText}`);
    }
  } catch (error) {
    // Fire-and-forget: log but don't throw
    console.error("Failed to sync window state:", error);
  }
}

/**
 * Debounced sync helper
 */
export function createSyncDebouncer(delay: number = 500) {
  let timeoutId: NodeJS.Timeout | null = null;

  return (appId: string, windowId: string, position: Position, size: Size) => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }

    timeoutId = setTimeout(() => {
      syncWindow(appId, windowId, position, size);
      timeoutId = null;
    }, delay);
  };
}
