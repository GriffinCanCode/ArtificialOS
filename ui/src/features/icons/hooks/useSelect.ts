/**
 * useSelect Hook
 * Icon selection management with keyboard modifiers
 */

import { useCallback } from "react";
import { useActions, useSelectedIcons } from "../store/store";

// ============================================================================
// Selection Hook
// ============================================================================

export interface SelectHook {
  selectedIds: Set<string>;
  isSelected: (iconId: string) => boolean;
  select: (iconId: string, modifiers?: SelectModifiers) => void;
  deselect: (iconId: string) => void;
  toggle: (iconId: string) => void;
  selectAll: () => void;
  clearSelection: () => void;
  selectRange: (startId: string, endId: string) => void;
}

export interface SelectModifiers {
  shift?: boolean; // Range selection
  ctrl?: boolean; // Toggle selection
  meta?: boolean; // Toggle selection (Mac)
}

/**
 * Selection management hook
 * Handles single, multi, and range selection
 */
export function useSelect(): SelectHook {
  const selectedIcons = useSelectedIcons();
  const { select, deselect, selectAll, clearSelection } = useActions();

  const selectedIds = new Set(selectedIcons.map((i) => i.id));

  const isSelected = useCallback(
    (iconId: string) => {
      return selectedIds.has(iconId);
    },
    [selectedIds]
  );

  const handleSelect = useCallback(
    (iconId: string, modifiers: SelectModifiers = {}) => {
      const { shift, ctrl, meta } = modifiers;

      // Toggle selection (Cmd/Ctrl+Click)
      if (ctrl || meta) {
        if (isSelected(iconId)) {
          deselect(iconId);
        } else {
          select(iconId, true); // Multi-select
        }
        return;
      }

      // Range selection (Shift+Click) - simplified version
      if (shift && selectedIds.size > 0) {
        // TODO: Implement range selection based on grid positions
        select(iconId, true);
        return;
      }

      // Single selection
      select(iconId, false);
    },
    [isSelected, select, deselect, selectedIds]
  );

  const toggle = useCallback(
    (iconId: string) => {
      if (isSelected(iconId)) {
        deselect(iconId);
      } else {
        select(iconId, true);
      }
    },
    [isSelected, select, deselect]
  );

  const selectRange = useCallback(
    (startId: string, endId: string) => {
      // TODO: Implement range selection based on grid traversal
      select(startId, false);
      select(endId, true);
    },
    [select]
  );

  return {
    selectedIds,
    isSelected,
    select: handleSelect,
    deselect,
    toggle,
    selectAll,
    clearSelection,
    selectRange,
  };
}

