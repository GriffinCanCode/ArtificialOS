/**
 * Selection Commands
 * Selection management keyboard shortcuts with factory pattern
 */

import type { ShortcutConfig, ShortcutScope } from "../core/types";

// ============================================================================
// Selection Actions Interface
// ============================================================================

/**
 * Interface for selection actions that components must implement
 */
export interface SelectionActions {
  selectAll: () => void;
  clearSelection: () => void;
  invertSelection?: () => void; // Optional - not all contexts support invert
}

// ============================================================================
// Selection Commands Factory
// ============================================================================

/**
 * Create selection command shortcuts wired to specific actions.
 * This factory pattern allows each component to register selection shortcuts
 * with its own implementation while maintaining centralized command definitions.
 *
 * @param actions - Selection actions to wire shortcuts to
 * @param scope - Shortcut scope (default: "desktop")
 * @returns Array of configured selection shortcuts
 *
 * @example
 * ```typescript
 * // In Desktop component
 * const iconActions = useIconActions();
 * useShortcuts(createSelectionCommands({
 *   selectAll: iconActions.selectAll,
 *   clearSelection: iconActions.clearSelection,
 *   invertSelection: () => { ... }
 * }));
 * ```
 */
export function createSelectionCommands(
  actions: SelectionActions,
  scope: ShortcutScope = "desktop"
): ShortcutConfig[] {
  const commands: ShortcutConfig[] = [
    {
      id: `selection.all.${scope}`,
      sequence: "$mod+a",
      label: "Select All",
      description: "Select all items in current context",
      category: "selection",
      priority: "high",
      scope,
      allowInInput: true, // Allow native behavior in input fields
      handler: (event) => {
        // In input fields, allow native browser behavior
        const target = event.target as HTMLElement;
        if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)) {
          return false; // Don't prevent default - let browser handle it
        }
        // Otherwise, call context-specific selectAll
        actions.selectAll();
      },
    },

    {
      id: `selection.clear.${scope}`,
      sequence: "Escape",
      label: "Clear Selection",
      description: "Clear all selections",
      category: "selection",
      priority: "normal",
      scope,
      allowInInput: false,
      handler: () => {
        actions.clearSelection();
      },
    },
  ];

  // Add invert command only if action is provided
  if (actions.invertSelection) {
    commands.push({
      id: `selection.invert.${scope}`,
      sequence: "$mod+i",
      label: "Invert Selection",
      description: "Invert the current selection",
      category: "selection",
      priority: "normal",
      scope,
      allowInInput: false,
      handler: () => {
        actions.invertSelection!();
      },
    });
  }

  return commands;
}

// ============================================================================
// Legacy Export (for backwards compatibility)
// ============================================================================

/**
 * @deprecated Use createSelectionCommands() factory instead
 */
export const selectionCommands: ShortcutConfig[] = [];

