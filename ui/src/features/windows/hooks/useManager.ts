/**
 * Manager Hook
 * High-level window management interface
 */

import { useStore, useActions } from "../store/store";
import { useKeyboard } from "./useKeyboard";
import type { Window } from "../core/types";

export interface ManagerInterface {
  windows: Window[];
  visible: Window[];
  focused: Window | null;
  open: (appId: string, title: string, uiSpec: any, icon?: string) => string;
  close: (windowId: string) => void;
  minimize: (windowId: string) => void;
  restore: (windowId: string) => void;
  maximize: (windowId: string) => void;
  focus: (windowId: string) => void;
  toggle: (windowId: string) => void;
}

export function useManager(): ManagerInterface {
  const windows = useStore((state) => state.windows);
  const actions = useActions();

  // Enable keyboard shortcuts
  useKeyboard(windows, {
    onFocus: actions.focus,
    onMinimize: actions.minimize,
    onClose: actions.close,
  });

  // Computed values
  const visible = windows.filter((w) => !w.isMinimized);
  const focused = windows.find((w) => w.isFocused) || null;

  return {
    windows,
    visible,
    focused,
    open: actions.open,
    close: actions.close,
    minimize: actions.minimize,
    restore: actions.restore,
    maximize: actions.maximize,
    focus: actions.focus,
    toggle: actions.toggle,
  };
}
