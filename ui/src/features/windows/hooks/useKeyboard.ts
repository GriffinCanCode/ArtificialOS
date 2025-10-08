/**
 * Keyboard Hook
 * Window keyboard shortcuts and navigation
 */

import { useShortcuts, useScope } from "../../input";
import type { Window } from "../core/types";

export interface KeyboardActions {
  onFocus: (windowId: string) => void;
  onMinimize: (windowId: string) => void;
  onClose: (windowId: string) => void;
}

export function useKeyboard(windows: Window[], actions: KeyboardActions) {
  // Activate window scope
  useScope("window");

  // Register window management shortcuts
  useShortcuts([
    {
      id: "window.cycle.forward",
      sequence: "$mod+Tab",
      label: "Next Window",
      description: "Focus next window",
      category: "window",
      scope: "global",
      priority: "high",
      handler: () => {
        const visible = windows.filter((w) => !w.isMinimized);
        if (visible.length === 0) return;

        const currentIndex = visible.findIndex((w) => w.isFocused);
        const nextIndex = (currentIndex + 1) % visible.length;
        const nextWindow = visible[nextIndex];

        if (nextWindow) {
          actions.onFocus(nextWindow.id);
        }
      },
    },
    {
      id: "window.cycle.backward",
      sequence: "$mod+Shift+Tab",
      label: "Previous Window",
      description: "Focus previous window",
      category: "window",
      scope: "global",
      priority: "high",
      handler: () => {
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
    {
      id: "window.minimize",
      sequence: "$mod+m",
      label: "Minimize Window",
      description: "Minimize the focused window",
      category: "window",
      scope: "window",
      priority: "high",
      handler: () => {
        const focused = windows.find((w) => w.isFocused && !w.isMinimized);
        if (focused) {
          actions.onMinimize(focused.id);
        }
      },
    },
    {
      id: "window.close",
      sequence: "$mod+w",
      label: "Close Window",
      description: "Close the focused window",
      category: "window",
      scope: "window",
      priority: "high",
      handler: () => {
        const focused = windows.find((w) => w.isFocused);
        if (focused) {
          actions.onClose(focused.id);
        }
      },
    },
  ]);
}
