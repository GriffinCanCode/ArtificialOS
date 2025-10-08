/**
 * Icon Context Menu
 * Right-click menu for desktop icons
 *
 * Note: This is a simplified wrapper. For full context menu support,
 * integrate with the floating feature's ContextMenu component.
 */

import React from "react";
import type { Icon } from "../core/types";

// ============================================================================
// Context Menu Props
// ============================================================================

export interface IconContextMenuProps {
  icon: Icon;
  onOpen?: (icon: Icon) => void;
  onRename?: (icon: Icon) => void;
  onDelete?: (icon: Icon) => void;
}

// ============================================================================
// Context Menu Component (Placeholder)
// ============================================================================

/**
 * Placeholder for context menu functionality
 * TODO: Integrate with floating feature's ContextMenu component
 */
export const IconContextMenu: React.FC<IconContextMenuProps> = () => {
  // This is a placeholder component
  // Full implementation requires integration with floating UI
  return null;
};

IconContextMenu.displayName = "IconContextMenu";

