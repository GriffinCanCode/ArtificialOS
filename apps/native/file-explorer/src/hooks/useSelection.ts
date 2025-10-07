/**
 * Selection Hook
 * Manages file selection state and multi-select operations
 */

import { useState, useCallback } from 'react';
import type { FileEntry, UseSelectionReturn } from '../types';

export function useSelection(entries: FileEntry[]): UseSelectionReturn {
  const [selected, setSelected] = useState<Set<string>>(new Set());

  /**
   * Check if path is selected
   */
  const isSelected = useCallback((path: string): boolean => {
    return selected.has(path);
  }, [selected]);

  /**
   * Toggle selection (supports Ctrl/Cmd for multi-select)
   */
  const toggle = useCallback((path: string, event?: React.MouseEvent) => {
    setSelected(prev => {
      const next = new Set(prev);

      // Ctrl/Cmd+Click = toggle individual
      if (event?.ctrlKey || event?.metaKey) {
        if (next.has(path)) {
          next.delete(path);
        } else {
          next.add(path);
        }
      }
      // Shift+Click = range select (handled by selectRange)
      else if (event?.shiftKey) {
        // This will be handled by selectRange if needed
        return prev;
      }
      // Normal click = clear others and select this
      else {
        next.clear();
        next.add(path);
      }

      return next;
    });
  }, []);

  /**
   * Select range between two paths
   */
  const selectRange = useCallback((start: string, end: string) => {
    const startIdx = entries.findIndex(e => e.path === start);
    const endIdx = entries.findIndex(e => e.path === end);

    if (startIdx === -1 || endIdx === -1) return;

    const [min, max] = startIdx < endIdx ? [startIdx, endIdx] : [endIdx, startIdx];

    setSelected(prev => {
      const next = new Set(prev);
      for (let i = min; i <= max; i++) {
        next.add(entries[i].path);
      }
      return next;
    });
  }, [entries]);

  /**
   * Select all entries
   */
  const selectAll = useCallback(() => {
    setSelected(new Set(entries.map(e => e.path)));
  }, [entries]);

  /**
   * Clear selection
   */
  const clearSelection = useCallback(() => {
    setSelected(new Set());
  }, []);

  /**
   * Get selected entry objects
   */
  const getSelectedEntries = useCallback((): FileEntry[] => {
    return entries.filter(e => selected.has(e.path));
  }, [entries, selected]);

  return {
    selected,
    isSelected,
    toggle,
    selectRange,
    selectAll,
    clearSelection,
    getSelectedEntries,
  };
}
