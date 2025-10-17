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
    allowInInput: true, // Allow native select all behavior in input fields
    handler: (event) => {
      // In input fields, allow native behavior (don't prevent default)
      const target = event.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable) {
        return false; // Return false to allow native browser behavior
      }
      // For other contexts (icon grids, file lists, etc.), will be implemented by context
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

