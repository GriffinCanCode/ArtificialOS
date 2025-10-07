/**
 * Session Restoration Hook
 * Restores window states from saved sessions
 */

import { useCallback } from "react";
import { useActions } from "../../features/windows";
import { logger } from "../utils/monitoring/logger";

interface SavedWindow {
  id: string;
  app_id: string;
  title: string;
  window_id?: string;
  window_pos?: { x: number; y: number };
  window_size?: { width: number; height: number };
  ui_spec: any;
  icon?: string;
}

export function useSessionRestore() {
  const { open: openWindow, updatePosition, updateSize } = useActions();

  const restoreWindows = useCallback(
    async (windows: SavedWindow[]) => {
      logger.info("Restoring windows from session", {
        component: "useSessionRestore",
        count: windows.length,
      });

      // Restore each window
      for (const savedWindow of windows) {
        try {
          // Open window with saved UI spec
          const windowId = openWindow(
            savedWindow.app_id,
            savedWindow.title || "Untitled",
            savedWindow.ui_spec,
            savedWindow.icon || "📦"
          );

          // Restore window geometry if saved
          if (savedWindow.window_pos && savedWindow.window_size) {
            // Small delay to ensure window is created
            setTimeout(() => {
              updatePosition(windowId, {
                x: savedWindow.window_pos!.x,
                y: savedWindow.window_pos!.y,
              });
              updateSize(windowId, {
                width: savedWindow.window_size!.width,
                height: savedWindow.window_size!.height,
              });
            }, 50);
          }

          logger.debug("Window restored", {
            component: "useSessionRestore",
            windowId,
            appId: savedWindow.app_id,
          });
        } catch (error) {
          logger.error("Failed to restore window", error as Error, {
            component: "useSessionRestore",
            appId: savedWindow.app_id,
          });
        }
      }

      logger.info("Session restoration complete", {
        component: "useSessionRestore",
        restoredCount: windows.length,
      });
    },
    [openWindow, updatePosition, updateSize]
  );

  return { restoreWindows };
}
