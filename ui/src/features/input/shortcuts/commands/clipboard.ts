/**
 * Clipboard Commands
 * Clipboard operation keyboard shortcuts
 */

import type { ShortcutConfig } from "../core/types";

// ============================================================================
// Clipboard Commands
// ============================================================================

export const clipboardCommands: ShortcutConfig[] = [
  {
    id: "clipboard.copy",
    sequence: "$mod+c",
    label: "Copy",
    description: "Copy selected items to clipboard",
    category: "clipboard",
    priority: "high",
    scope: "global",
    handler: () => {
      // Will be implemented by context
    },
  },

  {
    id: "clipboard.cut",
    sequence: "$mod+x",
    label: "Cut",
    description: "Cut selected items to clipboard",
    category: "clipboard",
    priority: "high",
    scope: "global",
    handler: () => {
      // Will be implemented by context
    },
  },

  {
    id: "clipboard.paste",
    sequence: "$mod+v",
    label: "Paste",
    description: "Paste from clipboard",
    category: "clipboard",
    priority: "high",
    scope: "global",
    handler: () => {
      // Will be implemented by context
    },
  },

  {
    id: "clipboard.viewer.toggle",
    sequence: "$mod+Shift+v",
    label: "Toggle Clipboard Viewer",
    description: "Open or close clipboard history viewer",
    category: "clipboard",
    priority: "normal",
    scope: "global",
    handler: () => {
      // Will be implemented by clipboard viewer
    },
  },
];

