/**
 * Keyboard Hook
 * App-specific keyboard shortcuts (Command Palette, Quick Access, etc.)
 * Selection shortcuts (Cmd+A, Escape) are handled by useSelectionShortcuts
 */

import { useCallback, useEffect } from 'react';

interface UseKeyboardOptions {
  onCommandP?: () => void;
  onCommandO?: () => void;
  onEscape?: () => void;
  onCommandEnter?: () => void;
  onSpace?: () => void;
}

/**
 * Hook for app-specific keyboard shortcuts
 * Handles File Explorer-specific commands only
 */
export function useKeyboard(options: UseKeyboardOptions) {
  const handleKeyDown = useCallback((event: React.KeyboardEvent | KeyboardEvent) => {
    const isCommand = event.metaKey || event.ctrlKey;

    // Command+P - Command Palette
    if (isCommand && event.key === 'p') {
      event.preventDefault();
      options.onCommandP?.();
      return;
    }

    // Command+O - Quick Access
    if (isCommand && event.key === 'o') {
      event.preventDefault();
      options.onCommandO?.();
      return;
    }

    // Escape - Close modals
    if (event.key === 'Escape') {
      event.preventDefault();
      options.onEscape?.();
      return;
    }

    // Command+Enter - Open file
    if (isCommand && event.key === 'Enter') {
      event.preventDefault();
      options.onCommandEnter?.();
      return;
    }

    // Space - Toggle preview
    if (event.key === ' ' && !(event.target as HTMLElement).matches('input, textarea')) {
      event.preventDefault();
      options.onSpace?.();
      return;
    }
  }, [options]);

  // Listen to global keyboard events
  useEffect(() => {
    const handler = (e: KeyboardEvent) => handleKeyDown(e);
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [handleKeyDown]);

  return {
    handleKeyDown: (e: React.KeyboardEvent) => handleKeyDown(e),
  };
}
