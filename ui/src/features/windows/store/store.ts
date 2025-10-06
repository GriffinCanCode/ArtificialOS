/**
 * Window Store
 * Zustand state management for windows
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { useShallow } from "zustand/react/shallow";
import type { Window, Position, Size, Bounds, State } from "../core/types";
import { DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, State as WinState } from "../core/types";
import { getMaximizedBounds, getCascadePosition } from "../core/bounds";
import type { Blueprint } from "../../../core/store/appStore";
import { filterValidComponents } from "../../../core/utils/blueprintParser";

// ============================================================================
// Store Interface
// ============================================================================

interface Store {
  windows: Window[];
  nextZIndex: number;

  // Actions
  open: (appId: string, title: string, uiSpec: Blueprint, icon?: string) => string;
  close: (windowId: string) => void;
  minimize: (windowId: string) => void;
  restore: (windowId: string) => void;
  maximize: (windowId: string) => void;
  unmaximize: (windowId: string) => void;
  toggle: (windowId: string) => void;
  focus: (windowId: string) => void;
  updatePosition: (windowId: string, position: Position) => void;
  updateSize: (windowId: string, size: Size) => void;
  updateBounds: (windowId: string, bounds: Bounds) => void;
  updateState: (windowId: string, state: State) => void;
  setAnimating: (windowId: string, isAnimating: boolean) => void;
  update: (windowId: string, updates: Partial<Omit<Window, "id">>) => void;
  get: (windowId: string) => Window | undefined;
  clearAll: () => void;
}

// ============================================================================
// Store Implementation
// ============================================================================

export const useStore = create<Store>()(
  devtools(
    (set, get) => ({
      windows: [],
      nextZIndex: 1000,

      open: (appId, title, uiSpec, icon) => {
        const windowId = `window-${appId}-${Date.now()}`;
        const state = get();

        const existingWindows = state.windows.filter((w) => !w.isMinimized);
        const position = getCascadePosition(existingWindows.length);

        // Filter and convert components (row -> container, etc.)
        const filteredComponents = filterValidComponents(uiSpec.components || []);
        const filteredSpec = {
          ...uiSpec,
          components: filteredComponents,
        };

        const newWindow: Window = {
          id: windowId,
          appId,
          title,
          icon,
          uiSpec: filteredSpec,
          position,
          size: { width: DEFAULT_WINDOW_WIDTH, height: DEFAULT_WINDOW_HEIGHT },
          isMinimized: false,
          isFocused: true,
          zIndex: state.nextZIndex,
          state: WinState.NORMAL,
          metadata: {
            isAnimating: false,
            childWindowIds: [],
          },
        };

        set(
          (state) => ({
            windows: [...state.windows.map((w) => ({ ...w, isFocused: false })), newWindow],
            nextZIndex: state.nextZIndex + 1,
          }),
          false,
          "open"
        );

        return windowId;
      },

      close: (windowId) => {
        set(
          (state) => {
            const remaining = state.windows.filter((w) => w.id !== windowId);
            const closed = state.windows.find((w) => w.id === windowId);

            if (closed?.isFocused && remaining.length > 0) {
              const topWindow = remaining.reduce((max, w) => (w.zIndex > max.zIndex ? w : max));
              topWindow.isFocused = true;
            }

            return { windows: remaining };
          },
          false,
          "close"
        );
      },

      minimize: (windowId) => {
        set(
          (state) => ({
            windows: state.windows.map((w) =>
              w.id === windowId ? { ...w, isMinimized: true, isFocused: false } : w
            ),
          }),
          false,
          "minimize"
        );
      },

      restore: (windowId) => {
        set(
          (state) => ({
            windows: state.windows.map((w) =>
              w.id === windowId
                ? { ...w, isMinimized: false, isFocused: true }
                : { ...w, isFocused: false }
            ),
            nextZIndex: state.nextZIndex + 1,
          }),
          false,
          "restore"
        );
      },

      maximize: (windowId) => {
        set(
          (state) => {
            const window = state.windows.find((w) => w.id === windowId);
            if (!window) return state;

            const maxBounds = getMaximizedBounds();

            return {
              windows: state.windows.map((w) =>
                w.id === windowId
                  ? {
                      ...w,
                      state: WinState.MAXIMIZED,
                      position: maxBounds.position,
                      size: maxBounds.size,
                      metadata: {
                        ...w.metadata,
                        lastNormalBounds: {
                          position: window.position,
                          size: window.size,
                        },
                      },
                    }
                  : w
              ),
            };
          },
          false,
          "maximize"
        );
      },

      unmaximize: (windowId) => {
        set(
          (state) => ({
            windows: state.windows.map((w) => {
              if (w.id !== windowId) return w;

              const lastBounds = w.metadata.lastNormalBounds;
              return {
                ...w,
                state: WinState.NORMAL,
                position: lastBounds?.position || w.position,
                size: lastBounds?.size || w.size,
              };
            }),
          }),
          false,
          "unmaximize"
        );
      },

      toggle: (windowId) => {
        const window = get().windows.find((w) => w.id === windowId);
        if (!window) return;

        if (window.state === WinState.MAXIMIZED) {
          get().unmaximize(windowId);
        } else {
          get().maximize(windowId);
        }
      },

      focus: (windowId) => {
        set(
          (state) => ({
            windows: state.windows.map((w) =>
              w.id === windowId
                ? { ...w, isFocused: true, zIndex: state.nextZIndex }
                : { ...w, isFocused: false }
            ),
            nextZIndex: state.nextZIndex + 1,
          }),
          false,
          "focus"
        );
      },

      updatePosition: (windowId, position) => {
        set(
          (state) => ({
            windows: state.windows.map((w) => (w.id === windowId ? { ...w, position } : w)),
          }),
          false,
          "updatePosition"
        );
      },

      updateSize: (windowId, size) => {
        set(
          (state) => ({
            windows: state.windows.map((w) => (w.id === windowId ? { ...w, size } : w)),
          }),
          false,
          "updateSize"
        );
      },

      updateBounds: (windowId, bounds) => {
        set(
          (state) => ({
            windows: state.windows.map((w) =>
              w.id === windowId ? { ...w, position: bounds.position, size: bounds.size } : w
            ),
          }),
          false,
          "updateBounds"
        );
      },

      updateState: (windowId, state) => {
        set(
          (s) => ({
            windows: s.windows.map((w) => (w.id === windowId ? { ...w, state } : w)),
          }),
          false,
          "updateState"
        );
      },

      setAnimating: (windowId, isAnimating) => {
        set(
          (state) => ({
            windows: state.windows.map((w) =>
              w.id === windowId ? { ...w, metadata: { ...w.metadata, isAnimating } } : w
            ),
          }),
          false,
          "setAnimating"
        );
      },

      update: (windowId, updates) => {
        set(
          (state) => ({
            windows: state.windows.map((w) => (w.id === windowId ? { ...w, ...updates } : w)),
          }),
          false,
          "update"
        );
      },

      get: (windowId) => {
        return get().windows.find((w) => w.id === windowId);
      },

      clearAll: () => {
        set({ windows: [], nextZIndex: 1000 }, false, "clearAll");
      },
    }),
    { name: "WindowStore" }
  )
);

// ============================================================================
// Convenience Hooks
// ============================================================================

// Stable selector function to prevent infinite loops
const actionsSelector = (state: Store) => ({
  open: state.open,
  close: state.close,
  minimize: state.minimize,
  restore: state.restore,
  maximize: state.maximize,
  unmaximize: state.unmaximize,
  toggle: state.toggle,
  focus: state.focus,
  updatePosition: state.updatePosition,
  updateSize: state.updateSize,
  updateBounds: state.updateBounds,
  updateState: state.updateState,
  setAnimating: state.setAnimating,
  update: state.update,
  get: state.get,
  clearAll: state.clearAll,
});

export function useActions() {
  // Use shallow comparison to prevent re-renders when actions haven't changed
  return useStore(useShallow(actionsSelector));
}

export function useVisibleWindows() {
  return useStore(useShallow((state) => state.windows.filter((w) => !w.isMinimized)));
}

export function useFocusedWindow() {
  return useStore((state) => state.windows.find((w) => w.isFocused) || null);
}
