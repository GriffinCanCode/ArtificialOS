/**
 * useSelect Hook
 * Icon selection management with keyboard modifiers
 */

import { useCallback } from "react";
import { useActions, useSelectedIcons, useAnchorId } from "../store/store";

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
  const anchorId = useAnchorId();
  const { select, selectRange, deselect, selectAll, clearSelection } = useActions();

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

      // Range selection (Shift+Click)
      if (shift && anchorId) {
        selectRange(anchorId, iconId);
        return;
      }

      // Single selection
      select(iconId, false);
    },
    [isSelected, select, selectRange, deselect, anchorId]
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

  return {
    selectedIds,
    isSelected,
    select: handleSelect,
    deselect,
    toggle,
    selectAll,
    clearSelection,
    selectRange, // Use the store's selectRange action directly
  };
}

