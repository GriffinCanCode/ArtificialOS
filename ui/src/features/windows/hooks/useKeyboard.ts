/**
 * Keyboard Hook
 * Window keyboard shortcuts and navigation
 */

import { useEffect } from "react";
import { shouldIgnoreKeyboardEvent } from "../../input";
import type { Window } from "../core/types";

export interface KeyboardActions {
  onFocus: (windowId: string) => void;
  onMinimize: (windowId: string) => void;
  onClose: (windowId: string) => void;
}

export function useKeyboard(windows: Window[], actions: KeyboardActions) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't trigger if typing in input
      if (shouldIgnoreKeyboardEvent(e)) {
        return;
      }

      // Alt/Cmd + Tab: Cycle forward
      if ((e.altKey || e.metaKey) && e.key === "Tab" && !e.shiftKey) {
        e.preventDefault();

        const visible = windows.filter((w) => !w.isMinimized);
        if (visible.length === 0) return;

        const currentIndex = visible.findIndex((w) => w.isFocused);
        const nextIndex = (currentIndex + 1) % visible.length;
        const nextWindow = visible[nextIndex];

        if (nextWindow) {
          actions.onFocus(nextWindow.id);
        }
      }

      // Alt/Cmd + Shift + Tab: Cycle backward
      if ((e.altKey || e.metaKey) && e.shiftKey && e.key === "Tab") {
        e.preventDefault();

        const visible = windows.filter((w) => !w.isMinimized);
        if (visible.length === 0) return;

        const currentIndex = visible.findIndex((w) => w.isFocused);
        const prevIndex = currentIndex <= 0 ? visible.length - 1 : currentIndex - 1;
        const prevWindow = visible[prevIndex];

        if (prevWindow) {
          actions.onFocus(prevWindow.id);
        }
      }

      // Cmd/Ctrl + M: Minimize focused
      if ((e.metaKey || e.ctrlKey) && e.key === "m") {
        e.preventDefault();

        const focused = windows.find((w) => w.isFocused && !w.isMinimized);
        if (focused) {
          actions.onMinimize(focused.id);
        }
      }

      // Cmd/Ctrl + W: Close focused
      if ((e.metaKey || e.ctrlKey) && e.key === "w") {
        e.preventDefault();

        const focused = windows.find((w) => w.isFocused);
        if (focused) {
          actions.onClose(focused.id);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [windows, actions]);
}
