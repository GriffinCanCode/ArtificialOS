/**
 * Clipboard Hook
 * Manages copy/cut/paste operations
 */

import { useState, useCallback } from 'react';
import type { UseClipboardReturn, ClipboardState } from '../types';

export function useClipboard(
  copyFn: (source: string, dest: string) => Promise<void>,
  moveFn: (source: string, dest: string) => Promise<void>
): UseClipboardReturn {
  const [clipboard, setClipboard] = useState<ClipboardState>({
    operation: null,
    paths: [],
  });

  /**
   * Copy paths to clipboard
   */
  const copy = useCallback((paths: string[]) => {
    setClipboard({
      operation: 'copy',
      paths,
    });
  }, []);

  /**
   * Cut paths to clipboard
   */
  const cut = useCallback((paths: string[]) => {
    setClipboard({
      operation: 'cut',
      paths,
    });
  }, []);

  /**
   * Paste from clipboard
   */
  const paste = useCallback(async (targetPath: string) => {
    if (!clipboard.operation || clipboard.paths.length === 0) {
      return;
    }

    const errors: string[] = [];

    for (const sourcePath of clipboard.paths) {
      const filename = sourcePath.split('/').pop() || 'unknown';
      const destPath = `${targetPath}/${filename}`;

      try {
        if (clipboard.operation === 'copy') {
          await copyFn(sourcePath, destPath);
        } else {
          await moveFn(sourcePath, destPath);
        }
      } catch (err) {
        errors.push(`${filename}: ${(err as Error).message}`);
      }
    }

    if (errors.length > 0) {
      throw new Error(`Paste failed:\n${errors.join('\n')}`);
    }

    // Clear clipboard after cut operation
    if (clipboard.operation === 'cut') {
      clear();
    }
  }, [clipboard, copyFn, moveFn]);

  /**
   * Clear clipboard
   */
  const clear = useCallback(() => {
    setClipboard({
      operation: null,
      paths: [],
    });
  }, []);

  const canPaste = clipboard.operation !== null && clipboard.paths.length > 0;

  return {
    clipboard,
    copy,
    cut,
    paste,
    canPaste,
    clear,
  };
}
