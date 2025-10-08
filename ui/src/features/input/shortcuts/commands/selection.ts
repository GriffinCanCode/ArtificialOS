/**
 * Selection Commands
 * Selection management keyboard shortcuts
 */

import type { ShortcutConfig } from "../core/types";

// ============================================================================
// Selection Commands
// ============================================================================

export const selectionCommands: ShortcutConfig[] = [
  {
    id: "selection.all",
    sequence: "$mod+a",
    label: "Select All",
    description: "Select all items",
    category: "selection",
    priority: "high",
    scope: "global",
    handler: () => {
      // Will be implemented by context
    },
  },

  {
    id: "selection.invert",
    sequence: "$mod+i",
    label: "Invert Selection",
    description: "Invert the current selection",
    category: "selection",
    priority: "normal",
    scope: "desktop",
    handler: () => {
      // Will be implemented by icon grid
    },
  },

  {
    id: "selection.clear",
    sequence: "Escape",
    label: "Clear Selection",
    description: "Clear all selections",
    category: "selection",
    priority: "normal",
    scope: "desktop",
    handler: () => {
      // Will be implemented by icon grid
    },
  },
];

