/**
 * Window Commands
 * Window management keyboard shortcuts
 */

import type { ShortcutConfig } from "../core/types";

// ============================================================================
// Window Management Commands
// ============================================================================

export const windowCommands: ShortcutConfig[] = [
  {
    id: "window.close",
    sequence: "$mod+w",
    label: "Close Window",
    description: "Close the focused window",
    category: "window",
    priority: "high",
    scope: "window",
    handler: () => {
      // Will be implemented by window manager
    },
  },

  {
    id: "window.minimize",
    sequence: "$mod+m",
    label: "Minimize Window",
    description: "Minimize the focused window",
    category: "window",
    priority: "normal",
    scope: "window",
    handler: () => {
      // Will be implemented by window manager
    },
  },

  {
    id: "window.maximize",
    sequence: "$mod+Shift+m",
    label: "Maximize Window",
    description: "Maximize the focused window",
    category: "window",
    priority: "normal",
    scope: "window",
    handler: () => {
      // Will be implemented by window manager
    },
  },

  {
    id: "window.cycle.forward",
    sequence: "$mod+Tab",
    label: "Next Window",
    description: "Focus next window",
    category: "window",
    priority: "high",
    scope: "global",
    handler: () => {
      // Will be implemented by window manager
    },
  },

  {
    id: "window.cycle.backward",
    sequence: "$mod+Shift+Tab",
    label: "Previous Window",
    description: "Focus previous window",
    category: "window",
    priority: "high",
    scope: "global",
    handler: () => {
      // Will be implemented by window manager
    },
  },

  {
    id: "window.fullscreen",
    sequence: "$mod+Control+f",
    label: "Toggle Fullscreen",
    description: "Enter or exit fullscreen mode",
    category: "window",
    priority: "normal",
    scope: "window",
    handler: () => {
      // Will be implemented by window manager
    },
  },
];

