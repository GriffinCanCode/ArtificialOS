/**
 * Keyboard Hook
 * Window keyboard shortcuts using factory pattern
 */

import { useShortcuts, useScope, createWindowCommands } from "../../input";
import type { Window } from "../core/types";

export interface KeyboardActions {
  onFocus: (windowId: string) => void;
  onMinimize: (windowId: string) => void;
  onClose: (windowId: string) => void;
}

export function useKeyboard(windows: Window[], actions: KeyboardActions) {
  // Activate window scope
  useScope("window");

  // Get focused window ID
  const focusedWindow = windows.find((w) => w.isFocused);
  const focusedWindowId = focusedWindow?.id || null;

  // Register window management shortcuts using factory
  useShortcuts(
    createWindowCommands(
      {
        onClose: actions.onClose,
        onMinimize: actions.onMinimize,
        onCycleForward: () => {
          // Cycle forward through visible windows
          const visible = windows.filter((w) => !w.isMinimized);
          if (visible.length === 0) return;

          const currentIndex = visible.findIndex((w) => w.isFocused);
          const nextIndex = (currentIndex + 1) % visible.length;
          const nextWindow = visible[nextIndex];

          if (nextWindow) {
            actions.onFocus(nextWindow.id);
          }
        },
        onCycleBackward: () => {
          // Cycle backward through visible windows
          const visible = windows.filter((w) => !w.isMinimized);
          if (visible.length === 0) return;

          const currentIndex = visible.findIndex((w) => w.isFocused);
          const prevIndex = currentIndex <= 0 ? visible.length - 1 : currentIndex - 1;
          const prevWindow = visible[prevIndex];

          if (prevWindow) {
            actions.onFocus(prevWindow.id);
          }
        },
      },
      focusedWindowId,
      "window"
    )
  );
}
