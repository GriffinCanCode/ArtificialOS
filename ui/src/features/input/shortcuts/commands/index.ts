/**
 * Command Definitions
 * Central registry of all keyboard shortcuts and commands
 */

import { systemCommands } from "./system";
import { windowCommands } from "./window";
import { selectionCommands } from "./selection";
import { clipboardCommands } from "./clipboard";
import type { ShortcutConfig } from "../core/types";

// ============================================================================
// Export All Commands
// ============================================================================

export { systemCommands } from "./system";
export {
  windowCommands,
  createWindowCommands,
  type WindowActions
} from "./window";
export {
  selectionCommands,
  createSelectionCommands,
  type SelectionActions
} from "./selection";
export { clipboardCommands } from "./clipboard";

// ============================================================================
// All Commands
// ============================================================================

/**
 * All built-in commands
 */
export const allCommands: ShortcutConfig[] = [
  ...systemCommands,
  ...windowCommands,
  ...selectionCommands,
  ...clipboardCommands,
];

/**
 * Get commands by category
 */
export function getCommandsByCategory(
  category: ShortcutConfig["category"]
): ShortcutConfig[] {
  return allCommands.filter((cmd) => cmd.category === category);
}

/**
 * Get command by ID
 */
export function getCommandById(id: string): ShortcutConfig | undefined {
  return allCommands.find((cmd) => cmd.id === id);
}

/**
 * Create custom command
 */
export function createCommand(
  id: string,
  sequence: string,
  label: string,
  handler: ShortcutConfig["handler"],
  options?: Partial<ShortcutConfig>
): ShortcutConfig {
  return {
    id,
    sequence,
    label,
    handler,
    category: "custom",
    priority: "normal",
    scope: "global",
    customizable: true,
    enabled: true,
    ...options,
  };
}

