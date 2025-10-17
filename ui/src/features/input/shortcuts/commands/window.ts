/**
 * Window Commands
 * Window management keyboard shortcuts with factory pattern
 */

import type { ShortcutConfig, ShortcutScope } from "../core/types";

// ============================================================================
// Window Actions Interface
// ============================================================================

/**
 * Interface for window management actions that components must implement
 */
export interface WindowActions {
  onClose: (windowId: string) => void;
  onMinimize: (windowId: string) => void;
  onMaximize?: (windowId: string) => void;
  onCycleForward: () => void;
  onCycleBackward: () => void;
  onFullscreen?: (windowId: string) => void;
}

// ============================================================================
// Window Commands Factory
// ============================================================================

/**
 * Create window management shortcuts wired to specific actions.
 * This factory pattern allows consistent window management across the app
 * while maintaining centralized command definitions.
 *
 * @param actions - Window management actions to wire shortcuts to
 * @param scope - Shortcut scope (default: "window")
 * @returns Array of configured window management shortcuts
 *
 * @example
 * ```typescript
 * // In WindowManager
 * useShortcuts(createWindowCommands({
 *   onClose: (id) => windowManager.close(id),
 *   onMinimize: (id) => windowManager.minimize(id),
 *   onCycleForward: () => windowManager.focusNext(),
 *   onCycleBackward: () => windowManager.focusPrev(),
 * }));
 * ```
 */
export function createWindowCommands(
  actions: WindowActions,
  focusedWindowId: string | null = null,
  scope: ShortcutScope = "window"
): ShortcutConfig[] {
  const commands: ShortcutConfig[] = [
    {
      id: `window.close.${scope}`,
      sequence: "$mod+w",
      label: "Close Window",
      description: "Close the focused window",
      category: "window",
      priority: "high",
      scope,
      handler: () => {
        if (focusedWindowId) {
          actions.onClose(focusedWindowId);
        }
      },
    },

    {
      id: `window.minimize.${scope}`,
      sequence: "$mod+m",
      label: "Minimize Window",
      description: "Minimize the focused window",
      category: "window",
      priority: "normal",
      scope,
      handler: () => {
        if (focusedWindowId) {
          actions.onMinimize(focusedWindowId);
        }
      },
    },

    {
      id: "window.cycle.forward",
      sequence: "$mod+Tab",
      label: "Next Window",
      description: "Focus next window",
      category: "window",
      priority: "high",
      scope: "global", // Always global
      handler: () => {
        actions.onCycleForward();
      },
    },

    {
      id: "window.cycle.backward",
      sequence: "$mod+Shift+Tab",
      label: "Previous Window",
      description: "Focus previous window",
      category: "window",
      priority: "high",
      scope: "global", // Always global
      handler: () => {
        actions.onCycleBackward();
      },
    },
  ];

  // Add optional commands if actions provided
  if (actions.onMaximize) {
    commands.push({
      id: `window.maximize.${scope}`,
      sequence: "$mod+Shift+m",
      label: "Maximize Window",
      description: "Maximize the focused window",
      category: "window",
      priority: "normal",
      scope,
      handler: () => {
        if (focusedWindowId) {
          actions.onMaximize!(focusedWindowId);
        }
      },
    });
  }

  if (actions.onFullscreen) {
    commands.push({
      id: `window.fullscreen.${scope}`,
      sequence: "$mod+Control+f",
      label: "Toggle Fullscreen",
      description: "Enter or exit fullscreen mode",
      category: "window",
      priority: "normal",
      scope,
      handler: () => {
        if (focusedWindowId) {
          actions.onFullscreen!(focusedWindowId);
        }
      },
    });
  }

  return commands;
}

// ============================================================================
// Legacy Export (for backwards compatibility)
// ============================================================================

/**
 * @deprecated Use createWindowCommands() factory instead
 */
export const windowCommands: ShortcutConfig[] = [];

