/**
 * Selection Shortcuts Hook
 * Handles file selection keyboard shortcuts (Cmd+A, Escape)
 * Separate from app-specific shortcuts for proper separation of concerns
 */

import { useEffect, useCallback } from 'react';

interface UseSelectionShortcutsOptions {
  onSelectAll: () => void;
  onClearSelection: () => void;
  disabled?: boolean;
}

/**
 * Hook for selection keyboard shortcuts
 * Follows the same pattern as main UI but self-contained for native app
 */
export function useSelectionShortcuts(options: UseSelectionShortcutsOptions) {
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (options.disabled) return;

    const isCommand = event.metaKey || event.ctrlKey;

    // Command+A - Select All (context-aware)
    if (isCommand && event.key === 'a') {
      // Allow native behavior in input fields
      const target = event.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
        return; // Let browser handle it
      }
      event.preventDefault();
      options.onSelectAll();
      return;
    }

    // Escape - Clear selection
    if (event.key === 'Escape') {
      event.preventDefault();
      options.onClearSelection();
      return;
    }
  }, [options]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}

