/**
 * useKeyboard Hook
 * Keyboard navigation and shortcuts for icons
 */

import { useEffect, useCallback } from "react";
import { useActions, useIcons, useSelectedIcons, useSearchState } from "../store/store";
import { shouldIgnoreKeyboardEvent } from "../../input";
import type { Icon, GridPosition } from "../core/types";

// ============================================================================
// Keyboard Hook Interface
// ============================================================================

export interface KeyboardOptions {
  onSearch?: (query: string) => void;
  onEscape?: () => void;
  disabled?: boolean;
}

// ============================================================================
// Navigation Helpers
// ============================================================================

/**
 * Find icon at grid position
 */
function findIconAt(icons: Icon[], position: GridPosition): Icon | undefined {
  return icons.find((icon) => icon.position.row === position.row && icon.position.col === position.col);
}

/**
 * Get next icon in direction
 */
function getNextIcon(icons: Icon[], current: Icon, direction: "up" | "down" | "left" | "right"): Icon | undefined {
  const deltas: Record<string, { row: number; col: number }> = {
    up: { row: -1, col: 0 },
    down: { row: 1, col: 0 },
    left: { row: 0, col: -1 },
    right: { row: 0, col: 1 },
  };

  const delta = deltas[direction];
  let nextPos = {
    row: current.position.row + delta.row,
    col: current.position.col + delta.col,
  };

  // Keep searching in direction until we find an icon or hit bounds
  for (let i = 0; i < 20; i++) {
    const icon = findIconAt(icons, nextPos);
    if (icon) {
      return icon;
    }

    // Move further in direction
    nextPos = {
      row: nextPos.row + delta.row,
      col: nextPos.col + delta.col,
    };

    // Bounds check
    if (nextPos.row < 0 || nextPos.col < 0) {
      break;
    }
  }

  return undefined;
}

/**
 * Get first/last icon
 */
function getFirstIcon(icons: Icon[]): Icon | undefined {
  return [...icons].sort((a, b) => {
    if (a.position.row !== b.position.row) {
      return a.position.row - b.position.row;
    }
    return a.position.col - b.position.col;
  })[0];
}

function getLastIcon(icons: Icon[]): Icon | undefined {
  const sorted = [...icons].sort((a, b) => {
    if (a.position.row !== b.position.row) {
      return b.position.row - a.position.row;
    }
    return b.position.col - a.position.col;
  });
  return sorted[0];
}

// ============================================================================
// Keyboard Hook
// ============================================================================

/**
 * Keyboard navigation and shortcuts for icons
 */
export function useKeyboard(options: KeyboardOptions = {}) {
  const icons = useIcons();
  const selectedIcons = useSelectedIcons();
  const searchState = useSearchState();
  const { select, selectAll, clearSelection, autoArrange, setSearchQuery } = useActions();

  // Get current focused icon (last selected)
  const focusedIcon = selectedIcons.length > 0 ? selectedIcons[selectedIcons.length - 1] : null;

  // Handle arrow key navigation
  const handleArrowKey = useCallback(
    (direction: "up" | "down" | "left" | "right", shiftKey: boolean) => {
      if (options.disabled) return;

      const current = focusedIcon || getFirstIcon(icons);
      if (!current) return;

      const next = getNextIcon(icons, current, direction);
      if (!next) return;

      // Select next icon (with or without multi-select)
      select(next.id, shiftKey);
    },
    [focusedIcon, icons, select, options.disabled]
  );

  // Handle keyboard events
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Allow Escape and Cmd+A even in inputs
      const allowedKeys = ["Escape", "a", "A"];
      const isAllowed = allowedKeys.includes(e.key) && (e.metaKey || e.ctrlKey || e.key === "Escape");

      if (!isAllowed && shouldIgnoreKeyboardEvent(e)) {
        return; // Let input handle the keystroke
      }

      if (options.disabled) return;

      // Arrow keys - Navigate
      if (e.key === "ArrowUp") {
        e.preventDefault();
        handleArrowKey("up", e.shiftKey);
        return;
      }

      if (e.key === "ArrowDown") {
        e.preventDefault();
        handleArrowKey("down", e.shiftKey);
        return;
      }

      if (e.key === "ArrowLeft") {
        e.preventDefault();
        handleArrowKey("left", e.shiftKey);
        return;
      }

      if (e.key === "ArrowRight") {
        e.preventDefault();
        handleArrowKey("right", e.shiftKey);
        return;
      }

      // Cmd/Ctrl + A - Select all
      if ((e.metaKey || e.ctrlKey) && e.key === "a") {
        e.preventDefault();
        selectAll();
        return;
      }

      // Escape - Clear selection or exit search
      if (e.key === "Escape") {
        e.preventDefault();
        if (searchState.isActive) {
          setSearchQuery("");
          if (options.onEscape) {
            options.onEscape();
          }
        } else {
          clearSelection();
        }
        return;
      }

      // Cmd/Ctrl + I - Invert selection
      if ((e.metaKey || e.ctrlKey) && e.key === "i") {
        e.preventDefault();
        const allIds = icons.map((i) => i.id);
        const selectedIds = new Set(selectedIcons.map((i) => i.id));
        const invertedIds = allIds.filter((id) => !selectedIds.has(id));

        clearSelection();
        for (const id of invertedIds) {
          select(id, true);
        }
        return;
      }

      // Cmd/Ctrl + Shift + A - Auto-arrange
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "A") {
        e.preventDefault();
        autoArrange("grid");
        return;
      }

      // Home - Select first icon
      if (e.key === "Home") {
        e.preventDefault();
        const first = getFirstIcon(icons);
        if (first) {
          select(first.id, e.shiftKey);
        }
        return;
      }

      // End - Select last icon
      if (e.key === "End") {
        e.preventDefault();
        const last = getLastIcon(icons);
        if (last) {
          select(last.id, e.shiftKey);
        }
        return;
      }

      // Cmd/Ctrl + F - Focus search (if handler provided)
      if ((e.metaKey || e.ctrlKey) && e.key === "f" && options.onSearch) {
        e.preventDefault();
        options.onSearch("");
        return;
      }

      // Delete/Backspace - Delete selected icons (requires confirmation in UI)
      // Not implemented here to prevent accidental deletion
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    handleArrowKey,
    selectAll,
    clearSelection,
    autoArrange,
    select,
    icons,
    selectedIcons,
    searchState.isActive,
    setSearchQuery,
    options,
  ]);

  return {
    focusedIcon,
    hasSelection: selectedIcons.length > 0,
  };
}

