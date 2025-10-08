/**
 * useKeyboard Hook
 * Handles keyboard navigation for app grid
 */

import { useEffect, useCallback } from 'react';

interface KeyboardOptions {
  onSearch: () => void;
  onNavigate: (direction: 'up' | 'down' | 'left' | 'right') => void;
  onSelect: () => void;
  onEscape: () => void;
}

export function useKeyboard({ onSearch, onNavigate, onSelect, onEscape }: KeyboardOptions) {
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      // Focus search on /
      if (e.key === '/' && !e.metaKey && !e.ctrlKey) {
        e.preventDefault();
        onSearch();
        return;
      }

      // Close on Escape
      if (e.key === 'Escape') {
        onEscape();
        return;
      }

      // Navigation with arrow keys
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        onNavigate('up');
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        onNavigate('down');
      } else if (e.key === 'ArrowLeft') {
        e.preventDefault();
        onNavigate('left');
      } else if (e.key === 'ArrowRight') {
        e.preventDefault();
        onNavigate('right');
      } else if (e.key === 'Enter') {
        e.preventDefault();
        onSelect();
      }
    },
    [onSearch, onNavigate, onSelect, onEscape]
  );

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}

