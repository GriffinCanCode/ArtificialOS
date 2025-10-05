/**
 * Window Management Store
 * Manages multiple windowed applications
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { useMemo } from "react";
import { logger } from "../utils/monitoring/logger";
import { Blueprint } from "./appStore";

// ============================================================================
// Type Definitions
// ============================================================================

export interface WindowPosition {
  x: number;
  y: number;
}

export interface WindowSize {
  width: number;
  height: number;
}

export interface WindowState {
  id: string;
  appId: string;
  title: string;
  icon?: string;
  uiSpec: Blueprint;
  position: WindowPosition;
  size: WindowSize;
  isMinimized: boolean;
  isFocused: boolean;
  zIndex: number;
}

interface WindowStore {
  windows: WindowState[];
  nextZIndex: number;
  
  // Actions
  openWindow: (appId: string, title: string, uiSpec: Blueprint, icon?: string) => string;
  closeWindow: (windowId: string) => void;
  minimizeWindow: (windowId: string) => void;
  restoreWindow: (windowId: string) => void;
  focusWindow: (windowId: string) => void;
  updateWindowPosition: (windowId: string, position: WindowPosition) => void;
  updateWindowSize: (windowId: string, size: WindowSize) => void;
  getWindow: (windowId: string) => WindowState | undefined;
  clearAllWindows: () => void;
}

// ============================================================================
// Initial State
// ============================================================================

const initialState = {
  windows: [],
  nextZIndex: 1000,
};

// ============================================================================
// Store
// ============================================================================

export const useWindowStore = create<WindowStore>()(
  devtools(
    (set, get) => ({
      ...initialState,

      openWindow: (appId, title, uiSpec, icon) => {
        const windowId = `window-${appId}-${Date.now()}`;
        const state = get();
        
        // Calculate position (cascade windows slightly)
        const existingWindows = state.windows.filter(w => !w.isMinimized);
        const offset = existingWindows.length * 30;
        
        const newWindow: WindowState = {
          id: windowId,
          appId,
          title,
          icon,
          uiSpec,
          position: { x: 100 + offset, y: 80 + offset },
          size: { width: 800, height: 600 },
          isMinimized: false,
          isFocused: true,
          zIndex: state.nextZIndex,
        };

        logger.info("Opening window", {
          component: "WindowStore",
          windowId,
          appId,
          title,
        });

        set(
          (state) => ({
            windows: [
              ...state.windows.map(w => ({ ...w, isFocused: false })),
              newWindow,
            ],
            nextZIndex: state.nextZIndex + 1,
          }),
          false,
          "openWindow"
        );

        return windowId;
      },

      closeWindow: (windowId) => {
        logger.info("Closing window", {
          component: "WindowStore",
          windowId,
        });

        set(
          (state) => {
            const remainingWindows = state.windows.filter(w => w.id !== windowId);
            
            // Focus the topmost window if we closed the focused one
            const closedWindow = state.windows.find(w => w.id === windowId);
            if (closedWindow?.isFocused && remainingWindows.length > 0) {
              const topWindow = remainingWindows.reduce((max, w) => 
                w.zIndex > max.zIndex ? w : max
              );
              topWindow.isFocused = true;
            }

            return { windows: remainingWindows };
          },
          false,
          "closeWindow"
        );
      },

      minimizeWindow: (windowId) => {
        logger.info("Minimizing window", {
          component: "WindowStore",
          windowId,
        });

        set(
          (state) => ({
            windows: state.windows.map(w =>
              w.id === windowId
                ? { ...w, isMinimized: true, isFocused: false }
                : w
            ),
          }),
          false,
          "minimizeWindow"
        );
      },

      restoreWindow: (windowId) => {
        logger.info("Restoring window", {
          component: "WindowStore",
          windowId,
        });

        set(
          (state) => ({
            windows: state.windows.map(w =>
              w.id === windowId
                ? { ...w, isMinimized: false, isFocused: true, zIndex: state.nextZIndex }
                : { ...w, isFocused: false }
            ),
            nextZIndex: state.nextZIndex + 1,
          }),
          false,
          "restoreWindow"
        );
      },

      focusWindow: (windowId) => {
        const state = get();
        const window = state.windows.find(w => w.id === windowId);
        
        if (!window || window.isFocused) return;

        logger.debug("Focusing window", {
          component: "WindowStore",
          windowId,
        });

        set(
          (state) => ({
            windows: state.windows.map(w =>
              w.id === windowId
                ? { ...w, isFocused: true, zIndex: state.nextZIndex }
                : { ...w, isFocused: false }
            ),
            nextZIndex: state.nextZIndex + 1,
          }),
          false,
          "focusWindow"
        );
      },

      updateWindowPosition: (windowId, position) => {
        set(
          (state) => ({
            windows: state.windows.map(w =>
              w.id === windowId ? { ...w, position } : w
            ),
          }),
          false,
          "updateWindowPosition"
        );
      },

      updateWindowSize: (windowId, size) => {
        set(
          (state) => ({
            windows: state.windows.map(w =>
              w.id === windowId ? { ...w, size } : w
            ),
          }),
          false,
          "updateWindowSize"
        );
      },

      getWindow: (windowId) => {
        return get().windows.find(w => w.id === windowId);
      },

      clearAllWindows: () => {
        logger.info("Clearing all windows", { component: "WindowStore" });
        set(initialState, false, "clearAllWindows");
      },
    }),
    { name: "WindowStore" }
  )
);

// ============================================================================
// Selectors
// ============================================================================

// Use stable selectors to prevent unnecessary re-renders
export const useWindows = () => useWindowStore((state) => state.windows);

// These selectors use useMemo internally via the component that calls them
// We select the minimal state needed to compute the derived value
export const useMinimizedWindows = () => {
  const windows = useWindowStore((state) => state.windows);
  return useMemo(() => windows.filter(w => w.isMinimized), [windows]);
};

export const useVisibleWindows = () => {
  const windows = useWindowStore((state) => state.windows);
  return useMemo(() => windows.filter(w => !w.isMinimized), [windows]);
};

export const useWindowActions = () => {
  const openWindow = useWindowStore((state) => state.openWindow);
  const closeWindow = useWindowStore((state) => state.closeWindow);
  const minimizeWindow = useWindowStore((state) => state.minimizeWindow);
  const restoreWindow = useWindowStore((state) => state.restoreWindow);
  const focusWindow = useWindowStore((state) => state.focusWindow);
  const updateWindowPosition = useWindowStore((state) => state.updateWindowPosition);
  const updateWindowSize = useWindowStore((state) => state.updateWindowSize);
  const getWindow = useWindowStore((state) => state.getWindow);
  const clearAllWindows = useWindowStore((state) => state.clearAllWindows);

  return useMemo(
    () => ({
      openWindow,
      closeWindow,
      minimizeWindow,
      restoreWindow,
      focusWindow,
      updateWindowPosition,
      updateWindowSize,
      getWindow,
      clearAllWindows,
    }),
    [
      openWindow,
      closeWindow,
      minimizeWindow,
      restoreWindow,
      focusWindow,
      updateWindowPosition,
      updateWindowSize,
      getWindow,
      clearAllWindows,
    ]
  );
};

