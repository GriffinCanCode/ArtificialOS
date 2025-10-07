/**
 * Keyboard Navigation Hook
 * Handles keyboard shortcuts and navigation
 */

import { useState, useCallback } from 'react';
import type { UseKeyboardReturn } from '../types';

export function useKeyboard(
  itemCount: number,
  onEnter: (index: number) => void,
  onDelete: () => void,
  onSelectAll: () => void,
  onCopy: () => void,
  onCut: () => void,
  onPaste: () => void
): UseKeyboardReturn {
  const [focusedIndex, setFocusedIndex] = useState(0);

  /**
   * Handle keyboard events
   */
  const handleKeyDown = useCallback((event: React.KeyboardEvent) => {
    const { key, ctrlKey, metaKey } = event;
    const modKey = ctrlKey || metaKey;

    switch (key) {
      // Navigation
      case 'ArrowDown':
        event.preventDefault();
        setFocusedIndex(prev => Math.min(prev + 1, itemCount - 1));
        break;

      case 'ArrowUp':
        event.preventDefault();
        setFocusedIndex(prev => Math.max(prev - 1, 0));
        break;

      case 'Home':
        event.preventDefault();
        setFocusedIndex(0);
        break;

      case 'End':
        event.preventDefault();
        setFocusedIndex(itemCount - 1);
        break;

      case 'PageDown':
        event.preventDefault();
        setFocusedIndex(prev => Math.min(prev + 10, itemCount - 1));
        break;

      case 'PageUp':
        event.preventDefault();
        setFocusedIndex(prev => Math.max(prev - 10, 0));
        break;

      // Actions
      case 'Enter':
        event.preventDefault();
        if (focusedIndex >= 0 && focusedIndex < itemCount) {
          onEnter(focusedIndex);
        }
        break;

      case 'Delete':
      case 'Backspace':
        if (!modKey) {
          event.preventDefault();
          onDelete();
        }
        break;

      // Shortcuts
      case 'a':
        if (modKey) {
          event.preventDefault();
          onSelectAll();
        }
        break;

      case 'c':
        if (modKey) {
          event.preventDefault();
          onCopy();
        }
        break;

      case 'x':
        if (modKey) {
          event.preventDefault();
          onCut();
        }
        break;

      case 'v':
        if (modKey) {
          event.preventDefault();
          onPaste();
        }
        break;
    }
  }, [itemCount, focusedIndex, onEnter, onDelete, onSelectAll, onCopy, onCut, onPaste]);

  return {
    focusedIndex,
    handleKeyDown,
  };
}
