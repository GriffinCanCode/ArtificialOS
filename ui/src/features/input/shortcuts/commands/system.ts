/**
 * System Commands
 * System-level keyboard shortcuts and commands
 */

import type { ShortcutConfig } from "../core/types";

// ============================================================================
// System Commands
// ============================================================================

export const systemCommands: ShortcutConfig[] = [
  {
    id: "system.creator.toggle",
    sequence: "$mod+k",
    label: "Toggle Creator",
    description: "Open or close the app creator",
    category: "system",
    priority: "high",
    scope: "global",
    handler: () => {
      // Will be implemented by App.tsx
    },
  },

  {
    id: "system.hub.open",
    sequence: "$mod+Space",
    label: "Open Hub",
    description: "Open the application hub",
    category: "system",
    priority: "high",
    scope: "desktop",
    handler: () => {
      // Will be implemented by Desktop.tsx
    },
  },

  {
    id: "system.escape",
    sequence: "Escape",
    label: "Escape",
    description: "Close modal, clear selection, or exit mode",
    category: "system",
    priority: "critical",
    scope: "global",
    allowInInput: true,
    handler: () => {
      // Context-aware handler
    },
  },
];

